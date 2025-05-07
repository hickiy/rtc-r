use std::collections::HashMap;
use std::sync::{ LazyLock, Mutex };

// This struct is used to represent the USERS
static USERS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(||
  Mutex::new(HashMap::new())
);

pub fn add_user(username: String, token: String) {
  let mut users = USERS.lock().unwrap();
  users.insert(username, token);
}
pub fn remove_user(username: &str) {
  let mut users = USERS.lock().unwrap();
  users.remove(username);
}

pub fn is_existing_user(username: &str) -> bool {
  let users = USERS.lock().unwrap();
  users.contains_key(username)
}

pub fn get_user_token(username: &str) -> Option<String> {
  let users = USERS.lock().unwrap();
  users.get(username).cloned()
}
