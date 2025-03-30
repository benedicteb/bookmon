# bookmon

A Rust-based command-line book management system that helps you track your reading progress, manage your book collection, and organize books by categories and authors.

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

## Configuration

The application uses a configuration system that stores settings in a config file. The most important setting is the storage file path, which must be set before using the application. You can set it using:

```bash
bookmon change-storage-path <path>
```

### Storage File

The application stores all your book data in a JSON file. This file contains:
- Your book collection
- Authors
- Categories
- Reading progress and history

The storage file is pure JSON, making it:
- Human-readable and editable
- Easy to backup
- Perfect for version control (e.g., Git)
- Portable across different systems

You can place the storage file anywhere on your system, including:
- Your private data repository
- A cloud-synced folder
- Any location that works well with your backup strategy

Example storage file location:
```bash
bookmon change-storage-path ~/Documents/books.json
# or
bookmon change-storage-path ~/dotfiles/private/bookmon/books.json
```

## Usage

The application can be used in two modes:

### Command Mode

Run specific commands using the following syntax:

```bash
bookmon <command>
```

Available commands:
- `add-book` - Add a new book to your collection
- `add-category` - Add a new category for organizing books
- `add-author` - Add a new author
- `add-reading` - Add a reading event for a book
- `currently-reading` - Show books that have been started but not finished
- `print-finished` - Show books that have been finished
- `print-backlog` - Show books that have not been started yet
- `print-all` - Show all books in your library
- `change-storage-path` - Change the storage file path
- `get-config-path` - Print the path to the config file
- `get-isbn <isbn>` - Fetch detailed book information using an ISBN

### ISBN Lookup

The application can fetch detailed book information using ISBNs through the Open Library API. When you use the `get-isbn` command, it will retrieve:
- Book title
- Author information (including personal name, birth/death dates, and biography)
- First publication date
- Book description
- Cover image IDs

Example:
```bash
bookmon get-isbn 0451524934
```

This feature is particularly useful when adding new books to your collection, as it can automatically populate many details for you.

### Interactive Mode

When run without any commands, the application enters interactive mode:
```bash
bookmon
```

In interactive mode, you can:
1. View a list of your books with their current status
2. Select a book to perform actions on it
3. Available actions include:
   - Start reading a book
   - Update reading progress (with page number)
   - Mark a book as finished

## Development

To run tests:
```bash
cargo test
```

To check code formatting:
```bash
cargo fmt
```
