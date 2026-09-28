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

You should use [ngrok](https://ngrok.com/) to expose your local server to the internet for testing or sharing with others without needing a public static IP address. Since the architecture uses two separate connections (one for raw TCP traffic and one for WebSockets), you will need to set up two parallel ngrok tunnels.

### Step-by-Step Guide:

1. **Create two tunnels using ngrok:**
   * Open two separate terminal windows and run a tunnel for each port:
   * **For WebSocket connections (HTTP/WS tunnel):**
     ```bash
     ngrok http 8080
     ```
   * **For raw TCP traffic:**
     ```bash
     ngrok tcp 7878
     ```

2. **Copy the addresses and ports from the ngrok terminals:**
   * For WebSocket, copy the generated public URL (e.g., `https://xxxx-xx-xx.ngrok-free.app`, IMPORTANT: replace `https` with `wss` and add `/ws` at the end).
   * For TCP, copy the host and port from the TCP forwarding line (e.g., `0.tcp.eu.ngrok.io:12345`).

3. **Run the client application:**
   * Start your compiled client binary (or run it via `cargo run --bin client`).
   * When prompted by the application, enter the two addresses provided by ngrok:
     1. **TCP address/port** (e.g., `0.tcp.eu.ngrok.io:12345`)
     2. **WebSocket URL** (e.g., `wss://xxxx-xx-xx.ngrok-free.app/ws`)

## Downloads (Pre-built Binaries)

You can download the latest compiled binaries from the [Releases page](../../releases/latest):

### Windows (x86_64)
[![Download Server](https://img.shields.io/badge/Download-server.exe-2496ED?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/0.2.1/server-x86_64-pc-windows-msvc.exe)
[![Download Client](https://img.shields.io/badge/Download-client.exe-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/0.2.1/client-x86_64-pc-windows-msvc.exe)
  
### Linux (x86_64)
[![Download Server](https://img.shields.io/badge/Download-server.exe-2496ED?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/0.2.1/server-x86_64-unknown-linux-gnu)
[![Download Client](https://img.shields.io/badge/Download-client.exe-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/download/0.2.1/client-x86_64-unknown-linux-gnu)

## Contributing
Contributors or Pull Requests are Welcome!!!
