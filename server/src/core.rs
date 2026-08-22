use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::mpsc;
use std::thread;

use crate::app::AppEvent;
use crossterm::event::{self};

#[derive(Debug)]
pub struct Core {
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub sender: mpsc::Sender<AppEvent>,
    pub receiver: mpsc::Receiver<AppEvent>,
}

impl Core {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        let (stream, addr) = listener
            .accept()
            .unwrap();

        let (sender, receiver) = mpsc::channel::<AppEvent>();
        
        Core {
            stream,
            addr,
            sender,
            receiver,
        }
    }

    pub fn apply_instructions(&mut self, input: String) -> Result<(), std::io::Error> {
        self.stream.write_all(input.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn start_listeners(&mut self) {
        // start keylogger
        let sender = self.sender.clone();
        if let Ok(mut stream) = self.stream.try_clone() {
            thread::spawn(move || {
                let mut buffer = [0; 512];

                while let Ok(bytes) = stream.read(&mut buffer[..]) { 
                    if bytes == 0 { break; }

                    if let Ok(key) = std::str::from_utf8(&buffer[..bytes]) {
                        if sender.send(AppEvent::Keylogger(key.to_string())).is_err() {
                            break;
                        }
                    }
                }
            });
        }

        // start terminal key events
        let sender = self.sender.clone();
        thread::spawn(move || {
            while let Ok(event) = event::read() {
                if let Some(key) = event.as_key_press_event() {
                    if sender.send(AppEvent::Terminal(key)).is_err() {
                        break;
                    }
                }
            }
        });
    }
}
