use axum_cargo_registry::registory_storage::local::LocalStorage;
use axum_cargo_registry::{App, Config};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    let confg = Config {
        domain: "http://localhost:3000".to_string(),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting local registry");
    let working_dir = std::env::var("REGISTRY_PATH")
        .map(|path| path.into())
        .or_else(|_| std::env::current_dir())
        .unwrap();
    tracing::info!("Working directory: {:?}", working_dir);
    let storage = LocalStorage::new(working_dir.join("index"), working_dir.join("crates"));

    let app = App::new(confg, storage);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.create_router()).await.unwrap();
}
