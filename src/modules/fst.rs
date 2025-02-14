use serde::Deserialize;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use tracing::{debug, info};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set, SetBuilder};

use crate::common::{TermExpansion, TermExpansions};
use crate::lexer::Term;
use crate::modules::{LoadError, Module};

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

    /// Levenshtein distance for lookups,
    distance: u8,

    /// Set this if the first line is a header
    #[serde(default)]
    skipfirstline: bool,
}

impl FstConfig {
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

    fn load(&mut self) -> Result<(), LoadError> {
        info!("Loading lexicon {}", self.config.file.as_path().display());
        let file = File::open(self.config.file.as_path())?;
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);
        let mut firstline = true;
        let mut builder = SetBuilder::memory();
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
                let line = buffer.trim();
                if !line.is_empty() {
                    builder.insert(line.as_bytes())?;
                }
            }
            buffer.clear();
        }
        info!("Building FST");
        self.set = Set::new(builder.into_inner()?)?;
        Ok(())
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        let mut expansions = TermExpansions::new();
        for term in terms {
            match Levenshtein::new(term.as_str(), self.config.distance as u32) {
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
        expansions
    }
}

impl From<fst::Error> for LoadError {
    fn from(value: fst::Error) -> Self {
        LoadError(format!("{}", value))
    }
}

impl From<fst::automaton::LevenshteinError> for LoadError {
    fn from(value: fst::automaton::LevenshteinError) -> Self {
        LoadError(format!("{}", value))
    }
}
