use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::AppEvent;
use crossterm::event::{self};

#[derive(Debug)]
pub struct Core {
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub addr: Arc<Mutex<Option<SocketAddr>>>,
    pub sender: mpsc::Sender<AppEvent>,
    pub receiver: mpsc::Receiver<AppEvent>,
}

impl Core {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        let stream = Arc::new(Mutex::new(None));
        let addr = Arc::new(Mutex::new(None));

        let stream_clone = Arc::clone(&stream);
        let addr_clone = Arc::clone(&addr);

        thread::spawn(move || {
            let (stream, addr) = listener.accept().unwrap();

            let mut stream_guard = stream_clone.lock().unwrap();
            let mut addr_guard = addr_clone.lock().unwrap();

            *stream_guard = Some(stream);
            *addr_guard = Some(addr);
        });
        

        let (sender, receiver) = mpsc::channel::<AppEvent>();
        
        Core {
            stream,
            addr,
            sender,
            receiver,
        }
    }

    pub fn apply_instructions(&mut self, input: String) -> Result<(), std::io::Error> {
        let mut stream_guard = self.stream.lock().unwrap();
        let stream = (*stream_guard).as_mut();

        match stream {
            Some(stream) => {
                stream.write_all(input.as_bytes())?;
                stream.flush()?;
            },
            None => {},
        }
        Ok(())
    }

    pub fn start_listeners(&mut self) {
        // start keylogger
        let stream = {
            let mut stream_guard = self.stream.lock().unwrap();
            stream_guard.take()
        };

        let sender = self.sender.clone();
        thread::spawn( move || {
            let Some(mut stream) = stream else {
                return;
            };

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
