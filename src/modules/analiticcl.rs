use serde::Deserialize;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::lexer::Term;
use crate::modules::Module;
use crate::{Error, QueryParams, TermExpansion, TermExpansions};

use analiticcl::{SearchParameters, VariantModel, VocabParams, Weights};

#[derive(Debug, Deserialize, Clone)]
pub struct Lexicon {
    filename: PathBuf,

    #[serde(default)]
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

impl AnaliticclConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        alphabet: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            alphabet: alphabet.into(),
            weights: Weights::default(),
            lexicons: Vec::new(),
            variantlists: Vec::new(),
            confusable_lists: Vec::new(),
            searchparams: SearchParameters::default(),
        }
    }

    pub fn with_lexicon(mut self, filename: impl Into<PathBuf>, params: VocabParams) -> Self {
        self.lexicons.push(Lexicon {
            filename: filename.into(),
            params,
        });
        self
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// A lexical module using anagram-based hashing
pub struct AnaliticclModule {
    config: AnaliticclConfig,

    /// the Variant Model from Analiticcl. None whilst not loaded yet.
    model: Option<VariantModel>,
}

impl AnaliticclModule {
    pub fn new(config: AnaliticclConfig) -> Self {
        Self {
            config,
            model: None,
        }
    }
}

impl Module for AnaliticclModule {
    fn id(&self) -> &str {
        self.config.id.as_str()
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "analiticcl"
    }

    fn load(&mut self) -> Result<(), Error> {
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

    fn expand_query(
        &self,
        terms: &Vec<Term>,
        params: &QueryParams,
    ) -> Result<TermExpansions, Error> {
        //construct analiticcl searchparams from params
        let searchparams: Option<SearchParameters> =
            if params.iter_for_module(self.id()).next().is_some() {
                let mut searchparams: SearchParameters = self.config.searchparams.clone();
                for param in params.iter_for_module(self.id()) {
                    match param.key() {
                        "max_matches" => {
                            searchparams = searchparams.with_max_matches(
                                param.value().as_u64().ok_or_else(|| {
                                    Error::QueryExpandError(
                                        "invalid value for max_matches parameter".into(),
                                    )
                                })? as usize,
                            )
                        }
                        "edit_distance" => {
                            //MAYBE TODO: absolute threshold only for now
                            searchparams = searchparams.with_edit_distance(
                                analiticcl::DistanceThreshold::Absolute(
                                    param.value().as_u64().ok_or_else(|| {
                                        Error::QueryExpandError(
                                            "invalid value for edit_distance parameter".into(),
                                        )
                                    })? as u8,
                                ),
                            )
                        }
                        "anagram_distance" => {
                            //MAYBE TODO: absolute threshold only for now
                            searchparams = searchparams.with_anagram_distance(
                                analiticcl::DistanceThreshold::Absolute(
                                    param.value().as_u64().ok_or_else(|| {
                                        Error::QueryExpandError(
                                            "invalid value for anagram_distance parameter".into(),
                                        )
                                    })? as u8,
                                ),
                            )
                        }
                        "score_threshold" => {
                            searchparams = searchparams.with_score_threshold(
                                param.value().as_f64().ok_or_else(|| {
                                    Error::QueryExpandError(
                                        "invalid value for edit_distance parameter".into(),
                                    )
                                })? as f64,
                            )
                        }
                        "cutoff_threshold" => {
                            searchparams = searchparams.with_cutoff_threshold(
                                param.value().as_f64().ok_or_else(|| {
                                    Error::QueryExpandError(
                                        "invalid value for cutoff_threshold parameter".into(),
                                    )
                                })? as f64,
                            )
                        }
                        //TODO: parse remaining analiticcl parameters
                        x => {
                            return Err(Error::QueryExpandError(format!(
                                "Got unexpected parameter for analiticcl: {}",
                                x
                            )))
                        }
                    }
                }
                Some(searchparams)
            } else {
                //no search parameters specified, we fall back to borrowing the config default later
                None
            };

        let mut expansions = TermExpansions::new();
        for term in terms {
            debug!("Looking up {}", term.as_str());
            if let Some(model) = self.model.as_ref() {
                let mut termexpansion = TermExpansion::default()
                    .with_source(self.config.id.as_str(), self.config.name.as_str());
                let mut found = false;
                for variant in model.find_variants(
                    term.as_str(),
                    if let Some(searchparams) = searchparams.as_ref() {
                        searchparams
                    } else {
                        &self.config.searchparams
                    },
                ) {
                    found = true;
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
                if found {
                    expansions.insert(term.as_str().to_string(), vec![termexpansion]);
                }
            } else {
                panic!("expand_query() was called before load()!");
            }
        }
        Ok(expansions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test() -> Result<AnaliticclModule, Error> {
        let mut testdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        testdir.push("test");
        let mut alphabet_file = testdir.clone();
        alphabet_file.push("simple.alphabet.tsv");
        let mut lexicon_file = testdir.clone();
        lexicon_file.push("test.freq.lexicon");
        let config = AnaliticclConfig::new("analiticcl", "analiticcl", alphabet_file)
            .with_lexicon(lexicon_file, VocabParams::default());
        Ok(AnaliticclModule::new(config))
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
        let expansions = module.expand_query(&terms, &QueryParams::default())?;
        assert_eq!(expansions.len(), 1, "Checking number of terms returned");
        let termexpansion = expansions
            .get("belangrijk")
            .expect("term must exists")
            .get(0)
            .expect("term must have results");
        assert_eq!(
            termexpansion.source_id(),
            Some("analiticcl"),
            "Checking source id"
        );
        assert_eq!(
            termexpansion.source_name(),
            Some("analiticcl"),
            "Checking source name"
        );
        assert_eq!(
            termexpansion.iter().collect::<Vec<_>>(),
            [
                "belangrijk",
                "belangrijke",
                "belangrijks",
                "belangrijker",
                "onbelangrijk",
                "belangrijkst",
                "belangrijkste",
                "belangrijkere",
                "belangrijkers",
                "onbelangrijke",
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
        let expansions = module.expand_query(&terms, &QueryParams::default())?;
        assert_eq!(expansions.len(), 0, "Checking number of terms returned");
        Ok(())
    }
}
