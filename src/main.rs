mod config;
use clap::{Parser, Subcommand};
use bookmon::{storage, book};

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
            match book::get_book_input() {
                Ok(book) => {
                    let mut storage = storage::load_storage(&settings.storage_file)
                        .expect("Failed to load storage");
                    
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
    }
}
