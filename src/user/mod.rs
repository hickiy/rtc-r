#![allow(dead_code)]
mod store;
use self::store::{add_user, get_user_token, is_existing_user, remove_user};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

const HASHED_FILE_PATH: &str = "./public/hashed.txt";
const SECRET: [u8; 6] = [115, 101, 99, 114, 101, 116];

// This struct is used to represent the JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user ID)
    exp: usize,  // expiration time (as UTC timestamp)
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
fn generate_jwt(username: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&SECRET),
    )
    .unwrap()
}

pub fn get_username_from_token(token: &str) -> Option<String> {
    if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&SECRET),
        &jsonwebtoken::Validation::default(),
    ) {
        Some(token_data.claims.sub)
    } else {
        None
    }
}

pub fn login(username: &str, password: &str) -> String {
    let mut token = "".to_string();
    // chick password first
    if !valid_password(password) {
        return token;
    }
    // user is'nt exist
    if !is_existing_user(&username) {
        token = generate_jwt(&username);
        add_user(username.to_string(), token.clone());
    } else {
        // user is exist
        token = get_user_token(&username).unwrap();
    }
    token
}

// to check if the JWT token has expired and it has username and username is exist
pub fn authenticate(token: &str) -> bool {
    if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&SECRET),
        &jsonwebtoken::Validation::default(),
    ) {
        let now = Utc::now().timestamp() as usize;
        let username = token_data.claims.sub;
        if token_data.claims.exp < now || username.is_empty() || !is_existing_user(&username) {
            return false;
        }
        true
    } else {
        false
    }
}

pub fn logout(token: &str) {
    remove_user(&get_username_from_token(token).unwrap());
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
        let token = generate_jwt(username);
        println!("Generated JWT: {}", token);
        let token_data = jsonwebtoken::decode::<Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(&SECRET),
            &jsonwebtoken::Validation::default(),
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
    fn get_username_from_token_test() {
        let username = "test_user";
        let token = generate_jwt(username);
        let decoded_username = get_username_from_token(&token);
        assert_eq!(decoded_username, Some(username.to_string()));
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
    fn authenticate_test() {
        let username = "test_user";
        let password = "245786";
        let token = login(username, password);
        assert!(authenticate(&token));
    }

    #[test]
    fn logout_test() {
        let username = "test_user";
        let password = "245786";
        login(username, password);
        assert!(is_existing_user(username));
        logout(username);
        assert!(!is_existing_user(username));
    }
}
