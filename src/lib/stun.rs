use stun::message::*;
use stun::xoraddr::XorMappedAddress;
use tokio::net::{ UdpSocket, SocketAddr };

pub async fn handle_stun_message(data: &[u8], socket: &UdpSocket, addr: SocketAddr) {
  let mut msg = Message::new();
  msg.raw = data.to_vec();

  if let Err(e) = msg.decode() {
    eprintln!("Failed to decode STUN message: {}", e);
    return;
  }

  let mut res = Message::new();
  let xor_addr = XorMappedAddress {
    ip: addr.ip(),
    port: addr.port(),
  };
  res.build(&[Box::new(xor_addr)]).unwrap();

  let res_buf = res.raw;
  socket.send_to(&res_buf, &addr).await.unwrap();
}
