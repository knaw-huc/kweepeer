use axum::{
    body::Body, extract::Path, extract::Query, extract::State, http::HeaderMap, http::HeaderValue,
    http::Request, routing::get, routing::post, Form, Router,
};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use serde::Deserialize;
use serde_json::json;
use toml;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod apidocs;
mod common;
mod lexer;
use common::{ApiError, ApiResponse, TermExpansion, TermExpansions};
use lexer::Term;
mod modules;
use modules::{AnaliticclConfig, AnaliticclModule};
use modules::{FstConfig, FstModule};
use modules::{LoadError, Module};
use modules::{LookupConfig, LookupModule};

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

struct AppState {
    args: Args,
    config: Config,
    modules: Vec<Box<dyn Module>>,
}

#[derive(Deserialize, Default)]
struct Config {
    lookup: Vec<LookupConfig>,
    analiticcl: Vec<AnaliticclConfig>,
    fst: Vec<FstConfig>,
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

    let mut state = AppState {
        args: args.clone(),
        config,
        modules: vec![],
    };

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
    state: State<Arc<AppState>>,
) -> Result<ApiResponse, ApiError> {
    let excludemods: Vec<_> = params
        .get("exclude")
        .into_iter()
        .map(|v| v.split(","))
        .flatten()
        .collect();
    let includemods: Vec<_> = params
        .get("include")
        .into_iter()
        .map(|v| v.split(","))
        .flatten()
        .collect();
    if let Some(querystring) = params.get("q") {
        let mut terms_map = TermExpansions::new();
        let (terms, query_template) = Term::extract_from_query(querystring);
        for module in state.modules.iter() {
            if (excludemods.is_empty() || !excludemods.contains(&module.id()))
                || (includemods.is_empty() || includemods.contains(&module.id()))
            {
                let expansion_map = module.expand_query(&terms);
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
        Ok(ApiResponse::new_queryexpansion(
            terms_map,
            querystring,
            query_template,
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
async fn list_modules(state: State<Arc<AppState>>) -> Result<ApiResponse, ApiError> {
    let mut modules = Vec::new();
    for module in state.modules.iter() {
        modules.push(json!({"id": module.id(), "name": module.name(), "type": module.kind()}));
    }
    Ok(ApiResponse::Modules(modules))
}

//TODO: modules endpoint to query available modules

impl AppState {
    fn load(&mut self) -> Result<(), LoadError> {
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
        Ok(())
    }
}
