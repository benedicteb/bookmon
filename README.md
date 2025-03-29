# Bookmon

A Rust project for book management.

## Prerequisites

- Rust toolchain (latest stable version)
- Cargo (comes with Rust)

## Installation

1. Install Rust by following the instructions at [rustup.rs](https://rustup.rs/)
   - For Windows, download and run the rustup-init.exe installer
   - For Unix-based systems, run:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

2. Verify the installation:
   ```bash
   rustc --version
   cargo --version
   ```

## Building the Project

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd bookmon
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. For release build (optimized):
   ```bash
   cargo build --release
   ```

The compiled binary will be available in:
- Debug build: `target/debug/bookmon`
- Release build: `target/release/bookmon`

## Running the Project

To run the project:
```bash
cargo run
```

For release mode:
```bash
cargo run --release
```

## Development

To run tests:
```bash
cargo test
```

To check code formatting:
```bash
cargo fmt
```

## License

[Add your license information here] 