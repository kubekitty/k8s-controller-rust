// src/main.rs
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

mod config;
mod controller;
mod rules;
mod types;

use crate::{
    config::AuditConfig,
    controller::Controller,
    rules::RulesEngine,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting KubeKitty security controller");

    let config = AuditConfig {
        rules_dir: std::env::var("RULES_DIR").unwrap_or_else(|_| "/etc/kubekitty/rules".to_string()),
    };

    let client = kube::Client::try_default().await?;
    let rules_engine = RulesEngine::new(&config.rules_dir)?;
    let controller = Controller::new(client, rules_engine).await;
    
    // Start HTTP health check server
    tokio::spawn(health_check_server());

    // Watch for resources
    controller.start().await?;

    Ok(())
}

async fn health_check_server() {
    use warp::Filter;

    let health_route = warp::path!("healthz").map(|| "ok");

    warp::serve(health_route)
        .run(([0, 0, 0, 0], 8080))
        .await;
}