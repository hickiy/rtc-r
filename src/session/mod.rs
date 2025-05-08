#![allow(dead_code)]
use std::sync::{ LazyLock, Mutex };
use std::collections::HashMap;
use uuid::Uuid;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

pub struct Session {
  // 会话唯一标识符
  pub id: String,
  // 用户名或用户ID
  pub username: String,
  // 会话创建时间（Unix时间戳，单位秒）
  pub created_at: u64,
  // 最后活动时间（Unix时间戳，单位秒）
  pub last_active_at: u64,
  // 套接字
  pub socket: WebSocketStream<TcpStream>,
}

// SessionManager
static SESSION_STORE: LazyLock<Mutex<HashMap<String, Session>>> = LazyLock::new(||
  Mutex::new(HashMap::new())
);

pub fn add_session(username: &str, socket: WebSocketStream<TcpStream>) -> String {
  let id = Uuid::new_v4().to_string();
  let session = Session {
    id: id.clone(),
    username: username.to_string(),
    created_at: chrono::Utc::now().timestamp() as u64,
    last_active_at: chrono::Utc::now().timestamp() as u64,
    socket,
  };

  let mut session_store = SESSION_STORE.lock().unwrap();
  session_store.insert(id.clone(), session);
  id
}

pub fn remove_session(id: &str) {
  let mut session_store = SESSION_STORE.lock().unwrap();
  session_store.remove(id);
}

pub fn get_session(id: &str) -> Option<String> {
  let session_store = SESSION_STORE.lock().unwrap();
  if let Some(session) = session_store.get(id) {
    Some(session.id.clone())
  } else {
    None
  }
}
