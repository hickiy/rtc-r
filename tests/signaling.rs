use rtc_r::signaling::run_web_server;

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn text_run_web_server() {
    tokio::spawn(async {
      // start the web server
      run_web_server("127.0.0.1:8888").await;
      // test the web server by connecting to it
      let client = reqwest::Client::new();
      let resp = client
        .get("http://127.0.0.1:8888/index.html")
        .send().await
        .expect("Failed to send request");
      assert!(resp.status().is_success(), "Request to index.html failed");
    });
  }
}
