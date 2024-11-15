use lazy_static::lazy_static;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

lazy_static! {
    static ref A_IP_PRIVATE_ADDR: Mutex<Option<String>> = Default::default();
    static ref A_IP_PUBLIC_ADDR: Mutex<Option<String>> = Default::default();
    static ref B_IP_PRIVATE_ADDR: Mutex<Option<String>> = Default::default();
    static ref B_IP_PUBLIC_ADDR: Mutex<Option<String>> = Default::default();
}

static LISTEN_PORT: u16 = 5001;

fn main() {
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", LISTEN_PORT)).expect("Could not bind to address");
    println!("Listening on port: {}", LISTEN_PORT);
    let mut buff = [0u8; 1024];
    let mut socket_vec = Vec::new();

    while let Ok((stream, _)) = listener.accept() {
        let mut stream = stream.try_clone().expect("Could not clone stream");
        socket_vec.push(stream.try_clone().unwrap());

        let data_len = stream.read(&mut buff).unwrap();
        let mut lock_a_pub = A_IP_PUBLIC_ADDR.lock().unwrap();

        match &*lock_a_pub {
            Some(_) => {
                let mut lock_b_pub = B_IP_PUBLIC_ADDR.lock().unwrap();
                match &*lock_b_pub {
                    Some(_) => {}
                    None => {
                        let pub_addr = String::from_utf8_lossy(&buff[..data_len]);
                        *lock_b_pub = Some(format!("{}", &pub_addr));

                        let client_info = stream.peer_addr().unwrap();
                        let mut lock_b_pri = B_IP_PRIVATE_ADDR.lock().unwrap();
                        *lock_b_pri = Some(format!("{}:{}", client_info.ip(), client_info.port()));

                        println!("B joined the server");
                        println!(
                            "{}\n{}\n",
                            lock_b_pub.clone().unwrap(),
                            lock_b_pri.clone().unwrap()
                        );
                        break;
                    }
                }
            }
            None => {
                let pub_addr = String::from_utf8_lossy(&buff[..data_len]);
                *lock_a_pub = Some(format!("{}", &pub_addr));

                let client_info = stream.peer_addr().unwrap();
                let mut lock_a_pri = A_IP_PRIVATE_ADDR.lock().unwrap();
                *lock_a_pri = Some(format!("{}:{}", client_info.ip(), client_info.port()));

                println!("A joined the server");
                println!(
                    "{}\n{}\n",
                    lock_a_pub.clone().unwrap(),
                    lock_a_pri.clone().unwrap()
                );
            }
        }
    }

    // Send peer's address to each other
    let a_info = format!(
        "{}\n{}",
        A_IP_PRIVATE_ADDR.lock().unwrap().take().unwrap(),
        A_IP_PUBLIC_ADDR.lock().unwrap().take().unwrap()
    );
    let b_info = format!(
        "{}\n{}",
        B_IP_PRIVATE_ADDR.lock().unwrap().take().unwrap(),
        B_IP_PUBLIC_ADDR.lock().unwrap().take().unwrap()
    );

    let client_a = Arc::new(Mutex::new(socket_vec[0].try_clone().unwrap()));
    let client_b = Arc::new(Mutex::new(socket_vec[1].try_clone().unwrap()));

    client_a
        .lock()
        .unwrap()
        .write_all(b_info.as_bytes())
        .unwrap();
    client_b
        .lock()
        .unwrap()
        .write_all(a_info.as_bytes())
        .unwrap();

    // Start threads to relay data between A and B
    let client_a_clone = Arc::clone(&client_a);
    let client_b_clone = Arc::clone(&client_b);

    // A -> B
    let handle_a_to_b = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match client_a_clone.lock().unwrap().read(&mut buffer) {
                Ok(size) if size > 0 => {
                    if client_b_clone
                        .lock()
                        .unwrap()
                        .write_all(&buffer[..size])
                        .is_err()
                    {
                        println!("Failed to write data from A to B");
                        break;
                    }
                }
                Ok(_) => break,
                Err(_) => {
                    println!("Connection closed by A");
                    break;
                }
            }
        }
    });

    // B -> A
    let handle_b_to_a = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match client_b.lock().unwrap().read(&mut buffer) {
                Ok(size) if size > 0 => {
                    if client_a.lock().unwrap().write_all(&buffer[..size]).is_err() {
                        println!("Failed to write data from B to A");
                        break;
                    }
                }
                Ok(_) => break,
                Err(_) => {
                    println!("Connection closed by B");
                    break;
                }
            }
        }
    });

    handle_a_to_b.join().unwrap();
    handle_b_to_a.join().unwrap();
}
