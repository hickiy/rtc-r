#![allow(dead_code)]
use tower_http::services::ServeDir;
use axum::{
  response::IntoResponse,
  routing::{get, get_service},
  Router
};

async fn socket_handler() -> impl IntoResponse {
  "WebSocket connection established"
}

async fn login_handler() -> impl IntoResponse {
  "Login page"
}

async fn logout_handler() -> impl IntoResponse {
  "Logout page"
}

pub async fn run_web_server(addr: &str) {
  let app = Router::new()
    .route("/login", get(login_handler))
    .route("/logout", get(logout_handler))
    .route("/ws", get(socket_handler))
    .fallback_service(get_service(ServeDir::new("./public")));

  let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind address");
  println!("Listening on {}", addr);
  axum::serve(listener, app).await.expect("Failed to start server");
}
