use lazy_static::lazy_static;
use std::io::Write;
use std::sync::Mutex;
use std::{io::Read, net::TcpListener, net::TcpStream};

lazy_static! {
    static ref A_IP_PRIVATE_ADDR: Mutex<Option<String>> = Default::default();
    static ref A_IP_PUBLIC_ADDR: Mutex<Option<String>> = Default::default();
    static ref B_IP_PRIVATE_ADDR: Mutex<Option<String>> = Default::default();
    static ref B_IP_PUBLIC_ADDR: Mutex<Option<String>> = Default::default();
}

static LISTEN_PORT: u16 = 5001;

fn main() {
    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", LISTEN_PORT))
        .expect("Could not bind to address");
    println!("Listening on port: {}", LISTEN_PORT);
    let mut buff = [0u8; 1024];
    let mut socket_vec: Vec<TcpStream> = Default::default();

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

                        println!("B join to server");
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

                println!("A join to server");
                println!(
                    "{}\n{}\n",
                    lock_a_pub.clone().unwrap(),
                    lock_a_pri.clone().unwrap()
                );
            }
        }
    }

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

    let mut client_a = &socket_vec[0];
    let mut client_b = &socket_vec[1];

    client_a
        .write_all(b_info.as_bytes()).unwrap();

    client_b
        .write_all(a_info.as_bytes()).unwrap();
}
