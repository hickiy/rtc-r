use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use turn::auth::{AuthHandler, LongTermAuthHandler};
use turn::server::{TurnServer, TurnServerConfig};
use turn::stun::server::{StunServer, StunServerConfig};

struct SimpleAuth {
  username: String,
  password: String,
}

#[async_trait::async_trait]
impl AuthHandler for SimpleAuth {
  async fn authenticate(&self, username: &str, realm: &str, _src_addr: &SocketAddr) -> Option<String> {
    if username == self.username {
      Some(self.password.clone())
    } else {
      None
    }
  }
}

pub async fn run_stun_turn_server(bind_addr: &str, username: &str, password: &str) -> anyhow::Result<()> {
  let socket = UdpSocket::bind(bind_addr).await?;
  let socket = Arc::new(socket);

  // STUN server config
  let stun_config = StunServerConfig {
    socket: socket.clone(),
  };
  let stun_server = StunServer::new(stun_config);

  // TURN server config
  let auth = Arc::new(SimpleAuth {
    username: username.to_string(),
    password: password.to_string(),
  });
  let turn_config = TurnServerConfig {
    socket: socket.clone(),
    auth_handler: Arc::new(LongTermAuthHandler::new(auth, "example.org".to_string())),
    realm: "example.org".to_string(),
    ..Default::default()
  };
  let turn_server = TurnServer::new(turn_config);

  // Run both servers concurrently
  tokio::try_join!(
    stun_server.run(),
    turn_server.run(),
  )?;

  Ok(())
}

// Example main
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Listen on all interfaces, port 3478 (default STUN/TURN port)
  run_stun_turn_server("0.0.0.0:3478", "user1", "pass1").await
}