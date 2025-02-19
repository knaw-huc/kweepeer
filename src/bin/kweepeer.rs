use axum::{
    extract::Path, extract::Query, extract::State, http::HeaderMap, http::HeaderValue,
    http::Request, routing::get, Form, Router,
};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use serde_json::json;
use toml;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use kweepeer::api::{ApiError, ApiResponse};
use kweepeer::*;

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

    #[arg(long = "config", short, default_value = "config.toml")]
    config_path: PathBuf,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        query_entrypoint,
        list_modules
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

    info!("Loading configuration from {}", &args.config_path.display());
    let toml_string =
        std::fs::read_to_string(&args.config_path).expect("Unable to read configuration file");
    let config: Config = toml::from_str(&toml_string).expect("Unable to parse configuration file");

    let mut state = QueryExpander::new().with_config(config);

    // Load all the modules
    state.load().expect("Failure whilst loading");

    let app = Router::new()
        .route("/", get(query_entrypoint))
        .route("/modules", get(list_modules))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

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
        ("include" = String, Query, description = "Comma separated list of modules to include", allow_reserved),
        ("exclude" = String, Query, description = "Comma separated list of modules to exclude", allow_reserved),
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
    state: State<Arc<QueryExpander>>,
) -> Result<ApiResponse, ApiError> {
    if let Some(querystring) = params.get("q") {
        let mut terms_map = TermExpansions::new();
        let (terms, query_template) = Term::extract_from_query(querystring);
        let params: QueryParams = (&params).into();
        state.expand_query_into(&mut terms_map, &terms, &params)?;
        let resolved_template =
            state.resolve_query_template(query_template.as_str(), &terms_map)?;
        Ok(ApiResponse::new_queryexpansion(
            terms_map,
            querystring,
            query_template,
            resolved_template,
        ))
    } else {
        Err(ApiError::MissingArgument("query"))
    }
}

#[utoipa::path(
    get,
    path = "/modules",
    params(
    ),
    responses(
        (status = 200, description = "Returns all available modules",content(
            (String = "application/json"),
        )),
    )
)]
async fn list_modules(state: State<Arc<QueryExpander>>) -> Result<ApiResponse, ApiError> {
    let mut modules = Vec::new();
    for module in state.modules() {
        modules.push(json!({"id": module.id(), "name": module.name(), "type": module.kind()}));
    }
    Ok(ApiResponse::Modules(modules))
}
