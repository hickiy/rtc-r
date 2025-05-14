use axum::{
  response::IntoResponse,
  extract::{ Query, ws::{ WebSocketUpgrade, Message } },
  http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use futures_util::{ StreamExt, SinkExt };
use tokio::sync::mpsc::{ unbounded_channel };
use crate::session::{ add_session, get_tx, remove_session };

#[derive(Debug, Clone, Deserialize)]
struct Msg {
  msg_type: String,
  target: String,
  content: String,
}

pub async fn socket_handler(
  ws: WebSocketUpgrade,
  Query(params): Query<HashMap<String, String>>
) -> impl IntoResponse {
  if let Some(username) = params.get("username") {
    let username = username.clone();
    ws.on_upgrade(|socket| async move {
      let (mut ws_tx, mut ws_rx) = socket.split();
      let (tx, mut rx) = unbounded_channel::<Message>();
      add_session(&username, Arc::new(tx));
      loop {
        tokio::select! {
          Some(msg) = ws_rx.next() => {
            match msg {
              Ok(msg) => {
                match msg {
                  Message::Text(text) => {
                    let msg: Msg = serde_json::from_str(&text).unwrap();
                    println!("Received message: {:?}", msg);
                    
                  }
                  Message::Close(_) => {
                    println!("Received close message");
                    break;
                  }
                  _ => {
                    println!("Received other message type");
                    break;
                  }
                }
              }
              Err(e) => {
                println!("Error receiving message: {:?}", e);
                break;
              }
            }
          }
          Some(msg) = rx.recv() => {
            if let Err(e) = ws_tx.send(msg).await {
              println!("Error sending message: {:?}", e);
              break;
            }
          }
        }
      }
      remove_session(&username);
      println!("WebSocket connection closed for user: {}", username);
    })
  } else {
    (StatusCode::BAD_REQUEST, "Missing username").into_response()
  }
}
