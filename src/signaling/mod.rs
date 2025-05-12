#![allow(dead_code, unused_imports)]
use tokio::{ net::TcpListener, select };
use tokio_tungstenite::tungstenite::{ protocol::Message, handshake::server::{ Request, Response } };
use tokio_tungstenite::accept_hdr_async;
use futures_util::{ StreamExt, SinkExt };
use tokio::sync::mpsc::{ UnboundedSender, UnboundedReceiver, unbounded_channel };
use std::sync::Arc;
use crate::user::{ login, logout };
use crate::session::{ add_session, remove_session, get_session };

pub async fn run_websocket_server(addr: &str) -> tokio::io::Result<()> {
  let listener = TcpListener::bind(addr).await?;
  println!("WebSocket server listening on: {}", addr);
  loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(async move {
      let mut temp_name = String::new();
      let callback = |req: &Request, response: Response| {
        let headers = req.headers();
        let username = headers
          .get("username")
          .and_then(|v| v.to_str().ok())
          .unwrap_or("");
        let password = headers
          .get("password")
          .and_then(|v| v.to_str().ok())
          .unwrap_or("");
        let authorized = !login(username, password).is_empty();
        temp_name = username.to_string();
        if authorized {
          Ok(response)
        } else {
          Err(
            tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(
              Some("Unauthorized".into())
            )
          )
        }
      };

      let ws_stream = accept_hdr_async(stream, callback).await;
      match ws_stream {
        Ok(ws_stream) => {
          // 拆分为 Sink 和 Stream
          let (mut ws_sink, mut ws_stream) = ws_stream.split();
          // 创建 mpsc channel
          let (tx, mut rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) = unbounded_channel();
          // 注册 session，存 tx
          let session_id = add_session(&temp_name, Arc::new(tx.clone()));
          println!("New WebSocket connection");

          loop {
            select! {
              // 客户端发来的消息
              msg = ws_stream.next() => {
                match msg {
                Some(Ok(msg)) => {
                    match msg {
                        Message::Ping(payload) => {
                            // 回复 Pong
                            if let Err(e) = ws_sink.send(Message::Pong(payload)).await {
                                eprintln!("Pong send error: {}", e);
                                break;
                            }
                        }
                        Message::Text(text) => {
                            println!("Received text message: {}", text);
                            let parts: Vec<&str> = text.splitn(2, ':').collect();
                            if parts.len() == 2 {
                              let session_id = parts[0].trim();
                              let message = parts[1].trim();
                              if let Some(tx) = get_session(session_id) {
                                // 发送消息到指定的 session
                                if let Err(e) = tx.send(Message::Text(message.into())) {
                                  eprintln!("Send error: {}", e);
                                  break;
                                }
                              } else {
                                let _ = ws_sink.send(Message::Text(format!("Session {} not found", session_id))).await;
                              }
                            } else {
                              let _ = ws_sink.send(Message::Text("Invalid message format. Use 'target: message'.".into())).await;
                            }
                        }
                        Message::Close(frame) => {
                            println!("Received close: {:?}", frame);
                            remove_session(&temp_name);
                            break;
                        }
                        _ => {}
                    }
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    remove_session(&session_id);
                    break;
                }
                None => {
                    // 客户端断开连接
                    println!("Client disconnected");
                    remove_session(&session_id);
                    break;
                }
                }
              }
              // 其他线程推送过来的消息
              Some(push_msg) = rx.recv() => {
                if let Err(e) = ws_sink.send(push_msg).await { 
                  eprintln!("Send error: {}", e);
                  break;
                }
              }
            }
          }
          // 断开时移除 session
          remove_session(&session_id);
        }
        Err(e) => eprintln!("WebSocket handshake error: {}", e),
      }
    });
  }
}