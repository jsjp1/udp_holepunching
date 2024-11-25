use dotenv::dotenv;
use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
    time::Duration,
};

static READ_TIMEOUT: u64 = 500;

fn main() {
    dotenv().ok();

    let (tx, rx) = std::sync::mpsc::channel();

    let _ = std::thread::spawn(move || loop {
        match rx.recv() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    });

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");
    let _ = tx.send(format!("Server addres: {}", server_addr));

    let mut stream = TcpStream::connect(server_addr).expect("Could not connect to server");

    let private_ip = stream.local_addr().unwrap().ip();
    let private_port = stream.local_addr().unwrap().port();
    let client_info = format!("{}:{}", private_ip, private_port);
    stream.write_all(client_info.as_bytes()).unwrap();

    let mut buff = [0u8; 1024];
    let len = stream.read(&mut buff).unwrap();
    let buff_str = String::from_utf8_lossy(&buff[..len]);
    let _ = tx.send(format!("another client info: \n{}", buff_str));

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
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let mut buff = [0u8; 1024];
        loop {
            match receive_socket.recv_from(&mut buff) {
                Ok((amt, src)) => {
                    let message = String::from_utf8_lossy(&buff[..amt]);
                    let _ = tx_clone.send(format!("Message from {}: {}", src, message));
                }
                Err(_) => {}
            }
        }
    });

    let message = "LAN Test Message";
    let tx_clone = tx.clone();
    match udp_socket.send_to(message.as_bytes(), &peer_addr_priv) {
        Ok(_) => {
            let _ = tx_clone.send(format!("Successfully found a peer on the LAN"));
            loop {
                let mut message = String::new();

                let _ = tx_clone.send(format!("Write Message to Peer: "));
                std::io::stdout().flush().unwrap();

                std::io::stdin().read_line(&mut message).unwrap();
                udp_socket
                    .send_to(message.as_bytes(), &peer_addr_priv)
                    .unwrap();
            }
        }
        Err(e) => {
            let _ = tx_clone.send(format!("Cannot find peer on the LAN: {}", e));

            // Next, find peer from WAN
            loop {
                let mut message = String::new();

                let _ = tx_clone.send(format!("Write Message to Peer: "));
                std::io::stdout().flush().unwrap();

                std::io::stdin().read_line(&mut message).unwrap();
                udp_socket
                    .send_to(message.as_bytes(), &peer_addr_pub)
                    .unwrap();
            }
        }
    }
}
