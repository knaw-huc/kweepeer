use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::common::{ApiError, TermExpansion, TermExpansions};
use crate::lexer::Term;
use crate::modules::{LoadError, Module};

/// A simple hash-map-based lookup module
/// mapping keywords to variants.
pub struct LookupModule {
    config: LookupConfig,
    data: LookupData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LookupConfig {
    /// Short identifier
    id: String,

    /// Human readable label
    name: String,

    /// The path to the variant list file that holds the lookup data.
    /// This is a simple tab-separated file with the keys in the first
    /// columns and variants in the subsequent (dynamic-sized) columns
    /// It will be loaded into memory entirely.
    file: PathBuf,

    #[serde(default = "tab")]
    delimiter: char,

    #[serde(default = "tab")]
    delimiter2: char,

    /// Set this if the first line is a header
    #[serde(default)]
    skipfirstline: bool,

    /// Allow numeric fields, otherwise they will be ignored (which is useful to filter out frequency/score information from input files)
    #[serde(default)]
    allow_numeric: bool,
}

impl LookupConfig {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn tab() -> char {
    '\t'
}

#[derive(Default)]
pub struct LookupData {
    variants: HashMap<String, Vec<String>>,
}

impl LookupModule {
    pub fn new(config: LookupConfig) -> Self {
        Self {
            config,
            data: LookupData::default(),
        }
    }
}

impl Module for LookupModule {
    fn id(&self) -> &str {
        self.config.id.as_str()
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "lookup"
    }

    fn load(&mut self) -> Result<(), LoadError> {
        info!("Loading lexicon {}", self.config.file.as_path().display());
        let file = File::open(self.config.file.as_path())?;
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);
        let mut firstline = true;
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
                let mut iter = buffer.trim().splitn(2, self.config.delimiter);
                if let (Some(keyword), Some(variants)) = (iter.next(), iter.next()) {
                    let variants: Vec<_> = variants
                        .split(self.config.delimiter2)
                        .filter_map(|s| {
                            //check if field is not purely numeric, ignore if it is
                            if self.config.allow_numeric || s.parse::<f64>().is_err() {
                                Some(s.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !variants.is_empty() {
                        self.data.variants.insert(keyword.to_owned(), variants);
                    }
                }
            }
            buffer.clear();
        }
        info!("Loaded {} terms", self.data.variants.len());
        Ok(())
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        let mut expansions = TermExpansions::new();
        for term in terms {
            debug!("Looking up {}", term.as_str());
            if let Some(variants) = self.data.variants.get(term.as_str()) {
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
        }
        expansions
    }
}
