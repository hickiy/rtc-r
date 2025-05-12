mod user;
mod session;
mod signaling;
use signaling::run_websocket_server;

#[tokio::main]
async fn main() {
    // 监听 127.0.0.1:9001，可以根据需要修改端口
    if let Err(e) = run_websocket_server("127.0.0.1:8888").await {
        eprintln!("WebSocket server error: {}", e);
    }
}