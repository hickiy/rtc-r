use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use warp::ws::Message;
use warp::Filter;
use warp::filters::ws::WebSocket;

type Clients = HashMap<String, WebSocket>;

#[derive(Deserialize, Serialize)]
struct SignalMessage {
    to: String,
    data: String,
    message_type: String, // 新增字段
}

async fn handle_connection(ws: warp::ws::WebSocket, clients: Clients) {
    let (client_ws_sender, mut client_ws_receiver) = ws.split();
    let client_id = Uuid::new_v4().to_string();
    clients.insert(client_id.clone(), ws);

    while let Some(result) = client_ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    let signal_message: SignalMessage = serde_json::from_str(text).unwrap();
                    match signal_message.message_type.as_str() {
                        "signal" => {
                            if let Some(receiver) = clients.get_mut(&signal_message.to)
                            {
                                let _ = receiver
                                    .send(Message::text(signal_message.data.clone()))
                                    .await;
                            }
                        }
                        "list_clients" => {
                            let client_list: Vec<String> =
                                clients.keys().cloned().collect();
                            let response = serde_json::to_string(&client_list).unwrap();
                            let _ = clients
                                .get_mut(&client_id)
                                .unwrap()
                                .send(Message::text(response))
                                .await;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("websocket error: {}", e);
                break;
            }
        }
    }

    clients.remove(&client_id);
}

#[tokio::main]
async fn main() {
    let clients: Clients = HashMap::new();
    let signal_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| ws.on_upgrade(move |socket| handle_connection(socket, clients)));
    warp::serve(signal_route).run(([127, 0, 0, 1], 3030)).await;
}
