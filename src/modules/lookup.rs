use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::common::{ApiError, TermExpansion, TermExpansions};
use crate::lexer::Term;
use crate::modules::Modular;

#[derive(Debug, Deserialize, Clone)]
pub struct LookupConfig {
    name: String,

    /// The path to the variant list file that holds the lookup data.
    /// This is a simple tab-separated file with the keys in the first
    /// columns and variants in the subsequent (dynamic-sized) columns
    /// It will be loaded into memory entirely.
    file: PathBuf,
}

#[derive(Default)]
pub struct LookupData {
    variants: HashMap<String, Vec<String>>,
}

pub struct LookupModule {
    config: LookupConfig,
    data: LookupData,
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
    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "lookup"
    }

    fn load(&mut self) -> Result<(), ApiError> {
        Ok(())
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        let mut expansions = TermExpansions::new();
        for term in terms {
            if let Some(variants) = self.data.variants.get(term.as_str()) {
                expansions.insert(
                    term.as_str().to_string(),
                    vec![TermExpansion::default()
                        .with_source(self.config.name.as_str())
                        .with_expansions(variants.to_vec())],
                );
            }
        }
        expansions
    }
}
