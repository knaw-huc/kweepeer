mod analiticcl;
mod fst;
mod lookup;

use crate::common::TermExpansions;
use crate::lexer::Term;

pub use crate::modules::analiticcl::*;
pub use crate::modules::fst::*;
pub use lookup::*;

pub enum Module {
    Lookup(LookupModule),
    Analiticcl(AnaliticclModule),
    Fst(FstModule),
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
            Self::Analiticcl(x) => x.kind(),
            Self::Fst(x) => x.kind(),
        }
    }

    fn load(&mut self) -> Result<(), LoadError> {
        match self {
            Self::Lookup(x) => x.load(),
            Self::Analiticcl(x) => x.load(),
            Self::Fst(x) => x.load(),
        }
    }

    fn expand_query(&self, terms: &Vec<Term>) -> TermExpansions {
        match self {
            Self::Lookup(x) => x.expand_query(terms),
            Self::Analiticcl(x) => x.expand_query(terms),
            Self::Fst(x) => x.expand_query(terms),
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Lookup(x) => x.id(),
            Self::Analiticcl(x) => x.id(),
            Self::Fst(x) => x.id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Lookup(x) => x.name(),
            Self::Analiticcl(x) => x.name(),
            Self::Fst(x) => x.name(),
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
