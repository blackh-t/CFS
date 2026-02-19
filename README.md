# CFS

**Cryptographic File Sharing** is a lightweight, secure file transfer protocol built in Rust.

## Features

- **Handshake**: Uses RSA-2048 to securely establish and exchange session keys between parties. **!!NO TSL**
- **Cryptographic**: Includes built-in support for AES-256-CBC encryption with Randomized IVs to ensure unique ciphertexts for identical data.
- **LVF Framing**: Custom Length-Value Framing protocol (4-byte header), reliable TCP streaming.
- **Async Engine**: Fully non-blocking I/O powered by the tokio runtime.
- **Performance Profiling**: Built-in instruction-level benchmarking using iai-callgrind for hardware-independent performance analysis.

## Protocol Details

1. **Handshake**:

- Client sends `HELLO`.
- Server sends RSA Public Key (PEM).
- Client generates AES-256 Session Key, encrypts it with RSA, and sends it back.

2. **Data Transfer**:

- All subsequent messages are encrypted using the shared AES key.
- Headers use a 4-byte big-endian length prefix.

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

## Performance Benchmarks

```
# Using the runner script (recommended)
./runner.sh  # Select option [3]

# Manual execution
cargo bench -p utils
```

- Instructions: Total CPU instructions executed per KB.
- L1 Hits: Cache efficiency during encryption/decryption.
- Estimated Cycles: Relative CPU cycle cost for cryptographic operations.

| Metric               | Encryption (`bench_encrypt_1kb`) | Decryption (`bench_decrypt_1kb`) |
| -------------------- | -------------------------------- | -------------------------------- |
| **Instructions**     | 3,921                            | 2,736                            |
| **L1 Hits**          | 5,164                            | 3,650                            |
| **Estimated Cycles** | 9,994                            | 5,190                            |
| **RAM Hits**         | 134                              | 44                               |

The result reveals that decryption approximately 30% more efficient than encryption. Due to the encryption process required more computational in generating random IV, calculate and append the **PKCS#7** padding to ensure the ciphertext fits 16-byte block boundaries. Decryption merely slices this padding away which involves fewer CPU computation, and less memory accesses then encryption, since encryption need to allocate memory to combine IV and ciphertext with padding to generate a complete payload.

This overhead can be reduced by generating the IV on the client side, which allow the server to handle multiple file requests in parallel much more efficiently.
