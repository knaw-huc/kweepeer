use clap::Parser;
use std::io::{stdin, BufRead};
use std::path::PathBuf;
use tracing::info;

use toml;

use kweepeer::api::ApiResponse;
use kweepeer::*;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long, default_value_t = false, help = "Debug mode")]
    debug: bool,

    #[arg(long = "config", short, default_value = "config.toml")]
    config_path: PathBuf,
}

fn main() -> Result<(), kweepeer::Error> {
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
    state.load().expect("Failure whilst loading modules");

    info!("Reading queries from standard input");

    let stdin = stdin();
    for line in stdin.lock().lines() {
        if let Ok(querystring) = line {
            let mut terms_map = TermExpansions::new();
            let (terms, query_template) = Term::extract_from_query(&querystring);
            let params = QueryParams::default(); //TODO: parse parameters from args
            state.expand_query_into(&mut terms_map, &terms, &params)?;
            let response = ApiResponse::new_queryexpansion(terms_map, &querystring, query_template);
            match serde_json::to_string_pretty(&response) {
                Ok(s) => println!("{}", s),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(2);
                }
            }
        }
    }
    Ok(())
}
