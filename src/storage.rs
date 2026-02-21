use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use serde_json::Map;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub added_on: DateTime<Utc>,
    pub isbn: String,
    pub category_id: String,
    pub author_id: String,
    #[serde(default)]
    pub total_pages: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ReadingEvent {
    Finished,
    Started,
    Update,
    Bought,
    WantToRead,
    UnmarkedAsWantToRead,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReadingMetadata {
    #[serde(default)]
    pub current_page: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: ReadingEvent,
    #[serde(default)]
    pub metadata: ReadingMetadata,
}

impl Author {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_on: Utc::now(),
        }
    }
}

impl Book {
    pub fn new(
        title: String,
        isbn: String,
        category_id: String,
        author_id: String,
        total_pages: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            added_on: Utc::now(),
            isbn,
            category_id,
            author_id,
            total_pages,
        }
    }

    /// Creates a display string for a book with its status and author name
    pub fn to_display_string(&self, storage: &Storage, status: &str) -> Result<String, String> {
        let author = storage
            .authors
            .get(&self.author_id)
            .ok_or_else(|| format!("Author with ID {} not found", self.author_id))?;
        Ok(format!(
            "[{}] \"{}\" by {}",
            status, self.title, author.name
        ))
    }

    /// Extracts a book title from a display string formatted as `[Status] "Title" by Author`
    ///
    /// Handles titles that contain " by " by finding the quoted title between the first
    /// pair of double quotes after the status bracket.
    pub fn title_from_display_string(display: &str) -> Result<String, String> {
        // Find the first '"' after '] '
        let after_bracket = display
            .find("] \"")
            .ok_or_else(|| format!("Invalid display string format: {}", display))?;
        let title_start = after_bracket + 3; // skip '] "'

        // Find the closing '"' before ' by ' — search from the end for the last '" by '
        let remaining = &display[title_start..];
        let title_end = remaining
            .rfind("\" by ")
            .ok_or_else(|| format!("Invalid display string format: {}", display))?;

        Ok(remaining[..title_end].to_string())
    }
}

impl Reading {
    pub fn new(book_id: String, event: ReadingEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata { current_page: None },
        }
    }

    pub fn with_metadata(book_id: String, event: ReadingEvent, current_page: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata {
                current_page: Some(current_page),
            },
        }
    }
}

impl Category {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_on: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Storage {
    pub books: HashMap<String, Book>,
    pub readings: HashMap<String, Reading>,
    pub authors: HashMap<String, Author>,
    pub categories: HashMap<String, Category>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            books: HashMap::new(),
            readings: HashMap::new(),
            authors: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Converts the storage to a sorted JSON string
    pub fn to_sorted_json_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json_value = serde_json::to_value(self)?;
        let sorted_value = sort_json_value(json_value);
        Ok(serde_json::to_string_pretty(&sorted_value)?)
    }

    pub fn add_book(&mut self, book: Book) -> Option<Book> {
        self.books.insert(book.id.clone(), book)
    }

    pub fn add_reading(&mut self, reading: Reading) -> Option<Reading> {
        self.readings.insert(reading.id.clone(), reading)
    }

    pub fn add_author(&mut self, author: Author) -> Option<Author> {
        self.authors.insert(author.id.clone(), author)
    }

    pub fn add_category(&mut self, category: Category) -> Option<Category> {
        self.categories.insert(category.id.clone(), category)
    }

    /// Returns the author name for a given book, or an empty string if the author is not found
    pub fn author_name_for_book(&self, book: &Book) -> &str {
        self.authors
            .get(&book.author_id)
            .map(|a| a.name.as_str())
            .unwrap_or("")
    }

    pub fn get_book(&self, id: &str) -> Option<&Book> {
        self.books.get(id)
    }

    pub fn get_reading(&self, id: &str) -> Option<&Reading> {
        self.readings.get(id)
    }

    pub fn get_author(&self, id: &str) -> Option<&Author> {
        self.authors.get(id)
    }

    pub fn get_category(&self, id: &str) -> Option<&Category> {
        self.categories.get(id)
    }

