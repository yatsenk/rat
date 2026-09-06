use tokio::sync::mpsc;
use tokio::net::TcpListener;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};

pub enum ClientEvent {
    Connected(std::net::SocketAddr),
    Disconnected,
    Data(Vec<u8>),
}

pub struct ClientHandle {
    pub app_sender: mpsc::Sender<Vec<u8>>,   
    pub app_receiver: mpsc::Receiver<ClientEvent>, 
}

pub fn spawn_client(addr: &str) -> ClientHandle {
    let (app_sender, mut socket_receiver) = mpsc::channel::<Vec<u8>>(32);
    let (socket_sender, app_receiver) = mpsc::channel::<ClientEvent>(32);
    let addr = addr.to_string();

    tokio::spawn(async move {
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(_) => return,
        };

        loop {
            let (socket, peer_addr) = match listener.accept().await {
                Ok(pair) => pair,
                Err(_) => continue,
            };
            let _ = socket_sender.send(ClientEvent::Connected(peer_addr)).await;

            let (mut rd, mut wr) = io::split(socket);
            let mut buf = [0u8; 1024];

            loop {
                tokio::select! {
                    Some(input) = socket_receiver.recv() => {
                        if wr.write_all(&input).await.is_err() {
                            break;
                        }
                    }
                    read_result = rd.read(&mut buf) => {
                        match read_result {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let _ = socket_sender.send(ClientEvent::Data(buf[..n].to_vec())).await;
                            }
                        }
                    }
                }
            }
            let _ = socket_sender.send(ClientEvent::Disconnected).await;
        }
    });

    ClientHandle { app_sender, app_receiver }
}
