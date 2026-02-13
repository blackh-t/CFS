# CFS
**Cryptographic File Sharing** is a lightweight, secure file transfer protocol built in Rust.


## Features

* **Handshake**: Uses RSA-2048 to securely establish and exchange session keys between parties. **!!NO TSL**
* **Cryptographic**: Includes built-in support for AES-256-CBC encryption with Randomized IVs to ensure unique ciphertexts for identical data.
* **LVF Framing**: Custom Length-Value Framing protocol (4-byte header), reliable TCP streaming.
* **Async Engine**: Fully non-blocking I/O powered by the tokio runtime.

## Protocol Details

1. **Handshake**:
* Client sends `HELLO`.
* Server sends RSA Public Key (PEM).
* Client generates AES-256 Session Key, encrypts it with RSA, and sends it back.

2. **Data Transfer**:
* All subsequent messages are encrypted using the shared AES key.
* Headers use a 4-byte big-endian length prefix.

## Installation
Ensure you have the Rust toolchain installed.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Quick Start

Use the included `runner.sh` for an easy, interactive setup:

```bash
git clone https://github.com/blackh-t/CFS.git
cd CFS

chmod +x runner.sh
./runner.sh

```
<p align="center">
  <img src="https://github.com/blackh-t/CFS/raw/dev/cfs.gif" width="1200" alt="CFS Demo">
</p>

### Manual Usage

**Start the Server:**

```bash
export CFS_PORT=45928
cargo run --bin server

```

**Run the Client:**

```bash
export CFS_IP=127.0.0.1
export CFS_PORT=45928
cargo run --bin client

```

## Testing

The project includes a robust test suite covering cryptographic round-trips, handshake logic, and network framing.

```bash
RUST_LOG=info cargo test -- --nocapture

```

