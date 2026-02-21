# bookmon

A Rust-based command-line book management system that helps you track your reading progress, manage your book collection, and organize books by categories, authors, and series.

## Prerequisites

- Rust toolchain (latest stable version)
- Cargo (comes with Rust)

## Installation via Homebrew

If you're on macOS, you can install bookmon using Homebrew:

1. Add the tap:
   ```bash
   brew tap benedicteb/bookmon git@github.com:benedicteb/bookmon.git
   ```

2. Install bookmon:
   ```bash
   brew install bookmon
   ```

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
- Series information
- Reviews
- Reading goals

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

#### Books
- `add-book` - Add a new book to your collection (with optional ISBN lookup)

#### Viewing Books
- `print-finished` - Show books that have been finished
- `print-backlog` - Show books that have not been started yet
- `print-want-to-read` - Show books in the want-to-read list
- `print-statistics` - Show reading statistics by year

The print commands `print-finished`, `print-backlog`, and `print-want-to-read` support filtering by series:

```bash
bookmon print-finished --series "Lord of the Rings"
bookmon print-backlog -s "Discworld"
```

#### Reading Goals
- `set-goal <number>` - Set a yearly reading goal (number of books to finish)
- `print-goal` - Show progress toward your reading goal

```bash
bookmon set-goal 24
bookmon set-goal 30 --year 2025
bookmon print-goal
bookmon print-goal --year 2025
```

When a goal is set for the current year, running `bookmon` with no command will also display your goal progress with a progress bar and motivational pace text.

#### Reviews
- `review-book` - Write a review for a book (opens your `$EDITOR`)
- `print-reviews` - Show all book reviews

#### Series Management
- `print-series` - Show all book series and their books
- `delete-series` - Delete a series (books are kept but unlinked)
- `rename-series` - Rename an existing series

Series can also be assigned to books through interactive mode.

#### ISBN Lookup
- `get-isbn <isbn>` - Fetch detailed book information using an ISBN

The application can fetch book information using ISBNs through multiple providers:
- **Open Library** - The primary provider, using the Open Library API
- **Bibsok** - A secondary provider using the Norwegian library search service

When looking up an ISBN, the application tries each provider in order and returns the first successful result, including:
- Book title
- Author information
- Publication date
- Description
- Cover image URL
- Series information (name and position)

Example:
```bash
bookmon get-isbn 0451524934
```

This feature is also integrated into the `add-book` flow, where entering an ISBN will automatically populate book details.

#### Configuration
- `change-storage-path <path>` - Change the storage file path
- `get-config-path` - Print the path to the config file

### Interactive Mode

You can run many commands in interactive mode by adding the `-i` or `--interactive` flag:

```bash
bookmon -i                    # Currently reading + want to read
bookmon print-finished -i
bookmon print-backlog -i
bookmon print-want-to-read -i
bookmon print-reviews -i
```

When you run the application without any commands, it defaults to showing your reading goal progress (if set) and currently-reading books:

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
   - Mark a book as want to read / unmark
   - Mark a book as bought
   - Assign a book to a series (or change/remove series assignment)
   - Write a review for a book

In interactive review mode (`print-reviews -i`), you can browse and view full review text for any review.

## Development

To run tests:
```bash
cargo test
```

To check code formatting:
```bash
cargo fmt -- --check
```

To run the linter:
```bash
cargo clippy
```
