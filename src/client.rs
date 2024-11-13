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
    let mut buff = [0u8; 1024];

    let private_ip = stream.local_addr().unwrap().ip();
    let private_port = stream.local_addr().unwrap().port();
    let _ = stream.write(format!("{}:{}", private_ip, private_port).as_bytes());
}
