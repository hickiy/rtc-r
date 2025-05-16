use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use turn_server::api::start_server;
use turn_server::config::{Auth, Config, Interface, Transport, Turn};

async fn run_turn_server(addr: String) {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3478);
    let interfase: Interface = Interface {
        transport: Transport::UDP,
        bind: socket_addr,
        external: socket_addr,
    };
    let turn: Turn = Turn {
        realm: "localhost".to_string(),
        interfaces: vec![interfase],
    };
    let auth = Auth {
        static_credentials: HashMap::with_capacity([("yanyun", "245786")]),
        static_auth_secret: None,
    };
}
