use rtc_r::signaling::run_web_server;

#[tokio::main]
async fn main() {
  run_web_server("127.0.0.1:8888").await;
}
