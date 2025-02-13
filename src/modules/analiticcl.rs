use serde::Deserialize;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::common::{TermExpansion, TermExpansions};
use crate::lexer::Term;
use crate::modules::{LoadError, Modular};

use analiticcl::{SearchParameters, VariantModel, VocabParams, Weights};

#[derive(Debug, Deserialize, Clone)]
pub struct Lexicon {
    filename: PathBuf,
    params: VocabParams,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VariantList {
    filename: PathBuf,

    #[serde(default)]
    params: Option<VocabParams>,

    #[serde(default)]
    transparent: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnaliticclConfig {
    /// Short identifier
    id: String,

    /// Human readable label
    name: String,

    #[serde(default)]
    weights: Weights,

    /// Alphabet file,
    alphabet: PathBuf,

    /// Lexicons or frequency lists
    #[serde(default)]
    lexicons: Vec<Lexicon>,

    /// Weighted variant lists
    #[serde(default)]
    variantlists: Vec<VariantList>,

    /// Confusable lists,
    #[serde(default)]
    confusable_lists: Vec<PathBuf>,

    /// Search parameters
    #[serde(default)]
    searchparams: SearchParameters,
}

/// A lexical module using anagram-based hashing
pub struct AnaliticclModule {
    config: AnaliticclConfig,

    /// the Variant Model from Analiticcl. None whilst not loaded yet.
    model: Option<VariantModel>,
}

impl Modular for AnaliticclModule {
    fn id(&self) -> &str {
        self.config.id.as_str()
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "analiticcl"
    }

    fn load(&mut self) -> Result<(), LoadError> {
        let mut model = VariantModel::new(
            &self.config.alphabet.to_string_lossy(),
            self.config.weights.clone(),
            0,
        );

        for lexicon in self.config.lexicons.iter() {
            info!("Loading lexicon {}", &lexicon.filename.to_string_lossy());
            model.read_vocabulary(&lexicon.filename.to_string_lossy(), &lexicon.params)?;
        }

        for confusable_list in self.config.confusable_lists.iter() {
            info!(
                "Loading confusable list {}",
                &confusable_list.to_string_lossy()
            );
            model.read_confusablelist(&confusable_list.to_string_lossy())?;
        }

        for variantlist in self.config.variantlists.iter() {
            info!(
                "Loading weighted variant list {}",
                &variantlist.filename.to_string_lossy()
            );
            model.read_variants(
                &variantlist.filename.to_string_lossy(),
                variantlist.params.as_ref(),
                variantlist.transparent,
            )?;
        }

        model.build();

        self.model = Some(model);
        Ok(())
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        let mut expansions = TermExpansions::new();
        for term in terms {
            debug!("Looking up {}", term.as_str());
            if let Some(model) = self.model.as_ref() {
                let mut termexpansion = TermExpansion::default()
                    .with_source(self.config.id.as_str(), self.config.name.as_str());
                for variant in model.find_variants(term.as_str(), &self.config.searchparams) {
                    let variant_text = &model
                        .decoder
                        .get(variant.vocab_id as usize)
                        .expect("vocab ID must be in decoder")
                        .text;
                    termexpansion.add_variant_with_score(
                        variant_text,
                        self.config.searchparams.freq_weight as f64,
                    );
                }
                expansions.insert(term.as_str().to_string(), vec![termexpansion]);
            } else {
                panic!("expand_query() was called before load()!");
            }
        }
        expansions
    }
}
