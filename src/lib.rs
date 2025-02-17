use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

pub mod api;
pub mod apidocs;
pub mod lexer;
pub mod modules;

use modules::analiticcl::{AnaliticclConfig, AnaliticclModule};
use modules::fst::{FstConfig, FstModule};
use modules::lookup::{LookupConfig, LookupModule};
use modules::Module;

pub use lexer::Term;

/// Maps a term to expansions, each `TermExpansion` corresponds to one source/module and may itself contain multiple expansions
pub type TermExpansions = HashMap<String, Vec<TermExpansion>>;

#[derive(Default)]
pub struct QueryExpander {
    config: Config,
    modules: Vec<Box<dyn Module>>,
    initialised: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    lookup: Vec<LookupConfig>,
    analiticcl: Vec<AnaliticclConfig>,
    fst: Vec<FstConfig>,
}

impl QueryExpander {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Adds a new module. Only valid before call to `load()`, will panic afterwards.
    pub fn add_module(&mut self, module: Box<dyn Module>) {
        if self.initialised {
            panic!("Can not add modules after load()!")
        }
        self.modules.push(module);
    }

    /// Adds a new module. Only valid before call to `load()`, will panic afterwards.
    pub fn with_module(mut self, module: Box<dyn Module>) -> Self {
        self.add_module(module);
        self
    }

    /// Returns an iterator over all the modules
    pub fn modules(&self) -> impl Iterator<Item = &dyn Module> {
        self.modules.iter().map(|x| x.as_ref())
    }

    /// Initialise all modules. This should be called once after all modules are loaded. Will panic if called multiple times.
    pub fn load(&mut self) -> Result<(), Error> {
        if self.initialised {
            panic!("load() can only be called once");
        }
        //MAYBE TODO: we could parallellize the loading for quicker startup time
        for lookupconfig in self.config.lookup.iter() {
            info!(
                "Adding Lookup module {} - {}",
                lookupconfig.id(),
                lookupconfig.name()
            );
            let mut module = LookupModule::new(lookupconfig.clone());
            module.load()?;
            self.modules.push(Box::new(module));
        }
        for fstconfig in self.config.fst.iter() {
            info!(
                "Adding Fst module {} - {}",
                fstconfig.id(),
                fstconfig.name()
            );
            let mut module = FstModule::new(fstconfig.clone());
            module.load()?;
            self.modules.push(Box::new(module));
        }
        for analiticclconfig in self.config.analiticcl.iter() {
            info!(
                "Adding Analiticcl module {} - {}",
                analiticclconfig.id(),
                analiticclconfig.name()
            );
            let mut module = AnaliticclModule::new(analiticclconfig.clone());
            module.load()?;
            self.modules.push(Box::new(module));
        }
        info!("All modules loaded");
        self.initialised = true;
        Ok(())
    }

    pub fn expand_query(
        &self,
        terms: &Vec<Term>,
        params: &QueryParams,
    ) -> Result<TermExpansions, Error> {
        let mut terms_map = TermExpansions::new();
        self.expand_query_into(&mut terms_map, terms, params)?;
        Ok(terms_map)
    }

    pub fn expand_query_into(
        &self,
        terms_map: &mut TermExpansions,
        terms: &Vec<Term>,
        params: &QueryParams,
    ) -> Result<(), Error> {
        let excludemods: Vec<_> = if let Some(mods) = params.get("", "excludemods") {
            value_to_str_array(mods)
        } else {
            Vec::new()
        };
        let includemods: Vec<_> = if let Some(mods) = params.get("", "includemods") {
            value_to_str_array(mods)
        } else {
            Vec::new()
        };
        for module in self.modules() {
            if (excludemods.is_empty() || !excludemods.contains(&module.id()))
                || (includemods.is_empty() || includemods.contains(&module.id()))
            {
                let expansion_map = module.expand_query(terms, params)?;
                for term in terms.iter() {
                    terms_map
                        .entry(term.as_str().to_string())
                        .and_modify(|expansions| {
                            if let Some(expansions2) = expansion_map.get(term.as_str()) {
                                for expansion in expansions2 {
                                    expansions.push(expansion.clone()); //TODO: work away the clone
                                }
                            }
                        })
                        .or_insert_with(|| {
                            if let Some(expansions2) = expansion_map.get(term.as_str()) {
                                expansions2.to_vec() //TODO: work away the clone
                            } else {
                                vec![]
                            }
                        });
                }
            }
        }
        Ok(())
    }
}

