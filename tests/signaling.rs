use rtc_r::signaling::run_web_server;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

#[tokio::test]
async fn text_run_web_server() {
    // test the web server by connecting to it
    tokio::spawn(async {
        // start the web server
        run_web_server("127.0.0.1:8888").await;
    });
    // test the websocket connection
    tokio::spawn(async {
        let client = reqwest::Client::new();
        let resp = client
            .get("http://127.0.0.1:8888/login?username=alice&password=245786")
            .send()
            .await
            .expect("Failed to login");
        assert!(resp.status().is_success(), "Failed to get token");
        let token = resp.text().await.expect("Failed to get token");
        println!("alice: {}", token);
        let mut req = "ws://127.0.0.1:8888/ws".into_client_request().unwrap();
        req.headers_mut()
            .insert("Cookie", format!("token={}", token).parse().unwrap());
        let (user1, _) = tokio_tungstenite::connect_async(req)
            .await
            .expect("Failed to connect to WebSocket");
        let (_, mut rx) = user1.split();
        loop {
            let msg = rx.next().await.expect("Failed to receive message");
            match msg {
                Ok(msg) => {
                    if let tokio_tungstenite::tungstenite::protocol::Message::Text(text) = msg {
                        println!("alice Received message: {}", text);
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                }
            }
        }
    });

    tokio::spawn(async {
        let client = reqwest::Client::new();
        let resp2 = client
            .get("http://127.0.0.1:8888/login?username=bob&password=245786")
            .send()
            .await
            .expect("Failed to login");
        assert!(resp2.status().is_success(), "Failed to get token");
        let token2: String = resp2.text().await.expect("Failed to get token");
        println!("bob: {}", token2);
        let mut req2 = "ws://127.0.0.1:8888/ws".into_client_request().unwrap();
        req2.headers_mut()
            .insert("Cookie", format!("token={}", token2).parse().unwrap());
        let (user2, _) = tokio_tungstenite::connect_async(req2)
            .await
            .expect("Failed to connect to WebSocket");
        let json: serde_json::Value = serde_json::json!({
            "msg_type": "Offer",
            "target": "alice",
            "sdp": "sdp-offer",
        });
        let bytes = tokio_tungstenite::tungstenite::Utf8Bytes::from(json.to_string());
        let (mut tx, mut rx) = user2.split();
        tx.send(tokio_tungstenite::tungstenite::protocol::Message::Text(
            bytes,
        ))
        .await
        .expect("Failed to send message");
        loop {
            let _ = rx.next().await.expect("Failed to receive message");
        }
    });
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
}
