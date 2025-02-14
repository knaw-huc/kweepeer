use axum::{
    http::HeaderValue,
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::value::Value;

use crate::lexer::Term;

use std::collections::HashMap;

/// Maps a term to expansions, each `TermExpansion` corresponds to one source/module and may itself contain multiple expansions
pub type TermExpansions = HashMap<String, Vec<TermExpansion>>;

#[derive(Debug)]
pub enum ApiResponse {
    QueryExpansion {
        /// Terms and expansions
        terms: TermExpansions,
        /// The input query
        original_query: String,
        /// A template for query expansion,
        /// expandable terms are replaced {{term}}, which refer back to the terms.
        query_expansion_template: String,
        /// The full expanded query
        query: String,
    },
    Modules(Vec<Value>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        let cors = (
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        match &self {
            Self::QueryExpansion { .. } => (StatusCode::OK, [cors], Json(&self)).into_response(),
            Self::Modules(data) => (StatusCode::OK, [cors], Json(data)).into_response(),
        }
    }
}

impl Serialize for ApiResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiResponse", 3)?;
        match self {
            Self::QueryExpansion {
                terms,
                original_query,
                query_expansion_template,
                query,
            } => {
                state.serialize_field("terms", terms)?;
                state.serialize_field("original_query", original_query)?;
                state.serialize_field("query_expansion_template", query_expansion_template)?;
                state.serialize_field("query", query)?;
            }
            Self::Modules(v) => state.serialize_field("modules", v)?,
        }
        state.end()
    }
}

impl ApiResponse {
    pub fn new_queryexpansion(
        terms: TermExpansions,
        query: &str,
        query_expansion_template: impl Into<String>,
    ) -> Self {
        Self::QueryExpansion {
            query_expansion_template: query_expansion_template.into(),
            terms,
            original_query: query.to_owned(),
            query: String::new(),
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    InternalError(&'static str),
    NotFound(&'static str),
    NotAcceptable(&'static str),
    PermissionDenied(&'static str),
    MissingArgument(&'static str),
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiError", 3)?;
        state.serialize_field("@type", "ApiError")?;
        match self {
            Self::NotFound(s) => {
                state.serialize_field("name", "NotFound")?;
                state.serialize_field("message", s)?;
            }
            Self::NotAcceptable(s) => {
                state.serialize_field("name", "NotAcceptable")?;
                state.serialize_field("message", s)?;
            }
            Self::PermissionDenied(s) => {
                state.serialize_field("name", "PermissionDenied")?;
                state.serialize_field("message", s)?;
            }
            Self::InternalError(s) => {
                state.serialize_field("name", "InternalError")?;
                state.serialize_field("message", s)?;
            }
            Self::MissingArgument(s) => {
                state.serialize_field("name", "MissingArgument")?;
                state.serialize_field("message", s)?;
            }
        }
        state.end()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let statuscode = match self {
            Self::InternalError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PermissionDenied(..) => StatusCode::FORBIDDEN,
            Self::NotAcceptable(..) => StatusCode::NOT_ACCEPTABLE,
            _ => StatusCode::NOT_FOUND,
        };
        (statuscode, Json(self)).into_response()
    }
}

impl From<axum::Error> for ApiError {
    fn from(_value: axum::Error) -> Self {
        Self::InternalError("web framework error")
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
