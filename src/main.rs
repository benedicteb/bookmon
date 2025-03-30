mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage::{self, Book}, book, category, author, reading, http_client};
use inquire::{Select, Text};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new book to the collection
    AddBook,
    /// Add a new category
    AddCategory,
    /// Add a new author
    AddAuthor,
    /// Add a reading event for a book
    AddReading,
    /// Show books that have been started but not finished
    CurrentlyReading,
    /// Show books that have been finished
    PrintFinished,
    /// Show books that have not been started yet
    PrintBacklog,
    /// Show all books in the library
    PrintAll,
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

    println!("Starting {} in {} mode", settings.app_name, if settings.debug { "debug" } else { "release" });
    
    // Initialize storage file if it doesn't exist
    if let Err(e) = storage::initialize_storage_file(&settings.storage_file) {
        eprintln!("Failed to initialize storage file: {}", e);
        std::process::exit(1);
    }

    match cli.command {
        Some(command) => {
            match command {
                Commands::AddBook => {
                    let mut storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match book::get_book_input(&mut storage) {
                        Ok(book) => {
                            match book::store_book(&mut storage, book) {
                                Ok(_) => {
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
                Commands::AddCategory => {
                    match category::get_category_input() {
                        Ok(category) => {
                            let mut storage = storage::load_storage(&settings.storage_file)
                                .expect("Failed to load storage");
                            
                            match category::store_category(&mut storage, category) {
                                Ok(_) => {
                                    storage::write_storage(&settings.storage_file, &storage)
                                        .expect("Failed to save storage");
                                    println!("Category added successfully!");
                                }
                                Err(e) => eprintln!("Failed to add category: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to get category input: {}", e),
                    }
                }
                Commands::AddAuthor => {
                    match author::get_author_input() {
                        Ok(author) => {
                            let mut storage = storage::load_storage(&settings.storage_file)
                                .expect("Failed to load storage");
                            
                            match author::store_author(&mut storage, author) {
                                Ok(_) => {
                                    storage::write_storage(&settings.storage_file, &storage)
                                        .expect("Failed to save storage");
                                    println!("Author added successfully!");
                                }
                                Err(e) => eprintln!("Failed to add author: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to get author input: {}", e),
                    }
                }
                Commands::AddReading => {
                    let storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match reading::get_reading_input(&storage) {
                        Ok(reading) => {
                            let mut storage = storage;
                            match reading::store_reading(&mut storage, reading) {
                                Ok(_) => {
                                    storage::write_storage(&settings.storage_file, &storage)
                                        .expect("Failed to save storage");
                                    println!("Reading event added successfully!");
                                }
                                Err(e) => eprintln!("Failed to add reading event: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to get reading input: {}", e),
                    }
                }
                Commands::CurrentlyReading => {
                    let storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match reading::show_started_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show started books: {}", e),
                    }
                }
                Commands::PrintFinished => {
                    let storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match reading::show_finished_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show finished books: {}", e),
                    }
                }
                Commands::PrintBacklog => {
                    let storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match reading::show_unstarted_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show unstarted books: {}", e),
                    }
                }
                Commands::PrintAll => {
                    let storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
                    match reading::show_all_books(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show all books: {}", e),
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
        }
        None => {
            // Interactive mode - show books and allow interaction
            let storage = storage::load_storage(&settings.storage_file)
                .expect("Failed to load storage");
            
            // Create options for book selection with status
            let mut options: Vec<(String, String)> = storage.books.iter()
                .filter(|(id, _)| !storage.is_book_finished(id))
                .map(|(_, b)| {
                    let status = if storage.is_book_started(&b.id) {
                        "Started"
                    } else {
                        "Not Started"
                    };
                    let display = b.to_display_string(&storage, status);
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

            if options.is_empty() {
                println!("No books available. Please add a book first.");
                return Ok(());
            }

            // Let user select a book
            match Select::new("Select a book to update:", options).prompt() {
                Ok(book_selection) => {
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

                    if actions.is_empty() {
                        println!("No available actions for this book.");
                        return Ok(());
                    }

                    // Let user select an action
                    match Select::new("Select an action:", actions).prompt() {
                        Ok(action_selection) => {
                            // Create and store the reading event
                            let event = match action_selection {
                                "Start reading" => storage::ReadingEvent::Started,
                                "Mark as finished" => storage::ReadingEvent::Finished,
                                "Update progress" => storage::ReadingEvent::Update,
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

                            let mut storage = storage;

                            match reading::store_reading(&mut storage, reading) {
                                Ok(_) => {
                                    storage::write_storage(&settings.storage_file, &storage)?;
                                    println!("Reading event added successfully!");
                                }
                                Err(e) => eprintln!("Failed to add reading event: {}", e),
                            }
                        }
                        Err(_) => println!("Operation cancelled"),
                    }
                }
                Err(_) => println!("Operation cancelled"),
            }
        }
    }

    Ok(())
}
