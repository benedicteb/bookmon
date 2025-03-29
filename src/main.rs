mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage, book, category, author, reading};
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
    /// Change the storage file path
    ChangeStoragePath {
        /// The new path for the storage file
        path: String,
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
                                    storage::save_storage(&settings.storage_file, &storage)
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
                                    storage::save_storage(&settings.storage_file, &storage)
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
                                    storage::save_storage(&settings.storage_file, &storage)
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
                                    storage::save_storage(&settings.storage_file, &storage)
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
                Commands::ChangeStoragePath { .. } => unreachable!(),
            }
        }
        None => {
            // Interactive mode - show books and allow interaction
            let storage = storage::load_storage(&settings.storage_file)
                .expect("Failed to load storage");
            
            // Create options for book selection with status
            let options: Vec<String> = storage.books.iter()
                .filter(|(id, _)| !storage.is_book_finished(id))
                .map(|(_, b)| {
                    let author = storage.authors.get(&b.author_id)
                        .expect("Author not found");
                    let status = if storage.is_book_started(&b.id) {
                        "[Started]"
                    } else {
                        "[Not Started]"
                    };
                    format!("{} {} by {}", status, b.title, author.name)
                })
                .collect();

            if options.is_empty() {
                println!("No books available. Please add a book first.");
                return Ok(());
            }

            // Let user select a book
            match Select::new("Select a book to update:", options).prompt() {
                Ok(book_selection) => {
                    // Extract book title from selection (remove status and author)
                    let title = book_selection.split(" by ").next()
                        .unwrap()
                        .split("] ")
                        .nth(1)
                        .unwrap();

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
                                    storage::save_storage(&settings.storage_file, &storage)?;
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
