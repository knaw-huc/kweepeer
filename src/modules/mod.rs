pub mod analiticcl;
pub mod fst;
pub mod lookup;

use crate::lexer::Term;
use crate::{Error, QueryParams, TermExpansions};

/// This trait is implemented for all query expansions modules
pub trait Module: Send + Sync {
    /// Get the module type
    fn kind(&self) -> &'static str;

    /// Get the module identifier, some arbititrary short string
    fn id(&self) -> &str;

    /// Get the module name, a human-readable label
    fn name(&self) -> &str;

    /// Load the module. This *MUST* be called (once) prior to calling *expand_query()*.
    fn load(&mut self) -> Result<(), Error>;

    /// Expands a (decomposed) query. Note that `load()` *MUST* be called (once) prior to calling this for the first time, otherwise it will result in a panic.
    fn expand_query(
        &self,
        terms: &Vec<Term>,
        queryparams: &QueryParams,
    ) -> Result<TermExpansions, Error>;
}
