use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

mod db_worker;
mod fhir;
mod models;
mod serial;
mod state;
mod websocket;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize Logging
    tracing_subscriber::fmt::init();
    println!("Server initializing...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = db::get_db_pool(&database_url)
        .await
        .expect("Failed to connect to database");
    println!("Database connection established.");

    //  Redis Connection
    let redis_client = redis::Client::open("redis://127.0.0.1:6379/").expect("Invalid Redis URL");
    println!("Redis client created.");

    //  Create the Broadcast Channel
    let (tx, _rx) = broadcast::channel(100);

    //  Start Background Tasks/Data Pipeline

    // Serial Listener Input - Pass Redis client for caching
    serial::spawn_serial_listener(tx.clone(), redis_client.clone());

    // DB Worker/Storage
    db_worker::spawn_db_worker(pool.clone(), tx.subscribe()).await;

    //  Build the Application State
    let app_state = AppState {
        db: pool,
        tx,
        redis: redis_client,
    };

    //  Define Routes
    let app = Router::new()
        // Real-Time WebSocket Stream
        .route("/ws", get(websocket::ws_handler))
        // FHIR Compliance API
        .route(
            "/api/fhir/observation/latest",
            get(fhir::get_latest_observation),
        )
        // Health Check
        .route("/health", get(|| async { "Status: Healthy" }))
        // Frontend Hosting
        .nest_service(
            "/",
            ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend")),
        )
        .with_state(app_state);

    // Start the Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Sedentary Tracker listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
