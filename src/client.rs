use dotenv::dotenv;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

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
    println!(
        "another client info: \n{}",
        String::from_utf8_lossy(&buff[..len])
    );
}
