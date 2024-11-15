use dotenv::dotenv;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    time::Duration,
};

static READ_TIMEOUT: u64 = 500;

fn attempt_connection(socket: &UdpSocket, peer_addr: SocketAddr, timeout: Duration) -> bool {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        let msg = b"Hello!";
        match socket.send_to(msg, peer_addr) {
            Ok(_) => {
                println!("Sent packet to {}", peer_addr);

                let mut buff = [0u8; 1024];
                match socket.recv_from(&mut buff) {
                    Ok((amt, src)) => {
                        println!(
                            "Received from {}: {}",
                            src,
                            String::from_utf8_lossy(&buff[..amt])
                        );
                        return true;
                    }
                    Err(_) => continue,
                }
            }
            Err(e) => {
                eprintln!("Failed to send to {}: {}", peer_addr, e);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn main() {
    dotenv().ok();

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");
    println!("Server address: {}", server_addr);

    let mut stream = TcpStream::connect(server_addr).expect("Could not connect to server");

    let private_ip = stream.local_addr().unwrap().ip();
    let private_port = stream.local_addr().unwrap().port();
    let client_info = format!("{}:{}", private_ip, private_port);
    stream.write_all(client_info.as_bytes()).unwrap();

    let mut buff = [0u8; 1024];
    let len = stream.read(&mut buff).unwrap();
    let buff_str = String::from_utf8_lossy(&buff[..len]);
    println!("another client info: \n{}", buff_str);

    let mut parts = buff_str.splitn(2, "\n");
    let peer_addr_pub = parts.next().unwrap_or_default().to_string();
    let peer_addr_priv = parts.next().unwrap_or_default().to_string();

    // First, find peer from LAN
    let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", private_port)).unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT)))
        .unwrap();

    // listening thread detach from main thread
    let receive_socket = udp_socket.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut buff = [0u8; 1024];
        loop {
            match receive_socket.recv_from(&mut buff) {
                Ok((amt, src)) => {
                    let message = String::from_utf8_lossy(&buff[..amt]);
                    println!("Message from {}: {}", src, message);
                }
                Err(_) => {}
            }
        }
    });

    let message = "LAN Test Message";
    match udp_socket.send_to(message.as_bytes(), &peer_addr_priv) {
        Ok(_) => {
            println!("Successfully found a peer on the LAN");
            loop {
                let mut message = String::new();

                print!("Write Message to Peer: ");
                std::io::stdout().flush().unwrap();

                std::io::stdin().read_line(&mut message).unwrap();
                udp_socket
                    .send_to(message.as_bytes(), &peer_addr_priv)
                    .unwrap();
            }
        }
        Err(e) => {
            println!("Cannot find peer on the LAN: {}", e);

            // Next, find peer from WAN
            loop {
                let mut message = String::new();

                print!("Write Message to Peer: ");
                std::io::stdout().flush().unwrap();

                std::io::stdin().read_line(&mut message).unwrap();
                udp_socket
                    .send_to(message.as_bytes(), &peer_addr_pub)
                    .unwrap();
            }
        }
    }
}
