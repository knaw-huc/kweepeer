mod lookup;

use crate::common::{ApiError, TermExpansions};
use crate::lexer::Term;

pub use lookup::*;

pub enum Module {
    Lookup(LookupModule),
}

#[derive(Debug, Clone)]
pub struct LoadError(String);

pub trait Modular {
    fn kind(&self) -> &'static str;
    fn load(&mut self) -> Result<(), LoadError>;
    fn name(&self) -> &str;
    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions;
}

impl Modular for Module {
    fn kind(&self) -> &'static str {
        match self {
            Self::Lookup(x) => x.kind(),
        }
    }

    fn load(&mut self) -> Result<(), LoadError> {
        match self {
            Self::Lookup(x) => x.load(),
        }
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        match self {
            Self::Lookup(x) => x.expand_query(terms),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Lookup(x) => x.name(),
        }
    }
}
