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
pub type TermExpansions = HashMap<String, Vec<TermExpansion>>;

#[derive(Debug)]
pub enum ApiResponse {
    QueryExpansion {
        /// Terms and expansions
        terms: TermExpansions,
        /// The input query
        original_query: String,
        /// The expanded query
        query: String,
    },
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        let cors = (
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        match &self {
            Self::QueryExpansion { .. } => (StatusCode::OK, [cors], Json(&self)).into_response(),
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
                query,
            } => {
                state.serialize_field("terms", terms)?;
                state.serialize_field("original_query", original_query)?;
                state.serialize_field("query", query)?;
            }
        }
        state.end()
    }
}

impl ApiResponse {
    pub fn new_queryexpansion(terms: &Vec<Term>, query: &str) -> Self {
        let mut terms_map = HashMap::new();
        for term in terms {
            terms_map.insert(term.as_str().to_owned(), vec![]);
        }
        Self::QueryExpansion {
            terms: terms_map,
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

#[derive(Debug, Serialize, Default)]
pub struct TermExpansion {
    expansions: Vec<String>,
    scores: Vec<f64>,
    source: Option<String>,
    link: Option<String>,
}

impl TermExpansion {
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
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
}
