mod analiticcl;
mod fst;
mod lookup;

use crate::common::TermExpansions;
use crate::lexer::Term;

pub use crate::modules::analiticcl::*;
pub use crate::modules::fst::*;
pub use lookup::*;

#[derive(Debug, Clone)]
pub struct LoadError(String);

pub trait Module: Send + Sync {
    fn kind(&self) -> &'static str;
    fn load(&mut self) -> Result<(), LoadError>;
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions;
}

impl From<std::io::Error> for LoadError {
    fn from(value: std::io::Error) -> Self {
        LoadError(format!("{}", value))
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}
