mod config;
use bookmon::{
    book,
    lookup::http_client,
    reading,
    storage::{self, Book, Storage},
};
use chrono::Datelike;
use clap::{Parser, Subcommand};
use inquire::{Select, Text};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Run in interactive mode
    #[arg(short, long, global = true)]
    interactive: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new book to the collection
    AddBook,
    /// Show books that have been finished
    PrintFinished,
    /// Show books that have not been started yet
    PrintBacklog,
    /// Show books that are in the want to read list
    PrintWantToRead,
    /// Show reading statistics by year
    PrintStatistics,
    /// Change the storage file path
    ChangeStoragePath {
        /// The new path for the storage file
        path: String,
    },
    /// Print the path to the config file
    GetConfigPath,
    /// Get book information by ISBN
    GetIsbn {
        /// The ISBN to look up
        isbn: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = config::Settings::load().expect("Failed to load config");
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::ChangeStoragePath { path }) => {
            settings.storage_file = path;
            settings.save()?;
            println!("Storage path updated successfully!");
            return Ok(());
        }
        _ => {
            if settings.storage_file.is_empty() {
                eprintln!("Error: Storage path not set. Please set it using the change-storage-path command.");
                std::process::exit(1);
            }
        }
    }

    // Initialize storage file if it doesn't exist
    if let Err(e) = storage::initialize_storage_file(&settings.storage_file) {
        eprintln!("Failed to initialize storage file: {}", e);
        std::process::exit(1);
    }

    let mut storage =
        storage::load_storage(&settings.storage_file).expect("Failed to load storage");

    // Handle the default case (no command) - show currently-reading
    if cli.command.is_none() {
        if cli.interactive {
            // Interactive mode for currently-reading
            interactive_mode(&storage, &settings.storage_file, None)?;
        } else {
            // Just show currently-reading
            match reading::show_started_books(&storage) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to show started books: {}", e),
            }
        }
        return Ok(());
    }

    // Handle commands with interactive flag
    match cli.command.as_ref().unwrap() {
        Commands::AddBook => {
            match book::get_book_input(&mut storage) {
                Ok((book, event)) => {
                    match book::store_book(&mut storage, book.clone()) {
                        Ok(_) => {
                            // Store all reading events
                            for event_type in event {
                                let reading = storage::Reading::new(book.id.clone(), event_type);
                                if let Err(e) = reading::store_reading(&mut storage, reading) {
                                    eprintln!("Failed to store reading event: {}", e);
                                }
                            }

                            storage::write_storage(&settings.storage_file, &storage)
                                .expect("Failed to save storage");
                            println!("Book added successfully!");
                        }
                        Err(e) => eprintln!("Failed to add book: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to get book input: {}", e),
            }
        }
        cmd @ (Commands::PrintFinished
        | Commands::PrintBacklog
        | Commands::PrintWantToRead
        | Commands::PrintStatistics) => {
            if cli.interactive {
                interactive_mode(&storage, &settings.storage_file, Some(cmd))?;
            } else {
                match cmd {
                    Commands::PrintFinished => match reading::show_finished_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show finished books: {}", e),
                    },
                    Commands::PrintBacklog => match reading::show_unstarted_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show unstarted books: {}", e),
                    },
                    Commands::PrintWantToRead => {
                        let want_to_read_books = storage.get_want_to_read_books();
                        match reading::print_book_list_table(
                            &storage,
                            want_to_read_books,
                            "No books in want to read list.",
                        ) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Failed to show want to read books: {}", e),
                        }
                    }
                    Commands::PrintStatistics => {
                        if let Some(earliest_year) = storage.get_earliest_finished_year() {
                            let current_year = chrono::Utc::now().year();
                            println!("\nReading Statistics by Year:");
                            println!("------------------------");

                            for year in earliest_year..=current_year {
                                let books = storage.get_books_finished_in_year(year);
                                if !books.is_empty() {
                                    println!("\n{}: {} books", year, books.len());
                                    for book in books {
                                        let author = storage.authors.get(&book.author_id).unwrap();
                                        println!("  - \"{}\" by {}", book.title, author.name);
                                    }
                                }
                            }
                        } else {
                            println!("No finished books found in your reading history.");
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        Commands::GetConfigPath => {
            println!("Config file path: {}", config::get_config_path()?.display());
        }
        Commands::GetIsbn { isbn } => {
            let client = http_client::HttpClient::new();
            let book = tokio::runtime::Runtime::new()?.block_on(client.get_book_by_isbn(&isbn))?;
            if let Some(book) = book {
                println!("Title: {}", book.title);
                println!("Authors:");
                for author in book.authors {
                    println!("  - {}", author.name);
                }
                if let Some(publish_date) = book.publish_date {
                    println!("Published: {}", publish_date);
                }
                if let Some(description) = book.description {
                    println!("Description: {}", description);
                }
                if let Some(cover_url) = book.cover_url {
                    println!("Cover URL: {}", cover_url);
                }
            } else {
                println!("No book found for ISBN {}", isbn);
            }
        }
        Commands::ChangeStoragePath { .. } => unreachable!(),
    }

    Ok(())
}

// Helper function for interactive mode
fn interactive_mode(
    storage: &Storage,
    storage_file: &str,
    command: Option<&Commands>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the appropriate books based on the command
    let filtered_books: Vec<&Book> = match command {
        None => storage.get_currently_reading_and_want_to_read_books(), // Default case - currently reading + want to read
        Some(cmd) => match cmd {
            Commands::PrintFinished => storage.get_finished_books(),
            Commands::PrintBacklog => storage.get_unstarted_books(),
            Commands::PrintWantToRead => storage.get_want_to_read_books(),
            Commands::PrintStatistics => storage.get_finished_books(),
            _ => storage.get_started_books(), // Fallback to currently reading
        },
    };

    if filtered_books.is_empty() {
        println!("No books available in this category.");
        return Ok(());
    }

    // Create options for book selection with status
    let mut options: Vec<(String, String)> = filtered_books
        .into_iter()
        .map(|b| {
            let status = if storage.is_book_started(&b.id) {
                "Started"
            } else {
                "Not Started"
            };
            let display = b.to_display_string(storage, status);
            (display, b.id.clone())
        })
        .collect();

    // Sort options by:
    // 1. Reading status (Started first)
    // 2. Author name
    // 3. Book title
    options.sort_by(|a, b| {
        let a_started = a.0.starts_with("[Started]");
        let b_started = b.0.starts_with("[Started]");

        if a_started != b_started {
            b_started.cmp(&a_started)
        } else {
            let a_author = a.0.split(" by ").nth(1).unwrap();
            let b_author = b.0.split(" by ").nth(1).unwrap();

            if a_author != b_author {
                a_author.cmp(b_author)
            } else {
                let a_title = Book::title_from_display_string(&a.0);
                let b_title = Book::title_from_display_string(&b.0);
                a_title.cmp(&b_title)
            }
        }
    });

    let options: Vec<String> = options.into_iter().map(|(display, _)| display).collect();

    // Let user select a book
    let book_selection = match Select::new("Select a book to update:", options).prompt() {
        Ok(selection) => selection,
        Err(_) => {
            println!("Operation cancelled");
            return Ok(());
        }
    };

    // Extract book title from selection
    let title = Book::title_from_display_string(&book_selection);

    // Find the selected book
    let selected_book = storage
        .books
        .values()
        .find(|b| b.title == title)
        .expect("Selected book not found");

    // Determine available actions based on book status
    let mut actions = Vec::new();

    // Check if book is currently being read
    let is_started = storage.is_book_started(&selected_book.id);
    let is_finished = storage.is_book_finished(&selected_book.id);

    // Check if book is marked as want to read using the proper method
    let is_want_to_read = storage
        .get_want_to_read_books()
        .iter()
        .any(|b| b.id == selected_book.id);

    // Check if book is already bought
    let is_bought = storage
        .get_readings_by_event(storage::ReadingEvent::Bought)
        .iter()
        .any(|r| r.book_id == selected_book.id);

    // Add appropriate actions based on book status
    if !is_started && !is_want_to_read {
        actions.push("Start reading");
        actions.push("Mark as want to read");
    } else if is_want_to_read {
        actions.push("Start reading");
        actions.push("Unmark as want to read");
    }

    if is_started && !is_finished {
        actions.push("Update progress");
        actions.push("Mark as finished");
    }

    if !is_bought {
        actions.push("Mark as bought");
    }

    if actions.is_empty() {
        println!("No available actions for this book.");
        return Ok(());
    }

    // Let user select an action
    let action_selection = match Select::new("Select an action:", actions).prompt() {
        Ok(selection) => selection,
        Err(_) => {
            println!("Operation cancelled");
            return Ok(());
        }
    };

    // Create and store the reading event
    let event = match action_selection {
        "Start reading" => storage::ReadingEvent::Started,
        "Mark as finished" => storage::ReadingEvent::Finished,
        "Update progress" => storage::ReadingEvent::Update,
        "Mark as bought" => storage::ReadingEvent::Bought,
        "Mark as want to read" => storage::ReadingEvent::WantToRead,
        "Unmark as want to read" => storage::ReadingEvent::UnmarkedAsWantToRead,
        _ => unreachable!(),
    };

    let reading = if event == storage::ReadingEvent::Update {
        let current_page = Text::new("Enter current page:")
            .prompt()
            .map_err(|e| format!("Failed to get current page: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid page number: {}", e))?;

        storage::Reading::with_metadata(selected_book.id.clone(), event, current_page)
    } else {
        storage::Reading::new(selected_book.id.clone(), event)
    };

    let mut storage = storage.clone();

    match reading::store_reading(&mut storage, reading) {
        Ok(_) => {
            storage::write_storage(storage_file, &storage)?;
            println!("Reading event added successfully!");
        }
        Err(e) => eprintln!("Failed to add reading event: {}", e),
    }

    Ok(())
}
