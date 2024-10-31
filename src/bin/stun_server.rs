use stun::message::*;
use stun::xoraddr::XorMappedAddress;
use tokio::net::UdpSocket;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = "0.0.0.0:3478".parse()?;
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    println!("STUN server listening on {}", addr);

    let mut buf = [0; 1024];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        println!("Received packet from {}", src);

        let socket = Arc::clone(&socket);
        tokio::task::spawn(async move {
            let mut msg = Message::new();
            msg.raw = buf[..len].to_vec();

            if let Err(e) = msg.decode() {
                eprintln!("Failed to decode STUN message: {}", e);
                return;
            }

            let mut res = Message::new();
            let xor_addr = XorMappedAddress {
                ip: src.ip(),
                port: src.port(),
            };
            res.build(&[Box::new(xor_addr)]).unwrap();

            let res_buf = res.raw;
            socket.send_to(&res_buf, &src).await.unwrap();
        });
    }
}
