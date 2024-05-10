use axum_cargo_registry::registory_storage::local::LocalStorage;
use axum_cargo_registry::{App, Config};

#[tokio::main]
async fn main() {
    let confg = Config {
        domain: "http://localhost:3000".to_string(),
    };
    let working_dir = std::env::current_dir().unwrap();
    let storage = LocalStorage::new(working_dir.join("index"), working_dir.join("crates"));

    let app = App::new(confg, storage);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.create_router()).await.unwrap();
}
