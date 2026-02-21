use bookmon::{
    book, config, goal,
    lookup::http_client,
    reading, review,
    storage::{self, Book, BookRepairInput, RepairPrompter, Storage},
};
use chrono::Datelike;
use clap::{Parser, Subcommand};
use inquire::{Select, Text};

/// Interactive prompter that uses `inquire` for user input during storage repair
struct InquirePrompter;

impl RepairPrompter for InquirePrompter {
    fn prompt_author_name(&self, book_title: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!(
            "Book '{}' references a missing author. Please provide the author name:",
            book_title
        );
        Ok(Text::new("Enter author name:")
            .prompt()
            .map_err(|e| format!("Failed to get author input: {}", e))?)
    }

    fn prompt_category_name(&self, book_title: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!(
            "Book '{}' references a missing category. Please provide the category name:",
            book_title
        );
        Ok(Text::new("Enter category name:")
            .prompt()
            .map_err(|e| format!("Failed to get category input: {}", e))?)
    }

    fn prompt_total_pages(&self, book_title: &str) -> Result<i32, Box<dyn std::error::Error>> {
        println!(
            "Book '{}' is missing total pages. Please provide the total pages:",
            book_title
        );
        let total_pages = Text::new("Enter total pages:")
            .prompt()
            .map_err(|e| format!("Failed to get total pages: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid total pages: {}", e))?;
        Ok(total_pages)
    }

    fn prompt_book_details(
        &self,
        reading_id: &str,
    ) -> Result<BookRepairInput, Box<dyn std::error::Error>> {
        println!(
            "Reading event {} references a missing book. Please provide the book details:",
            reading_id
        );

        let title = Text::new("Enter book title:")
            .prompt()
            .map_err(|e| format!("Failed to get book title: {}", e))?;

        let isbn = Text::new("Enter book ISBN:")
            .prompt()
            .map_err(|e| format!("Failed to get book ISBN: {}", e))?;

        let total_pages = Text::new("Enter total pages:")
            .prompt()
            .map_err(|e| format!("Failed to get total pages: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid total pages: {}", e))?;

        let author_name = Text::new("Enter author name:")
            .prompt()
            .map_err(|e| format!("Failed to get author name: {}", e))?;

        let category_name = Text::new("Enter category name:")
            .prompt()
            .map_err(|e| format!("Failed to get category name: {}", e))?;

        Ok(BookRepairInput {
            title,
            isbn,
            total_pages,
            author_name,
            category_name,
        })
    }
}

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
    /// Write a review for a book (opens $EDITOR)
    ReviewBook,
    /// Show all book reviews
    PrintReviews,
    /// Set a yearly reading goal (number of books to finish)
    SetGoal {
        /// Number of books to read
        target: u32,
        /// Year to set the goal for (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Show progress toward your reading goal
    PrintGoal {
        /// Year to check (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Show all book series and their books
    PrintSeries,
    /// Delete a series (books are kept but unlinked)
    DeleteSeries,
    /// Rename an existing series
    RenameSeries,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = config::Settings::load()?;
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
                return Err(
                    "Storage path not set. Please set it using the change-storage-path command."
                        .into(),
                );
            }
        }
    }

    // Initialize storage file if it doesn't exist
    storage::initialize_storage_file(&settings.storage_file)?;

    let mut storage = storage::load_and_repair_storage(&settings.storage_file, &InquirePrompter)?;

    // Handle commands (or default to showing currently-reading)
    if let Some(ref command) = cli.command {
        match command {
            Commands::AddBook => {
                match book::get_book_input(&mut storage) {
                    Ok((book, event)) => {
                        match book::store_book(&mut storage, book.clone()) {
                            Ok(_) => {
                                // Store all reading events
                                for event_type in event {
                                    let reading =
                                        storage::Reading::new(book.id.clone(), event_type);
                                    if let Err(e) = reading::store_reading(&mut storage, reading) {
                                        eprintln!("Failed to store reading event: {}", e);
                                    }
                                }

                                storage::write_storage(&settings.storage_file, &storage)?;
                                println!("Book added successfully!");
                            }
                            Err(e) => eprintln!("Failed to add book: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to get book input: {}", e),
                }
            }
            Commands::SetGoal { target, year } => {
                let year = year.unwrap_or_else(|| chrono::Utc::now().year());
                storage.set_goal(year, *target);
                storage::write_storage(&settings.storage_file, &storage)?;
                println!("Reading goal for {}: {} books", year, target);
            }
            Commands::PrintGoal { year } => {
                let year = year.unwrap_or_else(|| chrono::Utc::now().year());
                print_goal_status(&storage, year);
            }
            Commands::ReviewBook => {
                review_book_flow(&mut storage, &settings.storage_file)?;
            }
            Commands::PrintReviews => {
                if cli.interactive {
                    review_interactive_mode(&storage)?;
                } else {
                    match review::show_reviews(&storage) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to show reviews: {}", e),
                    }
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
                                        // Show goal progress if a goal is set for this year
                                        if let Some(target) = storage.get_goal(year) {
                                            let finished = books.len() as u32;
                                            let pct = goal_percentage(finished, target);
                                            let remaining = target.saturating_sub(finished);
                                            if year == current_year && remaining > 0 {
                                                println!(
                                                    "\n{}: {} books (Goal: {} \u{2014} {:.0}% complete, {} remaining)",
                                                    year,
                                                    books.len(),
                                                    target,
                                                    pct,
                                                    remaining
                                                );
                                            } else {
                                                println!(
                                                    "\n{}: {} books (Goal: {} \u{2014} {:.0}% complete)",
                                                    year,
                                                    books.len(),
                                                    target,
                                                    pct
                                                );
                                            }
                                        } else {
                                            println!("\n{}: {} books", year, books.len());
                                        }
                                        for book in books {
                                            let author_name = storage.author_name_for_book(book);
                                            let author_name = if author_name.is_empty() {
                                                "Unknown Author"
                                            } else {
                                                author_name
                                            };
                                            println!("  - \"{}\" by {}", book.title, author_name);
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
                let book =
                    tokio::runtime::Runtime::new()?.block_on(client.get_book_by_isbn(isbn))?;
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
                    if let Some(series_name) = book.series_name {
                        if let Some(pos) = book.series_position {
                            println!("Series: {} #{}", series_name, pos);
                        } else {
                            println!("Series: {}", series_name);
                        }
                    }
                } else {
                    println!("No book found for ISBN {}", isbn);
                }
            }
            Commands::PrintSeries => {
                print_series(&storage);
            }
            Commands::DeleteSeries => {
                delete_series_flow(&mut storage, &settings.storage_file)?;
            }
            Commands::RenameSeries => {
                rename_series_flow(&mut storage, &settings.storage_file)?;
            }
            Commands::ChangeStoragePath { .. } => unreachable!(),
        }
    } else {
        // Default case (no command) - show goal status + currently-reading
        show_goal_status_if_set(&storage);
        if cli.interactive {
            interactive_mode(&storage, &settings.storage_file, None)?;
        } else {
            match reading::show_started_books(&storage) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to show started books: {}", e),
            }
        }
    }

    Ok(())
}

