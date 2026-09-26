# Remote Administration Tool (RAT)

A modular Remote Administration Tool (RAT) written in **Rust**. Built as a Cargo Workspace, the project includes a client agent, a control server with a Terminal User Interface (TUI), and a network communication protocol.

> **Disclaimer:** 
> This project is created strictly for educational, demonstration, and research purposes. Using this software on devices without explicit prior authorization from the owner is illegal.

<img width="1280" height="720" alt="D__rust_rat_target_debug_server exe2026-09-2415-35-55-ezgif com-video-to-gif-converter" src="https://github.com/user-attachments/assets/250eb3bc-584a-4f26-a22a-696e265359a4" />

## Features

- **Cargo Workspace Architecture:** Clean separation into client (`client`), server (`server`).
- **Interactive TUI:** Feature-rich terminal interface for the server built on `ratatui` and `crossterm`.
- **Network Protocol:** Command and data transmission powered by `tokio::net::TcpListener` / `TcpStream`.
- **Client Capabilities:**
  - Remote command execution.
  - Input event capture using the `rdev` library.
  - Client screen broadcast.

## Prerequisites & Building

Building and running this project requires **Rust** (2021 edition or newer) and `cargo`.

### 1. Clone the repository
```bash
git clone https://github.com/yatsenk/rat.git
cd rat
```
### 2. Build the workspace
```bash
cargo build --release
```
### 3. Run the Control Server
```bash
cargo run --bin server
```
### 4. Run the Client Agent
```bash
cargo run --bin client
```

## Connecting from Other Devices (via ngrok)

Since the application is configured for localhost by default, you can use [ngrok](https://ngrok.com/) to expose your local server to the internet for testing or sharing with others without needing a public static IP address.

### Step-by-Step Guide:

1. **Create a tunnel using ngrok:**
   * For HTTP / WebSocket connections:
     ```bash
     ngrok http 8080
     ```
   * For raw TCP traffic:
     ```bash
     ngrok tcp 7878
     ```
2. **Copy the address and port from the ngrok terminal:**
   * Example for HTTP/WS: `https://xxxx-xx-xx.ngrok-free.app` (or `tcp://0.tcp.eu.ngrok.io:12345` for TCP).
3. **Update the client configuration in the code:**
   * Open the file where the server connection address is defined (`client/src/main.rs`).
   * Replace the local address (like `127.0.0.1:8080`) with the data provided by ngrok (for HTTP/WS you can use `wss://xxxx-xx-xx.ngrok-free.app`, for TCP use the host and port accordingly).
4. **Compile and build the binaries:**
   ```bash
   cargo build --release
   ```
5. **Run the binaries**
   * Run compiled binaries in your ./target/release directory.

## Downloads (Pre-built Binaries)

You can download the latest compiled binaries from the [Releases page](../../releases/latest):

### Windows (x86_64)
[![Download Server](https://img.shields.io/badge/Download-server.exe-2496ED?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/v0.1.0/server.exe)
[![Download Client](https://img.shields.io/badge/Download-client.exe-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/v0.1.0/client.exe)
  
### Linux (x86_64)
[![Download Server](https://img.shields.io/badge/Download-server.exe-2496ED?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/v0.1.1/server)
[![Download Client](https://img.shields.io/badge/Download-client.exe-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/v0.1.1/client)

## Contributing
Contributors or Pull Requests are Welcome!!!
