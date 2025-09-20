mod engine;
mod game;
mod session;
mod websocket;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::{get, get_service},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use session::SessionManager;
use websocket::handle_socket;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(session_manager): State<Arc<SessionManager>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, session_manager))
}

#[tokio::main]
async fn main() {
    let session_manager = Arc::new(SessionManager::new());

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .fallback_service(get_service(ServeDir::new("./static")))
        .with_state(session_manager);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
