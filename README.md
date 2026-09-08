# Remote Administration Tool (RAT)

A modular Remote Administration Tool (RAT) written in **Rust**. Built as a Cargo Workspace, the project includes a client agent, a control server with a Terminal User Interface (TUI), and a network communication protocol.

> **Disclaimer:** 
> This project is created strictly for educational, demonstration, and research purposes. Using this software on devices without explicit prior authorization from the owner is illegal.

---

## 🚀 Features

- **Cargo Workspace Architecture:** Clean separation into client (`client`), server (`server`).
- **Interactive TUI:** Feature-rich terminal interface for the server built on `ratatui` and `crossterm`.
- **Network Protocol:** Command and data transmission powered by `tokio::net::TcpListener` / `TcpStream`.
- **Client Capabilities:**
  - Remote command execution.
  - Input event capture using the `rdev` library.
  - Real-time system information and status streaming.

---

## 💻 Prerequisites & Building

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

## 📥 Downloads (Pre-built Binaries)

You can download the latest compiled binaries for Windows directly from the [Releases page](../../releases/latest):

[![Download Client](https://img.shields.io/badge/Download-client.exe-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/latest/download/client.exe)
[![Download Server](https://img.shields.io/badge/Download-server.exe-2496ED?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yatsenk/rat/releases/latest/download/server.exe)

> **Note:** Make sure to upload `client.exe` and `server.exe` as release assets in your GitHub repository Releases section.
