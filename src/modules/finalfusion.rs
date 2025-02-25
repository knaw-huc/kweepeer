use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use tracing::debug;

use crate::lexer::Term;
use crate::modules::Module;
use crate::{Error, QueryParams, TermExpansion, TermExpansions};

use finalfusion::prelude::*;
use finalfusion::similarity::WordSimilarity;

#[derive(Debug, Deserialize, Clone)]
pub struct FinalFusionConfig {
    /// Short identifier
    id: String,

    /// Human readable label
    name: String,

    /// Word-embeddings file
    file: PathBuf,

    /// Nearest Neighbours
    k: usize,
}

impl FinalFusionConfig {
    pub fn new(id: impl Into<String>, name: impl Into<String>, file: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            file: file.into(),
            k: 10,
        }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
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
pub struct FinalFusionModule {
    config: FinalFusionConfig,

    /// the Variant Model from Analiticcl. None whilst not loaded yet.
    model: Option<Embeddings<VocabWrap, StorageViewWrap>>,
}

impl FinalFusionModule {
    pub fn new(config: FinalFusionConfig) -> Self {
        Self {
            config,
            model: None,
        }
    }
}

impl Module for FinalFusionModule {
    fn id(&self) -> &str {
        self.config.id.as_str()
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn kind(&self) -> &'static str {
        "finalfusion"
    }

    fn load(&mut self) -> Result<(), Error> {
        let mut reader = BufReader::new(File::open(self.config.file.as_path())?);
        let embeddings = Embeddings::read_embeddings(&mut reader)?;
        self.model = Some(embeddings);
        Ok(())
    }

    fn expand_query(
        &self,
        terms: &Vec<Term>,
        params: &QueryParams,
    ) -> Result<TermExpansions, Error> {
        let k = if let Some(param) = params.get(self.id(), "k") {
            param.as_u64().ok_or_else(|| {
                Error::QueryExpandError("invalid value for k (nearest-neighbours) parameter".into())
            })? as usize
        } else {
            self.config.k
        };
        let mut expansions = TermExpansions::new();
        for term in terms {
            debug!("Looking up {}", term.as_str());
            if let Some(model) = self.model.as_ref() {
                let mut termexpansion = TermExpansion::default().with_source(self);

                if let Some(results) = model.word_similarity(term.as_str(), k, None) {
                    for variant in results {
                        termexpansion.add_variant_with_score(
                            variant.word(),
                            variant.cosine_similarity() as f64,
                        );
                    }
                    expansions.insert(term.as_str().to_string(), vec![termexpansion]);
                }
            } else {
                panic!("expand_query() was called before load()!");
            }
        }
        Ok(expansions)
    }
}

impl From<finalfusion::error::Error> for Error {
    fn from(value: finalfusion::error::Error) -> Self {
        Self::LoadError(format!("{}", value))
    }
}
