use axum_cargo_registry::registory_storage::s3::S3RegistoryStorage;
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

    tracing::info!("Starting registry for s3");
    let storage = S3RegistoryStorage::from_env()
        .await
        .expect("Failed to create S3RegistoryStorage");

    let app = App::new(confg, storage);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.create_router()).await.unwrap();
}
