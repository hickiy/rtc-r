#![allow(dead_code)]
use std::sync::{ LazyLock, Mutex };
use tokio::sync::mpsc::UnboundedSender;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;

type Tx = Arc<UnboundedSender<tokio_tungstenite::tungstenite::Message>>;

pub struct Session {
  // 会话唯一标识符
  pub id: String,
  // 用户名或用户ID
  pub username: String,
  // 会话创建时间（Unix时间戳，单位秒）
  pub created_at: u64,
  // 最后活动时间（Unix时间戳，单位秒）
  pub last_active_at: u64,
  // 发送消息的通道
  pub tx: Tx
}

// 全局会话表
pub static SESSION_MAP: LazyLock<Mutex<HashMap<String, Session>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn add_session(username: &str, tx:  Tx) -> String {
  let id = Uuid::new_v4().to_string();
  let session = Session {
    id: id.clone(),
    username: username.to_string(),
    created_at: chrono::Utc::now().timestamp() as u64,
    last_active_at: chrono::Utc::now().timestamp() as u64,
    tx,
  };

  let mut session_store = SESSION_MAP.lock().unwrap();
  session_store.insert(id.clone(), session);
  id
}

pub fn remove_session(id: &str) {
  let mut session_store = SESSION_MAP.lock().unwrap();
  session_store.remove(id);
}

pub fn get_session(id: &str) -> Option<String> {
  let session_store = SESSION_MAP.lock().unwrap();
  if let Some(session) = session_store.get(id) {
    Some(session.id.clone())
  } else {
    None
  }
}
