mod config;
use bookmon::factorial;

fn main() {
    let settings = config::Settings::load().expect("Failed to load config");
    println!("Starting {} in {} mode", settings.app_name, if settings.debug { "debug" } else { "release" });
    
    let result = factorial(5);
    println!("Factorial of 5 is: {}", result);
}
