use std::fs::{self, File};
use std::io::Write;
use std::sync::Mutex;
use bcrypt::{hash, DEFAULT_COST};

// filepath: e:\learn\rtc-r\tests\user\auto.rs

#[cfg(test)]
mod tests {
  use super::*;

  // Mock the users HashMap as a static Mutex for thread safety in tests
  lazy_static::lazy_static! {
    static ref USERS: Mutex<std::collections::HashMap<String, String>> = Mutex::new(std::collections::HashMap::new());
  }

  // Helper to write a hashed password to the file
  fn setup_hashed_file(password: &str) -> String {
    let hashed = hash(password, DEFAULT_COST).unwrap();
    let path = "./hashed.txt";
    let mut file = File::create(path).unwrap();
    writeln!(file, "{}", hashed).unwrap();
    hashed
  }

  // Clean up the hashed file after test
  fn cleanup_hashed_file() {
    let _ = fs::remove_file("./hashed.txt");
  }

  #[test]
  fn test_valid_password() {
    let password = "testpass";
    setup_hashed_file(password);
    assert!(crate::user::auth::valid_password(password));
    assert!(!crate::user::auth::valid_password("wrongpass"));
    cleanup_hashed_file();
  }

  #[test]
  fn test_login_and_authenticate() {
    let username = "testuser";
    let password = "testpass";
    setup_hashed_file(password);

    // Login should return a token
    let token = crate::user::auth::login(username, password);
    assert!(!token.is_empty());
    // Authenticate should return the username
    let auth_user = crate::user::auth::authenticate(&token);
    assert_eq!(auth_user, username);

    cleanup_hashed_file();
  }

  #[test]
  fn test_login_with_wrong_password() {
    let username = "testuser2";
    let password = "testpass2";
    setup_hashed_file(password);

    let result = crate::user::auth::login(username, "wrongpass");
    assert_eq!(result, "username or password is incorrect");

    cleanup_hashed_file();
  }

  #[test]
  fn test_logout() {
    let username = "testuser3";
    let password = "testpass3";
    setup_hashed_file(password);

    let token = crate::user::auth::login(username, password);
    assert!(!token.is_empty());

    // Logout should remove the user
    crate::user::auth::logout(username, &token);
    // Authenticate should now fail
    let auth_user = crate::user::auth::authenticate(&token);
    assert_ne!(auth_user, username);

    cleanup_hashed_file();
  }

  #[test]
  fn test_authenticate_invalid_token() {
    let invalid_token = "invalid.token.value";
    let result = crate::user::auth::authenticate(invalid_token);
    assert_eq!(result, "token is invalid");
  }
}