/// convert a json array of strings to a rust Vec<&str>
fn value_to_str_array(input: &Value) -> Vec<&str> {
    if let Value::Array(array) = input {
        let mut array_out = Vec::with_capacity(array.len());
        for value in array {
            if let Value::String(s) = value {
                array_out.push(s.as_str());
            }
        }
        array_out
    } else {
        Vec::new()
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct TermExpansion {
    expansions: Vec<String>,
    scores: Vec<f64>,
    source_id: Option<String>,
    source_name: Option<String>,
    link: Option<String>,
}

impl TermExpansion {
    pub fn with_source(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.source_id = Some(id.into());
        self.source_name = Some(name.into());
        self
    }

    pub fn with_link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn with_expansions(mut self, expansions: Vec<String>) -> Self {
        self.expansions = expansions;
        self
    }

    pub fn with_scores(mut self, scores: Vec<f64>) -> Self {
        self.scores = scores;
        self
    }

    pub fn add_variant_with_score(&mut self, expansion: impl Into<String>, score: f64) {
        self.expansions.push(expansion.into());
        self.scores.push(score);
    }

    pub fn add_variant(&mut self, expansion: impl Into<String>) {
        self.expansions.push(expansion.into());
    }

    pub fn expansions(&self) -> &Vec<String> {
        &self.expansions
    }

    pub fn scores(&self) -> &Vec<f64> {
        &self.scores
    }

    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    pub fn len(&self) -> usize {
        self.expansions.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.expansions.iter().map(|x| x.as_str())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParam {
    module_id: String,
    key: String,
    value: Value,
}

impl QueryParam {
    pub fn module_id(&self) -> &str {
        self.module_id.as_str()
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Holds arbitrary parameters passed to queries at runtime when requesting expansion
// The implementation uses a simple vec to save ourselves HashMap overhead.
pub struct QueryParams(Vec<QueryParam>);

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new key and value (builder pattern)
    pub fn with(
        mut self,
        module_id: impl Into<String>,
        key: impl Into<String>,
        value: Value,
    ) -> Self {
        self.insert(module_id, key, value);
        self
    }

    /// Insert a new key and value
    /// By convention, we use an empty module_id for a global scope.
    pub fn insert(&mut self, module_id: impl Into<String>, key: impl Into<String>, value: Value) {
        self.0.push(QueryParam {
            module_id: module_id.into(),
            key: key.into(),
            value,
        });
    }

    /// Check if a key exists. By convention, we use an empty module_id for a global scope.
    pub fn contains(&self, module_id: &str, key: &str) -> bool {
        for param in self.iter() {
            if param.module_id() == module_id && param.key() == key {
                return true;
            }
        }
        false
    }

    /// Iterate over all keys and values
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a QueryParam> {
        self.0.iter()
    }

    /// Retrieve a value by key
    /// By convention, we use an empty module_id for a global scope.
    pub fn get<'a>(&'a self, module_id: &str, key: &str) -> Option<&'a Value> {
        for param in self.iter() {
            if param.module_id() == module_id && param.key() == key {
                return Some(param.value());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    LoadError(String),
    QueryExpandError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadError(x) => {
                f.write_str("[Load error] ")?;
                f.write_str(x)
            }
            Self::QueryExpandError(x) => {
                f.write_str("[Query expansion error] ")?;
                f.write_str(x)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::LoadError(format!("{}", value))
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::LoadError(s) | Self::QueryExpandError(s) => serializer.serialize_str(s.as_str()),
        }
    }
}
