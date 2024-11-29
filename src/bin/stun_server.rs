use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3478".parse::<SocketAddr>()?;
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    let (tx, mut rx) = mpsc::channel(32);
    let cloned_socket = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = cloned_socket.recv_from(&mut buf).await.unwrap();
            let data = buf[..len].to_vec();
            tx.send((data, addr)).await.unwrap();
        }
    });

    while let Some((data, addr)) = rx.recv().await {
        // 解析并处理STUN/TURN消息
        if is_stun_message(&data) {
            handle_stun_message(&data, &socket, addr).await;
        } else if is_turn_message(&data) {
            handle_turn_message(&data, &socket, addr).await;
        }
    }

    Ok(())
}

fn is_stun_message(data: &[u8]) -> bool {
    // 检查数据是否为STUN消息
    // 例如，检查STUN消息的魔法Cookie
    data.len() > 0 && data[0] == 0x00
}

fn is_turn_message(data: &[u8]) -> bool {
    // 检查数据是否为TURN消息
    // TURN消息也是STUN消息的一种，因此可以复用STUN的检查逻辑
    is_stun_message(data)
}

async fn handle_stun_message(data: &[u8], socket: &UdpSocket, addr: SocketAddr) {
    // 处理STUN消息
    println!("Handling STUN message from {}: {:?}", addr, data);
    // 这里需要实现STUN消息的处理逻辑
}

async fn handle_turn_message(data: &[u8], socket: &UdpSocket, addr: SocketAddr) {
    // 处理TURN消息
    println!("Handling TURN message from {}: {:?}", addr, data);
    // 这里需要实现TURN消息的处理逻辑
}