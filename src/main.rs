mod config;
use bookmon::factorial;
use bookmon::storage;

fn main() {
    let settings = config::Settings::load().expect("Failed to load config");
    println!("Starting {} in {} mode", settings.app_name, if settings.debug { "debug" } else { "release" });
    
    // Initialize storage file if it doesn't exist
    if let Err(e) = storage::initialize_storage_file(&settings.storage_file) {
        eprintln!("Failed to initialize storage file: {}", e);
        std::process::exit(1);
    }
    
    let result = factorial(5);
    println!("Factorial of 5 is: {}", result);
}
