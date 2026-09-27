use std::net::SocketAddr;

use axum::{
    extract::{
        Request,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
        State,
    },
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use futures::{StreamExt, SinkExt};

use tokio::sync::broadcast;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver}; 
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf};

pub enum ClientEvent {
    Connected(std::net::SocketAddr),
    Disconnected,
    Data(Vec<u8>),
}

pub struct ClientHandle {
    pub app_sender: mpsc::Sender<Vec<u8>>,   
    pub app_receiver: mpsc::Receiver<ClientEvent>, 
    pub addr1: String,
    pub addr2: String,
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Vec<u8>>,
}

pub async fn spawn_client() -> Result<ClientHandle, io::Error> {
    let tcp_listener = TcpListener::bind("127.0.0.1:7878").await?;
    let websocket_listener = TcpListener::bind("127.0.0.1:8080").await?;

    let keys = String::new();

    let (tx, _) = broadcast::channel::<Vec<u8>>(16);
    let state = AppState { tx };

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .with_state(state);

    let (app_sender, mut socket_receiver) = mpsc::channel::<Vec<u8>>(32);
    let (socket_sender, app_receiver) = mpsc::channel::<ClientEvent>(32);

    tokio::spawn(async move {
        axum::serve(websocket_listener, app).await;
    });

    tcp_stream(tcp_listener, socket_sender, socket_receiver).await;

    Ok(ClientHandle { 
        app_sender, 
        app_receiver,
        addr1: String::new(),
        addr2: String::new(),
    })
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, state))
}

async fn websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            if sender.send(Message::Binary(data.into())).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Binary(bytes) = msg {
                if tx.send(bytes.to_vec()).is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../../web/index.html"))
}

async fn tcp_stream(
    listener: TcpListener, 
    socket_sender: Sender<ClientEvent>,
    mut socket_receiver: Receiver<Vec<u8>>,
) {
    tokio::spawn(async move {
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
}
