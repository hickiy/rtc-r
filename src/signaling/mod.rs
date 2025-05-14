#![allow(dead_code)]
mod socket;
use self::socket::socket_handler;
use tower_http::services::ServeDir;
use std::collections::HashMap;
use crate::user::{ authenticate, login, logout };
use axum::{
  Router,
  extract::Query,
  body::Body,
  response::{ IntoResponse, Response },
  routing::{ get, get_service },
  http::{ Request, StatusCode },
  middleware::{ self, Next },
};

// auth_middleware
async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
  // check cookie，extra the token from cookie
  if let Some(cookie) = req.headers().get("Cookie") {
    if let Ok(cookie_str) = cookie.to_str() {
      let token = cookie_str
        .split(';')
        .find(|s| s.trim().starts_with("token="))
        .map(|s| s.trim().split('=').nth(1).unwrap_or(""));
      if let Some(token) = token {
        println!("Token: {}", token);
        if authenticate(token) {
          return Ok(next.run(req).await);
        } else {
          return Err(StatusCode::UNAUTHORIZED);
        }
      }
    }
  }
  Err(StatusCode::UNAUTHORIZED)
}


async fn login_handler(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
  if let (Some(username), Some(password)) = (params.get("username"), params.get("password")) {
    let token = login(username, password);
    if !token.is_empty() {
      return (StatusCode::OK, format!("Logged in user: {} with token: {}", username, token));
    } else {
      return (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string());
    }
  } else {
    (StatusCode::BAD_REQUEST, "Missing username or password".to_string())
  }
}

async fn logout_handler(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
  if let Some(username) = params.get("username") {
    logout(username);
    return (StatusCode::OK, format!("Logged out user: {}", username));
  } else {
    return (StatusCode::BAD_REQUEST, "Missing username".to_string());
  }
}

pub async fn run_web_server(addr: &str) {
  let protected_routes = Router::new()
    .route("/logout", get(logout_handler))
    .route("/ws", get(socket_handler))
    .route_layer(middleware::from_fn(auth_middleware));

  let public_routes = Router::new().route("/login", get(login_handler));

  let app = public_routes
    .merge(protected_routes)
    .fallback_service(get_service(ServeDir::new("./public")));

  let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind address");
  println!("Listening on {}", addr);
  axum::serve(listener, app).await.expect("Failed to start server");
}
