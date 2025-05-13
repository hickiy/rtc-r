#![allow(dead_code)]
mod store;
use self::store::{ add_user, remove_user, is_existing_user, get_user_token };
use bcrypt::verify;
use jsonwebtoken::{ encode, Header, EncodingKey };
use serde::{ Serialize, Deserialize };
use chrono::{ Utc, Duration };

const HASHED_FILE_PATH: &str = "./public/hashed.txt";
const SECRET: [u8; 6] = [115, 101, 99, 114, 101, 116];

// This struct is used to represent the JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String, // subject (user ID)
  exp: usize, // expiration time (as UTC timestamp)
}

// check if the password is valid
fn valid_password(password: &str) -> bool {
  let hash = match std::fs::read_to_string(std::path::Path::new(HASHED_FILE_PATH)) {
    Ok(h) => h,
    Err(e) => {
      eprintln!("读取哈希文件失败: {}", e);
      return false;
    }
  };
  verify(password, &hash).unwrap_or(false)
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

// to check if the JWT token has expired and it has username and username is exist
pub fn authenticate(token: &str, secret: &[u8]) -> bool {
  let token_data = jsonwebtoken::decode::<Claims>(
    token,
    &jsonwebtoken::DecodingKey::from_secret(secret),
    &jsonwebtoken::Validation::default()
  );
  match token_data {
    Ok(data) => {
      let now = Utc::now().timestamp() as usize;
      let username = data.claims.sub;
      if data.claims.exp < now || username.is_empty() || !is_existing_user(&username) {
        return false;
      }
      true;
    }
    Err(_) => {}
  }
  false
}

pub fn login(username: &str, password: &str) -> String {
  let mut token = "".to_string();
  // chick password first
  if !valid_password(password) {
    return token;
  }
  // user is'nt exist
  if !is_existing_user(&username) {
    token = generate_jwt(&username, &SECRET);
    add_user(username.to_string(), token.clone());
  } else {
    // user is exist
    token = get_user_token(&username).unwrap();
  }
  token
}

pub fn logout(username: &str, token: &str) {
  // check if the token is valid
  if !authenticate(token, &SECRET) {
    return;
  }
  remove_user(username);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_password_test() {
    let password = "245786";
    assert!(valid_password(password));
    assert!(!valid_password("wrong_password"));
  }

  #[test]
  fn generate_jwt_test() {
    let username = "test_user";
    let token = generate_jwt(username, &SECRET);
    println!("Generated JWT: {}", token);
    let token_data = jsonwebtoken::decode::<Claims>(
      &token,
      &jsonwebtoken::DecodingKey::from_secret(&SECRET),
      &jsonwebtoken::Validation::default()
    );
    match token_data {
      Ok(data) => {
        println!("Decoded JWT: {:?}", data.claims);
        let now = Utc::now().timestamp() as usize;
        if data.claims.exp > now {
          println!("Token is valid");
        } else {
          println!("Token has expired");
        }
      }
      Err(e) => {
        println!("Failed to decode JWT: {}", e);
      }
    }
  }

  #[test]
  fn authenticate_test() {
    let username = "test_user";
    let password = "245786";
    let token = login(username, password);
    assert!(authenticate(&token, &SECRET));
  }

  #[test]
  fn login_test() {
    let username = "test_user";
    let password = "245786";
    let token = login(username, password);
    println!("Login token: {}", token);
    assert!(!token.is_empty());
  }

  #[test]
  fn logout_test() {
    let username = "test_user";
    let password = "245786";
    let token = login(username, password);
    println!("Login token: {}", token);
    logout(username, &token);
    assert!(!is_existing_user(username));
  }
}
