use bcrypt::verify;
use std::collections::HashMap;
use jsonwebtoken::{ encode, Header, EncodingKey };
use serde::{ Serialize, Deserialize };
use chrono::{ Utc, Duration };

const HASHED_FILE_PATH: &str = "./hashed.txt";
const SECRET: [u8; 6] = [115, 101, 99, 114, 101, 116];

// This struct is used to represent the users
static mut users = HashMap::new();

// This struct is used to represent the JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String, // subject (user ID)
  exp: usize, // expiration time (as UTC timestamp)
}

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

fn verify_jwt(token: &str, secret: &[u8]) -> String {
  let token_data = jsonwebtoken::decode::<Claims>(
    token,
    &jsonwebtoken::DecodingKey::from_secret(secret),
    &jsonwebtoken::Validation::default()
  );
  // chick whether the token has expired
  if let Ok(data) = token_data {
    let now = Utc::now().timestamp() as usize;
    if data.claims.exp > now && is_existing_user(&data.claims.sub) {
      return data.claims.sub.clone();
    } else {
      return "".to_string();
    }
  } else {
    return "".to_string();
  }
}

fn is_existing_user(username: &str) -> bool {
  users.contains_key(username)
}

fn valid_password(password: &str) -> bool {
  let hash = std::fs::read_to_string(std::path::Path::new(HASHED_FILE_PATH)).unwrap();
  verify(password, hash).unwrap_or(false)
}

pub fn login(username: &str, password: &str) -> String {
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
  // check if the token is valid
  if !verify_jwt(token, &SECRET) {
    return;
  }
  users.remove(username);
}

pub fn authenticate(token: &str) -> String {
  // get user username
  let username = verify_jwt(token, &SECRET);
  // wather the token is valid
  if username.is_empty() {
    return "token is invalid".to_string();
  }
  // check if the user is exist
  if !is_existing_user(&username) {
    return "user is not exist".to_string();
  }
  return username.to_string();
}
