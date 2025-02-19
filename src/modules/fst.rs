use serde::Deserialize;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use tracing::{debug, info};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set, SetBuilder};

use crate::lexer::Term;
use crate::modules::Module;
use crate::{Error, QueryParams, TermExpansion, TermExpansions};

/// A simple hash-map-based lookup module
/// mapping keywords to variants.
pub struct FstModule {
    config: FstConfig,
    set: Set<Vec<u8>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FstConfig {
    /// Short identifier
    id: String,

    /// Human readable label
    name: String,

    /// The path to the lexicon (a simple wordlist, one word per line, the entries *MUST* be in lexographical order!
    file: PathBuf,

    /// Default Levenshtein distance for lookups,
    distance: u8,

    /// Is the lexicon already sorted lexographically? If it is, setting this to true improves loading time/memory consumption
    #[serde(default)]
    sorted: bool,

    /// Set this if the first line is a header
    #[serde(default)]
    skipfirstline: bool,
}

impl FstConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        file: impl Into<PathBuf>,
        distance: u8,
        sorted: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            file: file.into(),
            distance,
            sorted,
            skipfirstline: false,
        }
    }

    pub fn with_distance(mut self, distance: u8) -> Self {
        self.distance = distance;
        self
    }

    pub fn with_skipfirstline(mut self) -> Self {
        self.skipfirstline = true;
        self
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl FstModule {
    pub fn new(config: FstConfig) -> Self {
        Self {
            config,
            set: Set::default(),
        }
    }
}

impl Module for FstModule {
    fn id(&self) -> &str {
        self.config.id.as_str()
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "fst"
    }

    fn load(&mut self) -> Result<(), Error> {
        info!("Loading lexicon {}", self.config.file.as_path().display());
        let file = File::open(self.config.file.as_path())?;
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);
        let mut firstline = true;
        let mut builder = SetBuilder::memory();
        let mut entries: Vec<String> = Vec::new();
        while let Ok(bytes) = reader.read_line(&mut buffer) {
            if bytes == 0 {
                //EOF
                break;
            }
            if firstline {
                firstline = false;
                if self.config.skipfirstline {
                    buffer.clear();
                    continue;
                }
            }
            if buffer.chars().next() != Some('#') {
                if let Some(line) = buffer.trim().splitn(2, '\t').next() {
                    if !line.is_empty() {
                        if self.config.sorted {
                            builder.insert(line.as_bytes())?;
                        } else {
                            entries.push(line.to_owned());
                        }
                    }
                }
            }
            buffer.clear();
        }
        if !entries.is_empty() {
            entries.sort();
            for entry in entries {
                builder.insert(entry.as_bytes())?;
            }
        }
        info!("Building FST");
        self.set = Set::new(builder.into_inner()?)?;
        Ok(())
    }

    fn expand_query(
        &self,
        terms: &Vec<Term>,
        params: &QueryParams,
    ) -> Result<TermExpansions, Error> {
        let distance = if let Some(param) = params.get(self.id(), "distance") {
            param.as_u64().ok_or_else(|| {
                Error::QueryExpandError("invalid value for distance parameter".into())
            })? as u32
        } else {
            self.config.distance as u32
        };
        let mut expansions = TermExpansions::new();
        for term in terms {
            match Levenshtein::new(term.as_str(), distance) {
                Ok(levaut) => {
                    debug!("Looking up {}", term.as_str());
                    let stream = self.set.search(levaut).into_stream();
                    if let Ok(variants) = stream.into_strs() {
                        if !variants.is_empty() {
                            debug!("found {} expansions", variants.len());
                            expansions.insert(
                                term.as_str().to_string(),
                                vec![TermExpansion::default()
                                    .with_source(self.config.id.as_str(), self.config.name.as_str())
                                    .with_expansions(variants.to_vec())],
                            );
                        } else {
                            debug!("not found");
                        }
                    } else {
                        debug!("UTF-8 decoding error, no results returned");
                    }
                }
                Err(e) => debug!("Can't build FST for term '{}': {}", term.as_str(), e),
            }
        }
        Ok(expansions)
    }
}

impl From<fst::Error> for Error {
    fn from(value: fst::Error) -> Self {
        Self::LoadError(format!("{}", value))
    }
}

impl From<fst::automaton::LevenshteinError> for Error {
    fn from(value: fst::automaton::LevenshteinError) -> Self {
        Self::QueryExpandError(format!("{}", value))
    }
}

mod tests {
    use super::*;

    fn init_test() -> Result<FstModule, Error> {
        let mut testdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        testdir.push("test");
        let mut lexicon_file = testdir.clone();
        lexicon_file.push("test.nofreq.lexicon");
        let config = FstConfig {
            id: "fst".into(),
            name: "fst".into(),
            file: lexicon_file,
            distance: 2,
            sorted: false,
            skipfirstline: false,
        };
        Ok(FstModule::new(config))
    }

    #[test]
    pub fn test001_lookup_load() -> Result<(), Error> {
        let mut module = init_test()?;
        module.load()?;
        Ok(())
    }

    #[test]
    pub fn test002_lookup_query() -> Result<(), Error> {
        let mut module = init_test()?;
        module.load()?;
        let terms = vec![Term::Singular("belangrijk")];
        let expansions = module.expand_query(&terms, &QueryParams::new())?;
        assert_eq!(expansions.len(), 1, "Checking number of terms returned");
        let termexpansion = expansions
            .get("belangrijk")
            .expect("term must exists")
            .get(0)
            .expect("term must have results");
        assert_eq!(termexpansion.source_id(), Some("fst"), "Checking source id");
        assert_eq!(
            termexpansion.source_name(),
            Some("fst"),
            "Checking source name"
        );
        assert_eq!(
            termexpansion.iter().collect::<Vec<_>>(),
            [
                "belangrijk",
                "belangrijke",
                "belangrijker",
                "belangrijks",
                "belangrijkst",
                "onbelangrijk"
            ],
            "Checking returned expansions"
        );
        Ok(())
    }

    #[test]
    pub fn test002_lookup_query_nomatch() -> Result<(), Error> {
        let mut module = init_test()?;
        module.load()?;
        let terms = vec![Term::Singular("blah")];
        let expansions = module.expand_query(&terms, &QueryParams::new())?;
        assert_eq!(expansions.len(), 0, "Checking number of terms returned");
        Ok(())
    }
}
