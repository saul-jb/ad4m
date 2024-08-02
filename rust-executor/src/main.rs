#[macro_use]
extern crate lazy_static;

pub mod agent;
mod config;
mod db;
mod entanglement_service;
mod globals;
mod graphql;
mod holochain_service;
pub mod init;
mod js_core;
mod languages;
mod neighbourhoods;
mod perspectives;
mod prolog;
mod prolog_service;
mod pubsub;
mod runtime_service;
mod types;
mod utils;
mod wallet;

#[cfg(test)]
mod test_utils;

use log::{error, info};
use rust_executor::Ad4mConfig;
use std::env;

use prolog_service::init_prolog_service;

use js_core::JsCore;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    prolog::run();
    env::set_var(
        "RUST_LOG",
        "holochain=warn,wasmer_compiler_cranelift=warn,rust_executor=info,warp::server",
    );
    env_logger::try_init().expect("could not logger");

    let mut config = Ad4mConfig::default();
    config.prepare();

    info!("Initializing Prolog service...");
    init_prolog_service().await;

    info!("Starting js_core... with config: {:#?}", config);

    let mut js_core_handle = JsCore::start(config.clone()).await;
    js_core_handle.initialized().await;
    info!("js_core initialized.");

    info!("Starting GraphQL...");
    match graphql::start_server(js_core_handle, config).await {
        Ok(_) => {
            info!("GraphQL server stopped.");
            std::process::exit(0);
        }
        Err(err) => {
            error!("GraphQL server stopped with error: {}", err);
            std::process::exit(1);
        }
    }
}
