# AGENTS.md - bookmon

## Project Overview

**bookmon** is a Rust-based command-line book management system that helps users track reading progress, manage book collections, and organize books by categories and authors.

- **Language:** Rust (edition 2021)
- **Version:** See `Cargo.toml` for current version
- **Build system:** Cargo

## Architecture

### Source Layout

```
src/
  main.rs          # CLI entry point (clap-based), interactive mode
  lib.rs           # Public module declarations
  storage.rs       # Core data model (Book, Author, Category, Reading, Series, Storage) + JSON persistence
  book.rs          # Book input, ISBN lookup integration, book storage
  author.rs        # Author management
  category.rs      # Category management
  reading.rs       # Reading event tracking, display/printing
  series.rs        # Series management, display, filtering
  review.rs        # Book review management (editor workflow)
  goal.rs          # Reading goal tracking and motivational text
  table.rs         # Unicode-aware table formatting
  config.rs        # App configuration (storage path, settings)
  lookup/
    http_client.rs       # HTTP client for ISBN lookups
    book_lookup_dto.rs   # DTO for book lookup results
    providers.rs         # Provider trait + manager (multi-provider ISBN lookup)
    providers/
      openlibrary.rs     # Open Library API provider
      bibsok.rs          # Bibsok (Norwegian library) provider
```

### Key Concepts

- **Storage:** All data persists in a single JSON file. The `Storage` struct holds `HashMap`s of `Book`, `Author`, `Category`, `Reading`, `Series`, and `Review`.
- **Reading events:** Books are tracked via `Reading` entries with events: `Started`, `Finished`, `Update`, `Bought`, `WantToRead`, `UnmarkedAsWantToRead`. The most recent event determines current status.
- **Providers:** ISBN lookup uses a `BookProvider` trait with multiple implementations (OpenLibrary, Bibsok). `ProviderManager` tries each provider in order.
- **CLI:** Built with `clap` (derive). Supports both command mode and interactive mode (`-i` flag) using `inquire` for prompts.

### Key Dependencies

- `clap` - CLI argument parsing
- `serde` / `serde_json` / `serde_yaml` - Serialization
- `chrono` - Date/time handling
- `uuid` - ID generation
- `reqwest` + `tokio` - Async HTTP for ISBN lookups
- `inquire` - Interactive terminal prompts
- `scraper` + `regex` - HTML scraping (Bibsok provider)
- `config` - Configuration management
- `tempfile` (dev) - Temporary files in tests

## Build & Test Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test
cargo fmt                # Format code
cargo fmt -- --check     # Check formatting without modifying
cargo clippy             # Run linter
```

## Test Structure

Tests live in `tests/` as integration tests:

```
tests/
  author_test.rs
  book_test.rs
  category_test.rs
  config_test.rs
  http_client_test.rs
  interactive_test.rs
  lookup_test.rs
  lookup/
  main_test.rs
  reading_test.rs
  storage_test.rs
```

## Configuration

- App config is stored in a platform-specific config directory (via `dirs` crate)
- `config/default.yml` contains default settings
- Storage file path is configured via `bookmon change-storage-path <path>`

## Conventions

- IDs are UUIDs (v4) stored as strings
- All timestamps use `chrono::DateTime<Utc>`
- Storage JSON is sorted deterministically before writing (via `sort_json_value`)
- Integration tests use `tempfile` for isolated storage files
- No ORM - direct JSON file I/O

## Agent Collaboration

The project uses a multi-agent workflow. The primary developer agent actively consults specialist subagents during planning, implementation, and review — not just at session end.

### Agents

| Agent | Role | When consulted |
|---|---|---|
| `@rust-developer` | Primary. Writes code, runs tests, makes commits. | Always active |
| `@book-domain-expert` | Domain specialist for books, publishing, authors, bibliographic standards. | Data model changes, terminology decisions, new features involving book concepts |
| `@ux-designer` | UX specialist for terminal applications. | CLI commands, interactive prompts, output formatting, error messages |
| `@reviewer` | Code reviewer. Checks correctness, tests, Rust idioms, commit hygiene. | End of session |

### Collaboration Model

- **Before planning:** The developer consults `@book-domain-expert` and/or `@ux-designer` to validate understanding and surface domain or UX concerns early.
- **Before implementing user-facing changes:** The developer consults `@ux-designer` on interaction design.
- **Before implementing model changes:** The developer consults `@book-domain-expert` on data modeling and terminology.
- **After implementing:** The developer calls `@ux-designer` to review the actual output/flow.
- **At session end:** All three subagents (`@ux-designer`, `@book-domain-expert`, `@reviewer`) review the session's work.
- **In ADRs:** Subagent input is recorded in a dedicated "Subagent Input" section when domain or UX decisions are involved.

## Documentation

- `docs/adr/` - Architecture Decision Records (ADR format, includes subagent input)
- `docs/learnings/` - Session learning summaries
