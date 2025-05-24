use rtc_r::signaling::run_web_server;
use rtc_r::turn_server::create_turn_server;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        run_web_server("0.0.0.0:8888").await;
    });
    create_turn_server("0.0.0.0:3478").await;
}
