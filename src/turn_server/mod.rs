use std::net::SocketAddrV4;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use turn_server::{
    config::{Api, Auth, Config, Interface, Log, Transport, Turn},
    startup,
};

pub async fn create_turn_server(addr: &str) {
    let bind = SocketAddr::V4(addr.parse::<SocketAddrV4>().unwrap());
    let mut static_credentials = HashMap::with_capacity(1);
    static_credentials.insert("yanyun".to_string(), "245786".to_string());
    let auth = Auth {
        static_credentials,
        static_auth_secret: None,
    };
    println!("TURN server started on {}", addr);
    startup(Arc::new(Config {
        log: Log::default(),
        turn: Turn {
            realm: "localhost".to_string(),
            interfaces: vec![Interface {
                transport: Transport::UDP,
                external: bind,
                bind,
            }],
        },
        auth,
        api: Api::default(),
    }))
    .await
    .expect("Failed to start TURN server");
}
