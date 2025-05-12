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

fn valid_password(password: &str) -> bool {
  let hash = match std::fs::read_to_string(std::path::Path::new(HASHED_FILE_PATH)) {
    Ok(h) => h,
    Err(e) => {
      eprintln!("读取哈希文件失败: {}", e); // 可以根据需要选择是否打印错误
      return false;
    }
  };
  verify(password, &hash).unwrap_or(false)
}

pub fn login(username: &str, password: &str) -> String {
  // chick password first
  if !valid_password(password) {
    return "".to_string();
  }
  // user is'nt exist
  if !is_existing_user(&username) {
    let token = generate_jwt(&username, &SECRET);
    add_user(username.to_string(), token.clone());
    return token;
  }
  // user is exist
  let token = get_user_token(&username).unwrap();
  // chick whether the token is valid
  if !verify_jwt(&token, &SECRET) {
    let token = generate_jwt(&username, &SECRET);
    add_user(username.to_string(), token.clone());
    return token;
  }
  return token;
}

pub fn logout(username: &str, token: &str) {
  // check if the token is valid
  if !verify_jwt(token, &SECRET) {
    return;
  }
  remove_user(username);
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
  fn is_existing_user_test() {
    let username = "test_user";
    let token = generate_jwt(username, &SECRET);
    add_user(username.to_string(), token.clone());
    assert!(is_existing_user(username));
    assert!(!is_existing_user("non_existent_user"));
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
  fn verify_jwt_test() {
    let username = "test_user";
    let token = generate_jwt(username, &SECRET);
    println!("Generated JWT: {}", token);
    let is_valid = verify_jwt(&token, &SECRET);
    if is_valid {
      println!("Token is valid");
    } else {
      println!("Token has expired");
    }
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

  #[test]
  fn get_username_from_token_test() {
    let username = "test_user";
    let token = generate_jwt(username, &SECRET);
    let extracted_username = get_username_from_token(&token);
    assert_eq!(extracted_username, username);
  }

  #[test]
  fn authenticate_test() {
    let username = "test_user";
    let password = "245786";
    let token = login(username, password);
    assert!(authenticate(&token));
  }
}
