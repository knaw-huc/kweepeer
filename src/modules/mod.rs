mod lookup;

use std::fmt::Display;

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
    fn id(&self) -> &str;
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

    fn id(&self) -> &str {
        match self {
            Self::Lookup(x) => x.id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Lookup(x) => x.name(),
        }
    }
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
