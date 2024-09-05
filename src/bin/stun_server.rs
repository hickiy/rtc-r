use rfc5766_turn_server::auth::AuthHandler;
use rfc5766_turn_server::relay::RelayServer;
use std::net::SocketAddr;

struct SimpleAuthHandler;

impl AuthHandler for SimpleAuthHandler {
    fn check_credentials(&self, _username: &str, _realm: &str, _nonce: &str, _password: &str) -> bool {
        // 在这里实现你的认证逻辑
        true
    }
}

fn main() {
    let addr: SocketAddr = "0.0.0.0:3478".parse().expect("Invalid address");
    let auth_handler = SimpleAuthHandler;
    let relay_server = RelayServer::new(addr, auth_handler);

    relay_server.start().expect("Failed to start TURN server");
}