/// Calculates the percentage of a reading goal completed.
fn goal_percentage(finished: u32, target: u32) -> f64 {
    if target > 0 {
        (finished as f64 / target as f64) * 100.0
    } else {
        100.0
    }
}

/// Prints a progress bar using Unicode block characters.
fn print_progress_bar(finished: u32, target: u32) {
    let bar_width = 20;
    let filled = if target > 0 {
        ((finished as f64 / target as f64) * bar_width as f64)
            .round()
            .min(bar_width as f64) as usize
    } else {
        bar_width
    };
    let empty = bar_width - filled;

    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
    print!("{}", bar);
}

/// Prints the reading goal status for a given year.
/// Shows book count, percentage, progress bar, remaining count, and motivational pace text.
fn print_goal_status(storage: &Storage, year: i32) {
    match storage.get_goal(year) {
        Some(target) => {
            let finished = storage.get_books_finished_in_year(year).len() as u32;
            let pct = goal_percentage(finished, target);
            let remaining = target.saturating_sub(finished);

            print!(
                "\nReading goal {}: {}/{} books ({:.0}%)\n",
                year, finished, target, pct
            );
            print_progress_bar(finished, target);
            if remaining > 0 {
                println!(" {} remaining", remaining);
            } else {
                println!(" Goal reached!");
            }
            if let Some(motivation) =
                goal::motivational_pace_text(finished, target, year, chrono::Utc::now())
            {
                println!("{}", motivation);
            }
            println!();
        }
        None => {
            println!(
                "No reading goal set for {}. Use `bookmon set-goal <number>` to set one.",
                year
            );
        }
    }
}

