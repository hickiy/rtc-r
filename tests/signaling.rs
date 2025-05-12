use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use url::Url;
use rtc_r::signaling::run_websocket_server;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_run_websocket_server() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
      // 启动 WebSocket 服务器
      let addr = "127.0.0.1:9001";
      let server = tokio::spawn(async move {
        run_websocket_server(addr).await.unwrap();
      });

      // 等待服务器启动
      tokio::time::sleep(std::time::Duration::from_millis(200)).await;

      // 客户端连接
      let mut url = Url::parse(&format!("ws://{}", addr)).unwrap();
      url.query_pairs_mut().append_pair("username", "yanyun").append_pair("password", "245786");
      let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

      // 简单断言连接成功
      assert!(ws_stream.close(None).await.is_ok());

      // 关闭服务器
      drop(ws_stream);
      // 结束测试
      server.abort();
    });
  }
}
