mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage::{self, Book, Storage}, book, category, author, reading, http_client};
use inquire::{Select, Text};
use pretty_table::prelude::*;

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
    /// Show all books in the library
    PrintAll,
    /// Show books that have been bought
    PrintBought,
    /// Show books that are in the want to read list
    PrintWantToRead,
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

    let mut storage = storage::load_storage(&settings.storage_file)
        .expect("Failed to load storage");

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
                            // If there's an event, store it
                            if let Some(event_type) = event {
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
        cmd @ (Commands::PrintFinished | Commands::PrintBacklog | Commands::PrintAll | Commands::PrintBought | Commands::PrintWantToRead) => {
            if cli.interactive {
                interactive_mode(&storage, &settings.storage_file, Some(cmd))?;
            } else {
                match cmd {
                    Commands::PrintFinished => {
                        match reading::show_finished_books(&storage) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Failed to show finished books: {}", e),
                        }
                    }
                    Commands::PrintBacklog => {
                        match reading::show_unstarted_books(&storage) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Failed to show unstarted books: {}", e),
                        }
                    }
                    Commands::PrintAll => {
                        match reading::show_all_books(&storage) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Failed to show all books: {}", e),
                        }
                    }
                    Commands::PrintBought => {
                        let bought_books = storage.get_bought_books();
                        if bought_books.is_empty() {
                            println!("No bought books found.");
                        } else {
                            // Create table data
                            let mut table_data = vec![
                                vec!["Title".to_string(), "Author".to_string(), "Category".to_string()], // header
                            ];

                            // Sort the bought books by author and title
                            let mut sorted_books = bought_books;
                            sorted_books.sort_by(|a, b| {
                                let a_author = storage.authors.get(&a.author_id).unwrap();
                                let b_author = storage.authors.get(&b.author_id).unwrap();
                                
                                if a_author.name != b_author.name {
                                    a_author.name.cmp(&b_author.name)
                                } else {
                                    a.title.cmp(&b.title)
                                }
                            });

                            // For each bought book, find the corresponding author and category
                            for book in sorted_books {
                                let author = storage.authors.get(&book.author_id)
                                    .expect("Author not found");
                                
                                let category = storage.categories.get(&book.category_id)
                                    .expect("Category not found");

                                // Add row to table data
                                table_data.push(vec![
                                    book.title.clone(),
                                    author.name.clone(),
                                    category.name.clone()
                                ]);
                            }

                            // Print the table
                            print_table!(table_data);
                        }
                    }
                    Commands::PrintWantToRead => {
                        let want_to_read_books = storage.get_want_to_read_books();
                        if want_to_read_books.is_empty() {
                            println!("No books in want to read list.");
                        } else {
                            // Create table data
                            let mut table_data = vec![
                                vec!["Title".to_string(), "Author".to_string(), "Category".to_string()], // header
                            ];

                            // Sort the want to read books by author and title
                            let mut sorted_books = want_to_read_books;
                            sorted_books.sort_by(|a, b| {
                                let a_author = storage.authors.get(&a.author_id).unwrap();
                                let b_author = storage.authors.get(&b.author_id).unwrap();
                                
                                if a_author.name != b_author.name {
                                    a_author.name.cmp(&b_author.name)
                                } else {
                                    a.title.cmp(&b.title)
                                }
                            });

                            // For each want to read book, find the corresponding author and category
                            for book in sorted_books {
                                let author = storage.authors.get(&book.author_id)
                                    .expect("Author not found");
                                
                                let category = storage.categories.get(&book.category_id)
                                    .expect("Category not found");

                                // Add row to table data
                                table_data.push(vec![
                                    book.title.clone(),
                                    author.name.clone(),
                                    category.name.clone()
                                ]);
                            }

                            // Print the table
                            print_table!(table_data);
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
            match tokio::runtime::Runtime::new()?.block_on(client.get_book_by_isbn(&isbn)) {
                Ok(book) => {
                    println!("Book Information:");
                    println!("Title: {}", book.title);
                    println!("Authors:");
                    for author in book.authors {
                        println!("  - {}", author.name.unwrap_or_else(|| "Unknown".to_string()));
                        if let Some(personal_name) = author.personal_name {
                            println!("    Personal Name: {}", personal_name);
                        }
                        if let Some(birth_date) = author.birth_date {
                            println!("    Born: {}", birth_date);
                        }
                        if let Some(death_date) = author.death_date {
                            println!("    Died: {}", death_date);
                        }
                        if let Some(bio) = author.bio {
                            println!("    Bio: {}", bio);
                        }
                    }
                    if let Some(publish_date) = book.first_publish_date {
                        println!("First Published: {}", publish_date);
                    }
                    if let Some(description) = book.description {
                        println!("Description: {}", description);
                    }
                    if let Some(covers) = book.covers {
                        let cover_strings: Vec<String> = covers.iter().map(|id| id.to_string()).collect();
                        println!("Cover IDs: {}", cover_strings.join(", "));
                    }
                }
                Err(e) => eprintln!("Failed to fetch book information: {}", e),
            }
        }
        Commands::ChangeStoragePath { .. } => unreachable!(),
    }

    Ok(())
}

// Helper function for interactive mode
fn interactive_mode(storage: &Storage, storage_file: &str, command: Option<&Commands>) -> Result<(), Box<dyn std::error::Error>> {
    // Get the appropriate books based on the command
    let filtered_books: Vec<&Book> = match command {
        None => storage.get_started_books(), // Default case - currently reading
        Some(cmd) => match cmd {
            Commands::PrintFinished => storage.get_finished_books(),
            Commands::PrintBacklog => storage.get_unstarted_books(),
            Commands::PrintAll => storage.books.values().collect(),
            Commands::PrintBought => storage.get_bought_books(),
            Commands::PrintWantToRead => storage.get_want_to_read_books(),
            _ => storage.get_started_books(), // Fallback to currently reading
        },
    };

    if filtered_books.is_empty() {
        println!("No books available in this category.");
        return Ok(());
    }

    // Create options for book selection with status
    let mut options: Vec<(String, String)> = filtered_books.into_iter()
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
    let selected_book = storage.books.values()
        .find(|b| b.title == title)
        .expect("Selected book not found");

    // Determine available actions based on book status
    let mut actions = Vec::new();
    if !storage.is_book_started(&selected_book.id) {
        actions.push("Start reading");
    }
    if storage.is_book_started(&selected_book.id) && !storage.is_book_finished(&selected_book.id) {
        actions.push("Update progress");
        actions.push("Mark as finished");
    }
    
    // Check if book is not already bought
    let is_bought = storage.get_readings_by_event(storage::ReadingEvent::Bought)
        .iter()
        .any(|r| r.book_id == selected_book.id);
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