    pub fn get_readings_by_event(&self, event_type: ReadingEvent) -> Vec<&Reading> {
        self.readings
            .values()
            .filter(|r| r.event == event_type)
            .collect()
    }

    pub fn get_unstarted_books(&self) -> Vec<&Book> {
        // Get all book IDs that have either started or finished readings
        let started_or_finished: std::collections::HashSet<String> = self
            .readings
            .iter()
            .filter(|(_, r)| matches!(r.event, ReadingEvent::Started | ReadingEvent::Finished))
            .map(|(_, r)| r.book_id.clone())
            .collect();

        // Find books that have no started or finished readings
        self.books
            .values()
            .filter(|book| !started_or_finished.contains(&book.id))
            .collect()
    }

    pub fn get_started_books(&self) -> Vec<&Book> {
        // Group readings by book_id
        let mut book_readings: HashMap<String, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            book_readings
                .entry(reading.book_id.clone())
                .or_default()
                .push(reading);
        }

        // Filter books to only those that are currently being read
        self.books
            .values()
            .filter(|book| {
                if let Some(readings) = book_readings.get(&book.id) {
                    // Sort readings by created_on in descending order
                    let mut sorted_readings = readings.clone();
                    sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

                    // Find the most recent Started event that isn't followed by a Finished event
                    for reading in sorted_readings {
                        match reading.event {
                            ReadingEvent::Started => return true,
                            ReadingEvent::Finished => return false,
                            ReadingEvent::Update => continue,
                            ReadingEvent::Bought => continue,
                            ReadingEvent::WantToRead => continue,
                            ReadingEvent::UnmarkedAsWantToRead => continue,
                        }
                    }
                    false
                } else {
                    false
                }
            })
            .collect()
    }

    /// Helper method to get books with a specific event as their most recent reading
    pub fn get_books_by_most_recent_event(&self, target_event: ReadingEvent) -> Vec<&Book> {
        // Group readings by book_id
        let mut book_readings: HashMap<String, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            book_readings
                .entry(reading.book_id.clone())
                .or_default()
                .push(reading);
        }

        // Filter books to only those that have the target event as their most recent reading
        self.books
            .values()
            .filter(|book| {
                if let Some(readings) = book_readings.get(&book.id) {
                    // Sort readings by created_on in descending order
                    let mut sorted_readings = readings.clone();
                    sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

                    // Check if the most recent reading matches the target event
                    if let Some(most_recent) = sorted_readings.first() {
                        most_recent.event == target_event
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_finished_books(&self) -> Vec<&Book> {
        self.get_books_by_most_recent_event(ReadingEvent::Finished)
    }

    pub fn get_bought_books(&self) -> Vec<&Book> {
        self.get_books_by_most_recent_event(ReadingEvent::Bought)
    }

    pub fn get_want_to_read_books(&self) -> Vec<&Book> {
        self.get_books_by_most_recent_event(ReadingEvent::WantToRead)
    }

    /// Returns books that are currently being read or marked as want to read
    pub fn get_currently_reading_and_want_to_read_books(&self) -> Vec<&Book> {
        // Get books that are currently being read
        let started_books = self.get_started_books();

        // Get books that are marked as want to read
        let want_to_read_books = self.get_want_to_read_books();

        // Combine the two lists, ensuring no duplicates
        let mut result = Vec::new();
        let mut book_ids = std::collections::HashSet::new();

        for book in started_books {
            book_ids.insert(book.id.clone());
            result.push(book);
        }

        for book in want_to_read_books {
            if !book_ids.contains(&book.id) {
                book_ids.insert(book.id.clone());
                result.push(book);
            }
        }

        result
    }

    pub fn is_book_started(&self, book_id: &str) -> bool {
        let readings: Vec<_> = self
            .readings
            .values()
            .filter(|r| r.book_id == book_id)
            .collect();

        if !readings.is_empty() {
            let mut sorted_readings = readings;
            sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

            // Check if there's a Started event that isn't followed by a Finished event
            for reading in sorted_readings {
                match reading.event {
                    ReadingEvent::Started => return true,
                    ReadingEvent::Finished => return false,
                    ReadingEvent::Update => continue,
                    ReadingEvent::Bought => continue,
                    ReadingEvent::WantToRead => continue,
                    ReadingEvent::UnmarkedAsWantToRead => continue,
                }
            }
            false
        } else {
            false
        }
    }

    pub fn is_book_finished(&self, book_id: &str) -> bool {
        let readings: Vec<_> = self
            .readings
            .values()
            .filter(|r| r.book_id == book_id)
            .collect();

        if !readings.is_empty() {
            let mut sorted_readings = readings;
            sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));
            if let Some(most_recent) = sorted_readings.first() {
                most_recent.event == ReadingEvent::Finished
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Sorts books by reading status, author name, and title
    pub fn sort_books(&self) -> Vec<&Book> {
        let mut books: Vec<&Book> = self.books.values().collect();
        books.sort_by(|a, b| {
            // First sort by reading status
            let a_status = if self.is_book_started(&a.id) {
                0 // Currently reading
            } else if self.is_book_finished(&a.id) {
                2 // Finished
            } else {
                1 // Not started
            };
            let b_status = if self.is_book_started(&b.id) {
                0 // Currently reading
            } else if self.is_book_finished(&b.id) {
                2 // Finished
            } else {
                1 // Not started
            };

            if a_status != b_status {
                a_status.cmp(&b_status)
            } else {
                // Then sort by author name, then by title
                let a_author_name = self.author_name_for_book(a);
                let b_author_name = self.author_name_for_book(b);

                if a_author_name != b_author_name {
                    a_author_name.cmp(b_author_name)
                } else {
                    a.title.cmp(&b.title)
                }
            }
        });
        books
    }

    /// Returns all books that were finished reading within the given time period
    pub fn get_read_books_by_time_period(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&Book> {
        // Get all finished readings within the time period
        let finished_readings: Vec<&Reading> = self
            .readings
            .values()
            .filter(|r| {
                r.event == ReadingEvent::Finished && r.created_on >= from && r.created_on <= to
            })
            .collect();

        // Get the corresponding books
        finished_readings
            .iter()
            .filter_map(|reading| self.books.get(&reading.book_id))
            .collect()
    }

    /// Returns the earliest year in which a book was finished
    pub fn get_earliest_finished_year(&self) -> Option<i32> {
        self.readings
            .values()
            .filter(|r| r.event == ReadingEvent::Finished)
            .map(|r| r.created_on.year())
            .min()
    }

    /// Returns all books that were finished in a specific year
    pub fn get_books_finished_in_year(&self, year: i32) -> Vec<&Book> {
        let from = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(year, 12, 31, 23, 59, 59).unwrap();
        self.get_read_books_by_time_period(from, to)
    }
}

pub fn sort_json_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = BTreeMap::new();
            for (k, v) in map {
                sorted_map.insert(k, sort_json_value(v));
            }
            Value::Object(Map::from_iter(sorted_map))
        }
        Value::Array(vec) => Value::Array(vec.into_iter().map(sort_json_value).collect()),
        _ => value,
    }
}

/// Writes the storage to a file, creating the file and parent directories if they don't exist
pub fn write_storage(
    storage_path: &str,
    storage: &Storage,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the storage data using the new method
    fs::write(path, storage.to_sorted_json_string()?)?;

    Ok(())
}

pub fn initialize_storage_file(storage_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);

    if !path.exists() {
        let initial_storage = Storage::new();
        write_storage(storage_path, &initial_storage)?;
    }

    Ok(())
}

/// A trait for providing user input during storage repair operations.
/// This separates the UI concern from the data layer, making it testable.
pub trait RepairPrompter {
    fn prompt_author_name(&self, book_title: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn prompt_category_name(&self, book_title: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn prompt_total_pages(&self, book_title: &str) -> Result<i32, Box<dyn std::error::Error>>;
    fn prompt_book_details(
        &self,
        reading_id: &str,
    ) -> Result<BookRepairInput, Box<dyn std::error::Error>>;
}

/// Input data needed to repair a missing book reference
pub struct BookRepairInput {
    pub title: String,
    pub isbn: String,
    pub total_pages: i32,
    pub author_name: String,
    pub category_name: String,
}

pub fn handle_missing_fields(
    storage: &mut Storage,
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<(), Box<dyn std::error::Error>> {
    // First, collect all missing references
    let mut missing_authors: Vec<(String, String)> = Vec::new(); // (book_id, book_title)
    let mut missing_categories: Vec<(String, String)> = Vec::new(); // (book_id, book_title)
    let mut missing_books: Vec<(String, String)> = Vec::new(); // (reading_id, book_id)
    let mut books_missing_fields: Vec<String> = Vec::new(); // book_ids

    // Check books for missing fields and references
    for (book_id, book) in storage.books.iter() {
        if !storage.authors.contains_key(&book.author_id) {
            missing_authors.push((book_id.clone(), book.title.clone()));
        }
        if !storage.categories.contains_key(&book.category_id) {
            missing_categories.push((book_id.clone(), book.title.clone()));
        }
        if book.total_pages <= 0 {
            books_missing_fields.push(book_id.clone());
        }
    }

    // Check readings for missing book references
    for (reading_id, reading) in storage.readings.iter() {
        if !storage.books.contains_key(&reading.book_id) {
            missing_books.push((reading_id.clone(), reading.book_id.clone()));
        }
    }

    // Handle missing authors — create new author AND update book's author_id
    for (book_id, book_title) in missing_authors {
        let author_name = prompter.prompt_author_name(&book_title)?;

        let author = Author::new(author_name.trim().to_string());
        let new_author_id = author.id.clone();
        storage.add_author(author);

        // Update the book's author_id to point to the new author
        if let Some(book) = storage.books.get_mut(&book_id) {
            book.author_id = new_author_id;
        }

        // Save after each fix
        write_storage(storage_path, storage)?;
    }

    // Handle missing categories — create new category AND update book's category_id
    for (book_id, book_title) in missing_categories {
        let category_name = prompter.prompt_category_name(&book_title)?;

        let category = Category::new(category_name.trim().to_string(), None);
        let new_category_id = category.id.clone();
        storage.add_category(category);

        // Update the book's category_id to point to the new category
        if let Some(book) = storage.books.get_mut(&book_id) {
            book.category_id = new_category_id;
        }

        // Save after each fix
        write_storage(storage_path, storage)?;
    }

    // Handle books with missing fields
    for book_id in books_missing_fields {
        let book_title = storage
            .books
            .get(&book_id)
            .map(|b| b.title.clone())
            .unwrap_or_default();

        let total_pages = prompter.prompt_total_pages(&book_title)?;

        if let Some(book) = storage.books.get_mut(&book_id) {
            book.total_pages = total_pages;
        }

        // Save after each book's total_pages is updated
        write_storage(storage_path, storage)?;
    }

    // Handle missing books — create book with new author and category
    for (reading_id, _book_id) in missing_books {
        let input = prompter.prompt_book_details(&reading_id)?;

        // Create author
        let author = Author::new(input.author_name.trim().to_string());
        let author_id = author.id.clone();
        storage.add_author(author);

        // Create category
        let category = Category::new(input.category_name.trim().to_string(), None);
        let category_id = category.id.clone();
        storage.add_category(category);

        // Create and add the book
        let book = Book::new(
            input.title.trim().to_string(),
            input.isbn.trim().to_string(),
            category_id,
            author_id,
            input.total_pages,
        );
        storage.add_book(book);

        // Save after book is added
        write_storage(storage_path, storage)?;
    }

    Ok(())
}

pub fn load_storage(storage_path: &str) -> Result<Storage, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let storage: Storage = serde_json::from_str(&contents)?;
    Ok(storage)
}

/// Loads storage and repairs any missing references using the given prompter
pub fn load_and_repair_storage(
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<Storage, Box<dyn std::error::Error>> {
    let mut storage = load_storage(storage_path)?;
    handle_missing_fields(&mut storage, storage_path, prompter)?;
    Ok(storage)
}
