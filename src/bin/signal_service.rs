use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
struct SignalMessage {
    from: String,
    to: String,
    data: String,
}

type Clients = Arc<Mutex<HashMap<String, warp::ws::WsSender>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let clients_filter = warp::any().map(move || clients.clone());

    let signal_route = warp::path("signal")
        .and(warp::ws())
        .and(clients_filter)
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    warp::serve(signal_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_connection(ws: warp::ws::WebSocket, clients: Clients) {
    let (client_ws_sender, mut client_ws_receiver) = ws.split();
    let client_id = uuid::Uuid::new_v4().to_string();

    clients.lock().unwrap().insert(client_id.clone(), client_ws_sender);

    while let Some(result) = client_ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    let signal_message: SignalMessage = serde_json::from_str(text).unwrap();
                    if let Some(receiver) = clients.lock().unwrap().get(&signal_message.to) {
                        let _ = receiver.send(warp::ws::Message::text(signal_message.data.clone())).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("websocket error: {}", e);
                break;
            }
        }
    }

    clients.lock().unwrap().remove(&client_id);
}