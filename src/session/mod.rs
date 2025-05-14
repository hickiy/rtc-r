#![allow(dead_code)]
use std::sync::{ LazyLock, Mutex };
use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::Message;
use tokio::sync::mpsc::UnboundedSender;

pub struct Session {
  // 会话创建时间（Unix时间戳，单位秒）
  pub created_at: u64,
  // 最后活动时间（Unix时间戳，单位秒）
  pub last_active_at: u64,
  // 发送消息的通道
  pub tx: Arc<UnboundedSender<Message>>,
}

// 全局会话表
pub static SESSION_MAP: LazyLock<Mutex<HashMap<String, Session>>> = LazyLock::new(||
  Mutex::new(HashMap::new())
);

pub fn add_session(username: &str, tx: Arc<UnboundedSender<Message>>) -> bool {
  let session = Session {
    created_at: chrono::Utc::now().timestamp() as u64,
    last_active_at: chrono::Utc::now().timestamp() as u64,
    tx,
  };

  let mut session_store = SESSION_MAP.lock().unwrap();
  session_store.insert(username.to_string(), session);
  true
}

pub fn remove_session(username: &str) {
  let mut session_store = SESSION_MAP.lock().unwrap();
  session_store.remove(username);
}

pub fn get_tx(username: &str) -> Option<Arc<UnboundedSender<Message>>> {
  let session_store = SESSION_MAP.lock().unwrap();
  if let Some(session) = session_store.get(username) {
    Some(session.tx.clone())
  } else {
    None
  }
}
