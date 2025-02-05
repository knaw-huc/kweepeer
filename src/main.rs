use axum::{
    body::Body, extract::Path, extract::Query, extract::State, http::HeaderMap, http::HeaderValue,
    http::Request, routing::get, routing::post, Form, Router,
};
use clap::Parser;
use std::collections::HashMap;
use tower_http::trace::TraceLayer;
use tracing::{debug, error};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod apidocs;
mod common;
mod lexer;
use common::{ApiError, ApiResponse, TermExpansion};
use lexer::Term;

struct AppState {
    debug: bool,
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(
        short,
        long,
        default_value_os = "127.0.0.1:8080",
        help = "The host and port to bind to"
    )]
    bind: String,

    #[arg(
        long,
        default_value_t = false,
        help = "Output logging info on incoming requests"
    )]
    debug: bool,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        query_entrypoint
    ),
    tags(
        (name = "kweepeer", description = "A generic webservice for interactive query expansion, expansion is provided via various modules")
    )
)]
pub struct ApiDoc;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let app = Router::new()
        .route("/", get(query_entrypoint))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(args.clone());

    //allow trailing slashes as well: (conflicts with swagger-ui!)
    //let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    eprintln!("[kweepeer] listening on {}", args.bind);
    let listener = tokio::net::TcpListener::bind(args.bind).await.unwrap();
    axum::serve(
        listener, app,
        //ServiceExt::<axum::http::Request<Body>>::into_make_service(app),
    )
    .await
    .unwrap();
}

#[utoipa::path(
    get,
    path = "/",
    params(
        ("q" = String, Query, description = "A query in Lucene syntax", allow_reserved),
    ),
    responses(
        (status = 200, description = "Query result",content(
            (String = "application/json"),
        )),
        (status = 404, body = apidocs::ApiError, description = "Return when the query is invalid or another error occurs", content_type = "application/json"),
    )
)]
/// Receive and process a query. This is the main entrypoint
async fn query_entrypoint(
    Query(params): Query<HashMap<String, String>>,
    _state: State<Args>,
) -> Result<ApiResponse, ApiError> {
    if let Some(querystring) = params.get("q") {
        let terms = Term::extract_from_query(querystring);
        Ok(ApiResponse::new_queryexpansion(&terms, querystring))
    } else {
        Err(ApiError::MissingArgument("query"))
    }
}
