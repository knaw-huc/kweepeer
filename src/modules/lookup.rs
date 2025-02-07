use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;

use crate::common::{ApiError, TermExpansion, TermExpansions};
use crate::lexer::Term;
use crate::modules::{LoadError, Modular};

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

impl Modular for LookupModule {
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
        let file = File::open(self.config.file.as_path())?;
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);
        let mut firstline = true;
        while let Ok(bytes) = reader.read_line(&mut buffer) {
            if firstline {
                firstline = false;
                if self.config.skipfirstline {
                    continue;
                }
            }
            if bytes > 0 && buffer.chars().next() != Some('#') {
                let mut iter = buffer.trim().splitn(2, self.config.delimiter);
                if let (Some(keyword), Some(variants)) = (iter.next(), iter.next()) {
                    let variants: Vec<_> = variants
                        .splitn(2, self.config.delimiter2)
                        .map(|s| s.to_owned())
                        .collect();
                    self.data.variants.insert(keyword.to_owned(), variants);
                }
            }
        }
        Ok(())
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        let mut expansions = TermExpansions::new();
        for term in terms {
            if let Some(variants) = self.data.variants.get(term.as_str()) {
                expansions.insert(
                    term.as_str().to_string(),
                    vec![TermExpansion::default()
                        .with_source(self.config.id.as_str(), self.config.name.as_str())
                        .with_expansions(variants.to_vec())],
                );
            }
        }
        expansions
    }
}
