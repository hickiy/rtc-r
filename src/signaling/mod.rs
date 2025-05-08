use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{protocol::Message, handshake::server::{Request, Response}};
use tokio_tungstenite::accept_hdr_async;
use futures_util::{StreamExt, SinkExt};

const PASSWORD: &str = "your_password";

pub async fn run_websocket_server(addr: &str) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);
    
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            // 握手认证回调
            let callback = |req: &Request, mut response: Response| {
                // 例如用自定义 Header: Authorization
                let authorized = req.headers()
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v == PASSWORD)
                    .unwrap_or(false);

                if authorized {
                    Ok(response)
                } else {
                    // 拒绝握手
                    Err(tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(Some("Unauthorized".into())))
                }
            };

            let ws_stream = accept_hdr_async(stream, callback).await;
            match ws_stream {
                Ok(mut ws_stream) => {
                    println!("New WebSocket connection");
                    while let Some(msg) = ws_stream.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("Received: {}", text);
                                if let Err(e) = ws_stream.send(Message::Text(text)).await {
                                    eprintln!("Send error: {}", e);
                                    break;
                                }
                            }
                            Ok(Message::Binary(bin)) => {
                                if let Err(e) = ws_stream.send(Message::Binary(bin)).await {
                                    eprintln!("Send error: {}", e);
                                    break;
                                }
                            }
                            Ok(Message::Close(_)) => {
                                println!("Connection closed");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => eprintln!("WebSocket handshake error: {}", e),
            }
        });
    }
}