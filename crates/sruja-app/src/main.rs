//! Sruja Desktop App Entry Point

use sruja_app::app::App;

fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dioxus::launch(App);
}
