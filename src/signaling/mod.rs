#![allow(dead_code)]
pub mod socket;
use self::socket::socket_handler;
use crate::user::{authenticate, login, logout};
use axum::http::{HeaderMap, HeaderValue};
use axum::{
    Router,
    body::Body,
    extract::Query,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, get_service},
};
use std::collections::HashMap;
use tower_http::services::ServeDir;

// auth_middleware
async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // check cookie，extra the token from cookie
    let cookies = req
        .headers()
        .get("Cookie")
        .unwrap_or(&HeaderValue::from_static(""))
        .clone();

    let token = cookies
        .to_str()
        .unwrap()
        .split(';')
        .find(|s| s.trim().starts_with("token="))
        .map(|s| s.trim().split('=').nth(1).unwrap())
        .unwrap();
    if authenticate(token) {
        // 将 token 添加到请求的扩展中，传递给后续处理函数
        req.extensions_mut().insert(token.to_string());
        return Ok(next.run(req).await);
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}

async fn login_handler(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    if let (Some(username), Some(password)) = (params.get("username"), params.get("password")) {
        let token = login(username, password);
        if !token.is_empty() {
            headers.insert(
                axum::http::header::SET_COOKIE,
                HeaderValue::from_str(&format!("token={}; HttpOnly; Path=/", token)).unwrap(),
            );
            return (
                StatusCode::OK,
                headers,
                token,
            );
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                headers,
                "Invalid username or password".to_string(),
            );
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            headers,
            "Missing username or password".to_string(),
        )
    }
}

async fn logout_handler(req: Request<Body>) -> impl IntoResponse {
    if let Some(token) = req.extensions().get::<String>() {
        logout(token);
        return (StatusCode::OK, format!("Logged out user: {}", token));
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

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");
    println!("Listening on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
