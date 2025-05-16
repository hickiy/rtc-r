use crate::session::{add_session, get_tx, get_tx_all, get_users, remove_session};
use crate::user::get_username_from_token;
use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum Msg {
    Offer {
        name: String,
        target: String,
        sdp: String,
    },
    Answer {
        name: String,
        target: String,
        sdp: String,
    },
    Candidate {
        target: String,
        candidate: String,
    },
    HangUp {
        target: String,
    },
    UserList {
        users: Vec<String>,
    },
    UserJoin {
        username: String,
    },
    UserLeave {
        username: String,
    },
}

pub async fn socket_handler(
    ws: WebSocketUpgrade,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    if let Some(token) = req.extensions().get::<String>() {
        let username = get_username_from_token(token).unwrap();
        println!("WebSocket connection established for user: {}", username);
        ws.on_upgrade(|socket| async move {
            let (mut ws_tx, mut ws_rx) = socket.split();
            let (tx, mut rx) = unbounded_channel::<Message>();
            // get all users
            let users = get_users();
            // send the user list to the new user
            let user_list = serde_json::to_string(&Msg::UserList { users }).unwrap();
            ws_tx.send(Message::Text(user_list.into())).await.unwrap();
            // get all users' senders
            let mut txs = get_tx_all();
            // send the user join message to all users
            let user_join = serde_json::to_string(&Msg::UserJoin {
                username: username.clone(),
            })
            .unwrap();
            txs.iter().for_each(|target_tx| {
                target_tx
                    .send(Message::Text(user_join.clone().into()))
                    .unwrap();
            });
            // save the sender to the session map
            add_session(&username, Arc::new(tx));
            loop {
                tokio::select! {
                  Some(msg) = ws_rx.next() => {
                    match msg {
                      Ok(msg) => {
                        match msg {
                          Message::Text(text) => {
                            let body: Msg = serde_json::from_str(&text).unwrap();
                            let handle_relay = |target| {
                                if let Some(target_tx) = get_tx(target) {
                                    target_tx.send(Message::Text(text)).unwrap();
                                } else {
                                    println!("Target user not found");
                                }
                            };
                            match body {
                              Msg::Offer { target, .. }
                              | Msg::Answer { target, .. }
                              | Msg::Candidate { target, .. }
                              | Msg::HangUp {target, ..} => {
                                  handle_relay(&target);
                              }
                              _ => {
                                  println!("Unknown message type");
                              }
                            }
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
            // remove the user from the session map
            remove_session(&username);
            // update txs
            txs = get_tx_all();
            // crate the user leave message
            let user_leave = serde_json::to_string(&Msg::UserLeave {
                username: username.clone(),
            })
            .unwrap();
            // send the user leave message to all users
            txs.iter().for_each(|target_tx| {
                target_tx
                    .send(Message::Text(user_leave.clone().into()))
                    .unwrap();
            });
            println!("WebSocket connection closed for user: {}", username);
        })
    } else {
        (StatusCode::BAD_REQUEST, "Missing username").into_response()
    }
}
