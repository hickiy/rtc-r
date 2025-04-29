use bcrypt::verify;
use std::collections::HashMap;
use jsonwebtoken::{ encode, Header, EncodingKey };
use serde::{ Serialize, Deserialize };
use chrono::{ Utc, Duration };
use std::sync::{LazyLock, Mutex};

const HASHED_FILE_PATH: &str = "./hashed.txt";
const SECRET: [u8; 6] = [115, 101, 99, 114, 101, 116];

// This struct is used to represent the USERS
static USERS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// This struct is used to represent the JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String, // subject (user ID)
  exp: usize, // expiration time (as UTC timestamp)
}

// To generate a JWT token
fn generate_jwt(username: &str, secret: &[u8]) -> String {
  let expiration = Utc::now()
    .checked_add_signed(Duration::hours(24))
    .expect("valid timestamp")
    .timestamp() as usize;

  let claims = Claims {
    sub: username.to_owned(),
    exp: expiration,
  };
  encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap()
}

// to check if the JWT token has expired
fn verify_jwt(token: &str, secret: &[u8]) -> bool {
  let token_data = jsonwebtoken::decode::<Claims>(
    token,
    &jsonwebtoken::DecodingKey::from_secret(secret),
    &jsonwebtoken::Validation::default()
  );
  match token_data {
    Ok(data) => {
      let now = Utc::now().timestamp() as usize;
      if data.claims.exp > now {
        return true;
      }
    }
    Err(_) => {}
  }
  false
}

fn is_existing_user(username: &str) -> bool {
  let users = USERS.lock().unwrap();
  users.contains_key(username)
}

fn valid_password(password: &str) -> bool {
  let hash = std::fs::read_to_string(std::path::Path::new(HASHED_FILE_PATH)).unwrap();
  verify(password, &hash).unwrap_or(false)
}

pub fn login(username: &str, password: &str) -> String {
  let mut users = USERS.lock().unwrap();
  // chick password first
  if !valid_password(password) {
    return "username or password is incorrect".to_string();
  }
  // user is'nt exist
  if !is_existing_user(username) {
    let token = generate_jwt(username, &SECRET);
    users.insert(username.to_string(), token.clone());
    return token;
  }
  // user is exist
  let mut token = users.get(username);
  // chick whether the token is valid
  if !verify_jwt(token.unwrap(), &SECRET) {
    token = generate_jwt(username, &SECRET);
    users.set(username.to_string(), token.clone());
    return token;
  }
  return token;
}

pub fn logout(username: &str, token: &str) {
  let mut users = USERS.lock().unwrap();
  // check if the token is valid
  if !verify_jwt(token, &SECRET) {
    return;
  }
  users.remove(username);
}

// to get unsrname from token
pub fn get_username_from_token(token: &str) -> String {
  let token_data = jsonwebtoken::decode::<Claims>(
    token,
    &jsonwebtoken::DecodingKey::from_secret(&SECRET),
    &jsonwebtoken::Validation::default()
  );
  match token_data {
    Ok(data) => {
      return data.claims.sub;
    }
    Err(_) => {}
  }
  "".to_string()
}

pub fn authenticate(token: &str) -> bool {
  // verify the token
  if !verify_jwt(token, &SECRET) {
    return false;
  }
  // get user username
  let username = get_username_from_token(token);
  // wather the token is valid
  if username.is_empty() {
    return false;
  }
  // check if the user is exist
  if !is_existing_user(&username) {
    return false;
  }
  true
}
