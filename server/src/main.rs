use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    println!("[*] client is connected");
    let mut buf = [0; 512];
    loop {
        print!(">> ");
        io::stdout().flush().unwrap(); 
        /*
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("could not read line");

        stream.write_all(input.as_bytes()).unwrap(); 
        */
        while let Ok(bytes) = stream.read(&mut buf[..]) { 
            let key = std::str::from_utf8(&buf[..bytes]);
            println!("User wrote: {}", key.unwrap());
        } 

        return;
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("[*] listener is created, waiting for client ...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(_e) => { println!("[ERROR] could not connect to server") }
        }
    }
}
