# Remote Administration Tool (RAT)

A modular Remote Administration Tool (RAT) written in **Rust**. Built as a Cargo Workspace, the project includes a client agent, a control server with a Terminal User Interface (TUI), and a network communication protocol.

> **Disclaimer:** 
> This project is created strictly for educational, demonstration, and research purposes. Using this software on devices without explicit prior authorization from the owner is illegal.

---

## 🚀 Features

- **Cargo Workspace Architecture:** Clean separation into client (`client`), server (`server`), and shared components.
- **Interactive TUI:** Feature-rich terminal interface for the server built on `ratatui` and `crossterm`.
- **Network Protocol:** Command and data transmission powered by `std::net::TcpListener` / `TcpStream`.
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
