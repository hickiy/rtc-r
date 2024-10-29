use tokio::net::UdpSocket;
use stun::message::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3478".parse()?;
    let socket = UdpSocket::bind(addr).await?;
    println!("TURN server listening on {}", addr);

    let mut buf = [0; 1024];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        println!("Received packet from {}", src);

        let mut msg = Message::new();
        msg.raw = buf[..len].to_vec();

        if let Err(e) = msg.decode() {
            eprintln!("Failed to decode STUN message: {}", e);
            continue;
        }

        let mut res = Message::new();
        res.build(&msg.get_setters())?;

        let res_buf = res.raw;
        socket.send_to(&res_buf, &src).await?;
    }
}