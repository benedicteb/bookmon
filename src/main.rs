mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage, book, category};

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
            let storage = storage::load_storage(&settings.storage_file)
                .expect("Failed to load storage");
            
            match book::get_book_input(&storage) {
                Ok(book) => {
                    let mut storage = storage;
                    
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
    }
}
