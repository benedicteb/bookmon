mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage, book, category, author, reading};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    GetStarted,
}

fn main() {
    let settings = config::Settings::load().expect("Failed to load config");
    println!("Starting {} in {} mode", settings.app_name, if settings.debug { "debug" } else { "release" });
    
    // Initialize storage file if it doesn't exist
    if let Err(e) = storage::initialize_storage_file(&settings.storage_file) {
        eprintln!("Failed to initialize storage file: {}", e);
        std::process::exit(1);
    }

    let cli = Cli::parse();

    match cli.command {
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
        Commands::GetStarted => {
            let storage = storage::load_storage(&settings.storage_file)
                .expect("Failed to load storage");
            
            match reading::show_started_books(&storage) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to show started books: {}", e),
            }
        }
    }
}