/// Prints the current year's goal status if one is set. Used by the default command.
fn show_goal_status_if_set(storage: &Storage) {
    let year = chrono::Utc::now().year();
    if storage.get_goal(year).is_some() {
        print_goal_status(storage, year);
    }
}

/// Prints all series and their books, sorted by series name then position.
fn print_series(storage: &Storage) {
    if storage.series.is_empty() {
        println!("No series found.");
        return;
    }

    // Sort series by name
    let mut all_series: Vec<&storage::Series> = storage.series.values().collect();
    all_series.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    for s in all_series {
        println!("\n{}", s.name);
        println!("{}", "-".repeat(s.name.len()));

        let books = storage.get_books_in_series(&s.id);
        if books.is_empty() {
            println!("  (no books)");
        } else {
            for book in books {
                let author_name = storage.author_name_for_book(book);
                let author_name = if author_name.is_empty() {
                    "Unknown Author"
                } else {
                    author_name
                };
                let pos = book
                    .position_in_series
                    .as_deref()
                    .map(|p| format!("#{} ", p))
                    .unwrap_or_default();
                println!("  {}\"{}\" by {}", pos, book.title, author_name);
            }
        }
    }
    println!();
}

/// Interactive flow to delete a series. Prompts the user to select which series to delete.
fn delete_series_flow(
    storage: &mut Storage,
    storage_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if storage.series.is_empty() {
        println!("No series to delete.");
        return Ok(());
    }

    let mut series_list: Vec<(&String, &storage::Series)> = storage.series.iter().collect();
    series_list.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));

    let display_names: Vec<String> = series_list
        .iter()
        .map(|(_, s)| {
            let count = storage.get_books_in_series(&s.id).len();
            format!("{} ({} books)", s.name, count)
        })
        .collect();

    let selection = match Select::new(
        "Select series to delete:",
        display_names.iter().map(|s| s.as_str()).collect(),
    )
    .prompt()
    {
        Ok(s) => s,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let idx = display_names
        .iter()
        .position(|s| s.as_str() == selection)
        .expect("selection from prompt must exist in display list");
    let series_id = series_list[idx].0.clone();
    let series_name = series_list[idx].1.name.clone();

    // Confirm deletion
    let confirm = match Select::new(
        &format!(
            "Are you sure you want to delete '{}'? Books will be kept but unlinked.",
            series_name
        ),
        vec!["Yes", "No"],
    )
    .prompt()
    {
        Ok(s) => s,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    if confirm == "No" {
        println!("Deletion cancelled.");
        return Ok(());
    }

    match bookmon::series::delete_series(storage, &series_id) {
        Ok(_) => {
            storage::write_storage(storage_file, storage)?;
            println!("Series '{}' deleted.", series_name);
        }
        Err(e) => eprintln!("Failed to delete series: {}", e),
    }

    Ok(())
}

/// Interactive flow to rename a series. Prompts the user to select which series to rename.
fn rename_series_flow(
    storage: &mut Storage,
    storage_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if storage.series.is_empty() {
        println!("No series to rename.");
        return Ok(());
    }

    let mut series_list: Vec<(&String, &storage::Series)> = storage.series.iter().collect();
    series_list.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));

    let names: Vec<&str> = series_list.iter().map(|(_, s)| s.name.as_str()).collect();

    let selection = match Select::new("Select series to rename:", names).prompt() {
        Ok(s) => s,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let idx = series_list
        .iter()
        .position(|(_, s)| s.name.as_str() == selection)
        .expect("selection from prompt must exist in series list");
    let series_id = series_list[idx].0.clone();

    let new_name = match Text::new("Enter new name:")
        .with_default(selection)
        .prompt()
    {
        Ok(n) => n,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let new_name = new_name.trim();
    if new_name.is_empty() {
        println!("Name cannot be empty.");
        return Ok(());
    }

    match bookmon::series::rename_series(storage, &series_id, new_name) {
        Ok(_) => {
            storage::write_storage(storage_file, storage)?;
            println!("Series renamed to '{}'.", new_name);
        }
        Err(e) => eprintln!("Failed to rename series: {}", e),
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

    // Create options for book selection with status, keeping book IDs
    let mut options: Vec<(String, String)> = Vec::new();
    for b in &filtered_books {
        let status = if storage.is_book_started(&b.id) {
            "Started"
        } else {
            "Not Started"
        };
        let display = b.to_display_string(storage, status)?;
        options.push((display, b.id.clone()));
    }

    // Sort options by:
    // 1. Reading status (Started first)
    // 2. Author name (from the book's author_id, not from parsing display string)
    // 3. Book title
    options.sort_by(|a, b| {
        let a_started = a.0.starts_with("[Started]");
        let b_started = b.0.starts_with("[Started]");

        if a_started != b_started {
            b_started.cmp(&a_started)
        } else {
            // Look up author names by book ID for robust sorting
            let a_book = storage.books.get(&a.1);
            let b_book = storage.books.get(&b.1);
            let a_author = a_book
                .map(|book| storage.author_name_for_book(book))
                .unwrap_or("");
            let b_author = b_book
                .map(|book| storage.author_name_for_book(book))
                .unwrap_or("");

            if a_author != b_author {
                a_author.cmp(b_author)
            } else {
                let a_title = a_book.map(|b| b.title.as_str()).unwrap_or("");
                let b_title = b_book.map(|b| b.title.as_str()).unwrap_or("");
                a_title.cmp(b_title)
            }
        }
    });

    // Build a mapping from display string → book ID
    let display_to_id: std::collections::HashMap<String, String> = options
        .iter()
        .map(|(display, id)| (display.clone(), id.clone()))
        .collect();

    let display_options: Vec<String> = options.into_iter().map(|(display, _)| display).collect();

    // Let user select a book
    let book_selection = match Select::new("Select a book to update:", display_options).prompt() {
        Ok(selection) => selection,
        Err(_) => {
            println!("Operation cancelled");
            return Ok(());
        }
    };

    // Find the selected book by ID (not by fragile title parsing)
    let selected_book_id = display_to_id
        .get(&book_selection)
        .ok_or("Selected book not found in display mapping")?;

    let selected_book = storage
        .books
        .get(selected_book_id)
        .ok_or_else(|| format!("Book with ID {} not found in storage", selected_book_id))?;

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

    // Series assignment is always available
    if selected_book.series_id.is_some() {
        actions.push("Change series");
    } else {
        actions.push("Assign to series");
    }

    // Review is always available for any book
    actions.push("Write review");

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

    // Handle "Assign to series" / "Change series" action
    if action_selection == "Assign to series" || action_selection == "Change series" {
        let mut storage = storage.clone();

        // Build series options
        let existing_series: Vec<(String, String)> = storage
            .series
            .iter()
            .map(|(id, s)| (s.name.clone(), id.clone()))
            .collect();

        let mut options: Vec<String> = Vec::new();
        let mut sorted_existing: Vec<&(String, String)> = existing_series.iter().collect();
        sorted_existing.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        for (name, _) in &sorted_existing {
            options.push(name.clone());
        }
        options.push("+ Create new series".to_string());
        if action_selection == "Change series" {
            options.push("Remove from series".to_string());
        }

        let selection = match Select::new(
            "Select series:",
            options.iter().map(|s| s.as_str()).collect(),
        )
        .prompt()
        {
            Ok(s) => s,
            Err(_) => {
                println!("Operation cancelled");
                return Ok(());
            }
        };

        if selection == "Remove from series" {
            if let Some(book) = storage.books.get_mut(selected_book_id) {
                book.series_id = None;
                book.position_in_series = None;
            }
            storage::write_storage(storage_file, &storage)?;
            println!("Removed from series.");
        } else {
            let series_id = if selection == "+ Create new series" {
                let name = match Text::new("Enter series name:").prompt() {
                    Ok(n) => n,
                    Err(_) => {
                        println!("Operation cancelled");
                        return Ok(());
                    }
                };
                bookmon::series::get_or_create_series(&mut storage, name.trim())
            } else {
                existing_series
                    .iter()
                    .find(|(name, _)| name.as_str() == selection)
                    .map(|(_, id)| id.clone())
                    .ok_or("Selected series not found")?
            };

            let position_str =
                match Text::new("Position in series (or leave empty to skip):").prompt() {
                    Ok(s) => s,
                    Err(_) => {
                        println!("Operation cancelled");
                        return Ok(());
                    }
                };
            let position = bookmon::series::parse_position_input(&position_str);

            if let Some(book) = storage.books.get_mut(selected_book_id) {
                book.series_id = Some(series_id);
                book.position_in_series = position;
            }
            storage::write_storage(storage_file, &storage)?;
            println!("Series assignment updated!");
        }

        return Ok(());
    }

    // Handle "Write review" action separately from reading events
    if action_selection == "Write review" {
        let author_name = storage.author_name_for_book(selected_book);
        let author_name = if author_name.is_empty() {
            "Unknown Author"
        } else {
            author_name
        };

        match review::get_review_text_from_editor(&selected_book.title, author_name) {
            Ok(Some(text)) => {
                let review_obj = storage::Review::new(selected_book.id.clone(), text);
                let mut storage = storage.clone();
                match review::store_review(&mut storage, review_obj) {
                    Ok(_) => {
                        storage::write_storage(storage_file, &storage)?;
                        println!("Review saved successfully!");
                    }
                    Err(e) => eprintln!("Failed to store review: {}", e),
                }
            }
            Ok(None) => {
                println!("Review aborted (empty text).");
            }
            Err(e) => eprintln!("Failed to get review text: {}", e),
        }
        return Ok(());
    }

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

/// Flow for the `review-book` command: select a book, open editor, save review.
fn review_book_flow(
    storage: &mut Storage,
    storage_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if storage.books.is_empty() {
        println!("No books in your collection. Add a book first.");
        return Ok(());
    }

    // Build sorted book list for selection
    let sorted_books = storage.sort_books();
    let mut options: Vec<(String, String)> = Vec::new();
    for book in &sorted_books {
        let author_name = storage.author_name_for_book(book);
        let display = format!("\"{}\" by {}", book.title, author_name);
        options.push((display, book.id.clone()));
    }

    let display_to_id: std::collections::HashMap<String, String> = options
        .iter()
        .map(|(d, id)| (d.clone(), id.clone()))
        .collect();
    let display_options: Vec<String> = options.into_iter().map(|(d, _)| d).collect();

    let selection = match Select::new("Select a book to review:", display_options).prompt() {
        Ok(s) => s,
        Err(_) => {
            println!("Operation cancelled");
            return Ok(());
        }
    };

    let book_id = display_to_id
        .get(&selection)
        .ok_or("Selected book not found")?
        .clone();
    let book = storage
        .books
        .get(&book_id)
        .ok_or("Book not found in storage")?;
    let author_name = storage.author_name_for_book(book);
    let author_name = if author_name.is_empty() {
        "Unknown Author"
    } else {
        author_name
    };
    let book_title = book.title.clone();

    match review::get_review_text_from_editor(&book_title, author_name) {
        Ok(Some(text)) => {
            let review_obj = storage::Review::new(book_id, text);
            match review::store_review(storage, review_obj) {
                Ok(_) => {
                    storage::write_storage(storage_file, storage)?;
                    println!("Review saved successfully!");
                }
                Err(e) => eprintln!("Failed to store review: {}", e),
            }
        }
        Ok(None) => {
            println!("Review aborted (empty text).");
        }
        Err(e) => eprintln!("Failed to get review text: {}", e),
    }

    Ok(())
}

/// Interactive mode for browsing reviews: select a review to view full text, loop.
fn review_interactive_mode(storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut reviews: Vec<&storage::Review> = storage.reviews.values().collect();

        if reviews.is_empty() {
            println!("No reviews found.");
            return Ok(());
        }

        // Sort by date, newest first
        reviews.sort_by(|a, b| b.created_on.cmp(&a.created_on));

        let mut options: Vec<(String, String)> = Vec::new();
        for r in &reviews {
            let book = storage.books.get(&r.book_id);
            let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
            let author = book
                .map(|b| storage.author_name_for_book(b))
                .unwrap_or("Unknown Author");
            let date = r.created_on.format("%Y-%m-%d").to_string();
            let preview: String = r.text.replace('\n', " ");
            let preview = if preview.chars().count() > 40 {
                let truncated: String = preview.chars().take(37).collect();
                format!("{}...", truncated)
            } else {
                preview
            };
            let display = format!("[{}] \"{}\" by {} - {}", date, title, author, preview);
            options.push((display, r.id.clone()));
        }

        let display_to_id: std::collections::HashMap<String, String> = options
            .iter()
            .map(|(d, id)| (d.clone(), id.clone()))
            .collect();
        let display_options: Vec<String> = options.into_iter().map(|(d, _)| d).collect();

        let selection =
            match Select::new("Select a review to view (Esc to quit):", display_options).prompt() {
                Ok(s) => s,
                Err(_) => {
                    return Ok(());
                }
            };

        let review_id = display_to_id
            .get(&selection)
            .ok_or("Selected review not found")?;

        review::show_review_detail(storage, review_id)?;
    }
}
