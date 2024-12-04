use dotenv::dotenv;
use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
    time::Duration,
};

static READ_TIMEOUT: u64 = 50;
static MAX_RETRIES: u32 = 3;

fn send_message(socket: &UdpSocket, message: &str, peer_addr: &str, try_count: u32) -> bool {
    let mut message_send_success = false;

    socket.send_to(message.as_bytes(), &peer_addr).unwrap();

    let mut buff = [0u8; 1024];
    for attempt in 1..=try_count {
        println!("Attempt {} to test connection...", attempt);

        socket
            .send_to(message.as_bytes(), &peer_addr)
            .expect("Failed to send test message");

        match socket.recv_from(&mut buff) {
            Ok((amt, src)) => {
                let message = String::from_utf8_lossy(&buff[..amt]);
                println!("Received message from {}: {}", src, message);
                message_send_success = true;
                break;
            }
            Err(e) => {
                println!("Timeout or error receiving response: {}", e);
            }
        }
    }

    return message_send_success;
}

fn main() {
    dotenv().ok();
    let mut lan_test_success = false;

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || loop {
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
    tx.send(format!("Server addres: {}", server_addr)).unwrap();

    let mut stream = TcpStream::connect(server_addr).expect("Could not connect to server");

    let private_ip = stream.local_addr().unwrap().ip();
    let private_port = stream.local_addr().unwrap().port();
    let client_info = format!("{}:{}", private_ip, private_port);
    stream.write_all(client_info.as_bytes()).unwrap();

    let mut buff = [0u8; 1024];
    let len = stream.read(&mut buff).unwrap();
    let buff_str = String::from_utf8_lossy(&buff[..len]);
    tx.send(format!("another client info: \n{}", buff_str))
        .unwrap();

    let mut parts = buff_str.splitn(2, "\n");
    let peer_addr_pub = parts.next().unwrap_or_default().to_string();
    let peer_addr_priv = parts.next().unwrap_or_default().to_string();

    // create udp socket
    let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", private_port)).unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT)))
        .unwrap();

    // test LAN communication
    send_message(
        &udp_socket,
        "LAN Test Message",
        &peer_addr_priv,
        MAX_RETRIES,
    );

    // create Receive only thread
    let tx_clone = tx.clone();
    let receive_socket = udp_socket.try_clone().unwrap();

    let _ = std::thread::spawn(move || {
        let mut buff = [0u8; 1024];

        loop {
            match receive_socket.recv_from(&mut buff) {
                Ok((amt, src)) => {
                    let message: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buff[..amt]);
                    tx_clone
                        .send(format!("Message from {}: {}", src, message))
                        .unwrap();
                    tx_clone
                        .send("Write message to peer: ".to_string())
                        .unwrap();
                }
                Err(_) => {}
            }
        }
    });

    // test WAN communication
    // udp hole punching, send message to peer to open port
    // because of Restriced Cone NAT & Symmetric NAT, have to send to initial message for peer each other (for allow peer to send message)
    send_message(&udp_socket, "WAN Test Message", &peer_addr_pub, MAX_RETRIES);

    loop {
        tx.send("Write message to peer: ".to_string()).unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if lan_test_success == true {
            udp_socket
                .send_to(input.as_bytes(), &peer_addr_priv)
                .unwrap();
        } else {
            udp_socket
                .send_to(input.as_bytes(), &peer_addr_pub)
                .unwrap();
        }
    }
}
