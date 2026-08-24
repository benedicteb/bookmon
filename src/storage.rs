use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use serde_json::Map;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// A book author with a unique ID and creation timestamp.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub created_on: DateTime<Utc>,
}

/// A book category (e.g. "Fiction", "Science") with optional description.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_on: DateTime<Utc>,
}

/// Whether a book series is still being published or has concluded.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum SeriesStatus {
    /// Series is still being published (new books expected).
    Ongoing,
    /// Series is complete (no more books expected).
    Completed,
    /// Series was abandoned (author died, publisher cancelled, etc.).
    Abandoned,
}

/// A book series (e.g. "Harry Potter", "Lord of the Rings").
/// Books can optionally belong to a series with a position number.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub created_on: DateTime<Utc>,
    /// Whether the series is ongoing, completed, or abandoned.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SeriesStatus>,
    /// Known total number of books in the series (if known).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_books: Option<u32>,
}

/// A book in the collection, linked to an author and category by ID.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub added_on: DateTime<Utc>,
    pub isbn: String,
    pub category_id: String,
    pub author_id: String,
    /// Total number of pages in the book.
    /// NOTE: Uses i32 (not u32) for serde compatibility with existing JSON data.
    /// Values <= 0 are treated as "unknown" and trigger repair prompts on load.
    #[serde(default)]
    pub total_pages: i32,
    /// Optional FK -> Series.id. None means the book is standalone (not part of a series).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_id: Option<String>,
    /// Optional position within the series (e.g. 1, 2, or 0 for a prequel).
    /// Non-negative whole numbers only; books without a position sort last.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_position")]
    pub position_in_series: Option<i32>,
}

/// Parses a position value that must be a non-negative integer.
///
/// Accepts `"3"` and the lossless decimal form `"3.0"`. Rejects fractional
/// values, negatives, non-finite values and anything non-numeric — these are
/// exactly the values the interactive migration prompts about, and the two
/// must not disagree.
pub fn parse_integral_position(raw: &str) -> Option<i32> {
    let value: f64 = raw.trim().parse().ok()?;
    if !value.is_finite() || value.fract() != 0.0 || value < 0.0 {
        return None;
    }
    if value > i32::MAX as f64 {
        return None;
    }
    Some(value as i32)
}

/// Custom deserializer for `position_in_series`.
///
/// Absent, `null` and `""` all mean "no position". A JSON number and a numeric
/// string are both accepted when integral — the number form is the pre-`efca310`
/// legacy shape, the string form is the shape written between then and the move
/// to `i32`. Anything else is an error rather than a silent coercion, so that no
/// unmigrated value can reach the program as an altered integer.
fn deserialize_position<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    let raw = match value {
        None | Some(serde_json::Value::Null) => return Ok(None),
        Some(serde_json::Value::String(s)) => {
            if s.trim().is_empty() {
                return Ok(None);
            }
            s
        }
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    };

    parse_integral_position(&raw).map(Some).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "invalid position_in_series {:?}: must be a non-negative whole number. \
             Run `bookmon` to migrate this file interactively.",
            raw
        ))
    })
}

/// One book whose stored position is not a valid non-negative integer.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPositionFix {
    pub book_id: String,
    pub title: String,
    /// `None` when the book has no series, or a `series_id` that does not resolve.
    /// Such a book is cleared silently rather than prompted about, because
    /// `handle_missing_fields` clears the positions of orphaned books anyway.
    pub series_id: Option<String>,
    pub series_name: String,
    /// The offending value as written in the file, shown to the user.
    pub raw: String,
    pub suggested: Option<i32>,
}

/// Renders a JSON position value the way it should be shown to a user:
/// strings without their quotes, everything else as written.
fn raw_position_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Suggests an integer slot for a legacy position.
///
/// Uses the ceiling, clamped at 0: a `2.5` sorts after book 2, so slot 3 (with
/// everything from 3 up shifted by one) preserves the original reading order.
/// Non-numeric values have no anchor, so they get no suggestion.
fn suggest_position(raw: &str) -> Option<i32> {
    let value: f64 = raw.trim().parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    let ceiling = value.ceil().max(0.0);
    if ceiling > i32::MAX as f64 {
        return None;
    }
    Some(ceiling as i32)
}

/// Finds every book whose `position_in_series` is not a valid non-negative integer.
///
/// Operates on raw JSON because these values cannot be deserialized into `Book` —
/// that is the whole reason the migration runs before deserialization ever happens.
/// The "needs fixing" predicate here must exactly match the deserializer's reject
/// set (`parse_integral_position`), or a file could end up permanently unloadable
/// with no in-app way to fix it.
pub fn scan_invalid_positions(value: &serde_json::Value) -> Vec<PendingPositionFix> {
    let series = value.get("series");
    let books = match value.get("books").and_then(|b| b.as_object()) {
        Some(books) => books,
        None => return Vec::new(),
    };

    let mut fixes = Vec::new();
    for (book_id, book) in books {
        let position = match book.get("position_in_series") {
            None | Some(serde_json::Value::Null) => continue,
            Some(position) => position,
        };

        let raw = raw_position_text(position);
        if raw.trim().is_empty() || parse_integral_position(&raw).is_some() {
            continue;
        }

        // Resolve the series; an absent or dangling id means "clear, don't ask".
        let series_id = book.get("series_id").and_then(|s| s.as_str());
        let resolved = series_id.zip(series_id.and_then(|sid| series.and_then(|s| s.get(sid))));

        fixes.push(PendingPositionFix {
            book_id: book_id.clone(),
            title: book
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string(),
            series_id: resolved.map(|(sid, _)| sid.to_string()),
            series_name: resolved
                .and_then(|(_, s)| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or_default()
                .to_string(),
            suggested: suggest_position(&raw),
            raw,
        });
    }

    // Group by series, then by suggested slot, so the user is asked about a series'
    // books in reading order rather than in HashMap order.
    fixes.sort_by(|a, b| {
        a.series_name
            .cmp(&b.series_name)
            .then(a.suggested.cmp(&b.suggested))
            .then(a.book_id.cmp(&b.book_id))
    });
    fixes
}

/// Returns the positions already validly occupied in a series, read from raw JSON.
pub fn taken_positions(value: &serde_json::Value, series_id: &str) -> Vec<i32> {
    let books = match value.get("books").and_then(|b| b.as_object()) {
        Some(books) => books,
        None => return Vec::new(),
    };

    books
        .values()
        .filter(|book| book.get("series_id").and_then(|s| s.as_str()) == Some(series_id))
        .filter_map(|book| book.get("position_in_series"))
        .filter_map(|position| parse_integral_position(&raw_position_text(position)))
        .collect()
}

/// The type of reading event recorded for a book.
///
/// The most recent status-bearing event determines the book's current status.
/// Progress updates (`Update`) don't participate in status determination — they record
/// how far the reader got but don't change whether the book is started, finished, or owned.
/// `Abandoned` ends a read-through without finishing: the book is no longer
/// being read, but a later `Started` begins a fresh attempt.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ReadingEvent {
    Finished,
    Started,
    Update,
    Bought,
    WantToRead,
    UnmarkedAsWantToRead,
    Abandoned,
    /// The first review written for a book. At most one per book.
    CreateReview,
    /// A revision of an existing review. Carries the full revised text.
    EditReview,
}

impl ReadingEvent {
    /// Whether this event participates in determining a book's current status.
    ///
    /// Progress updates say how far the reader got, not whether the book is
    /// started or finished, so they must not displace the last real status
    /// event when the most recent event is looked up.
    pub fn affects_status(self) -> bool {
        match self {
            ReadingEvent::Started
            | ReadingEvent::Finished
            | ReadingEvent::WantToRead
            | ReadingEvent::UnmarkedAsWantToRead
            | ReadingEvent::Bought
            | ReadingEvent::Abandoned => true,
            ReadingEvent::Update | ReadingEvent::CreateReview | ReadingEvent::EditReview => false,
        }
    }
}

/// Optional metadata attached to a reading event.
///
/// `current_page` records progress for `Update` events. `note` holds the user's
/// free-text remarks about that progress, written in their editor.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReadingMetadata {
    /// Both fields are omitted from the JSON entirely when absent, so events
    /// that carry no progress data are not written with null keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_page: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The complete review text as of this event, for `CreateReview` and
    /// `EditReview`. A full snapshot, not a patch — see ADR 0017.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_text: Option<String>,
}

impl ReadingMetadata {
    /// True when no metadata is set. Lets `Reading` omit the whole object for
    /// events that carry none, rather than writing an empty `{}`.
    pub fn is_empty(&self) -> bool {
        self.current_page.is_none() && self.note.is_none() && self.review_text.is_none()
    }
}

/// A timestamped reading event for a book (event-sourcing pattern).
///
/// Each reading records a single event (Started, Finished, Update, etc.)
/// and is linked to a book by `book_id`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: ReadingEvent,
    /// Omitted entirely when empty, so non-progress events (Started, Finished,
    /// WantToRead, Bought) keep the shape they had before metadata existed.
    #[serde(default, skip_serializing_if = "ReadingMetadata::is_empty")]
    pub metadata: ReadingMetadata,
}

/// One recorded state of a review, as of a single event.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewRevision {
    pub created_on: DateTime<Utc>,
    pub event: ReadingEvent,
    pub text: String,
}

/// A book's review, derived by replaying its review events.
///
/// Not persisted. A book has at most one, identified by the book itself —
/// there is no review id. Built by `Storage::review_for_book`.
#[derive(Debug, Clone)]
pub struct Review {
    pub book_id: String,
    /// When the review was first written.
    pub created_on: DateTime<Utc>,
    /// When it was last changed. Equal to `created_on` if never edited.
    pub updated_on: DateTime<Utc>,
    /// The current text: the newest snapshot.
    pub text: String,
    /// Every revision, oldest first.
    pub revisions: Vec<ReviewRevision>,
}

impl Review {
    /// How many times the review has been revised since it was written.
    pub fn edit_count(&self) -> usize {
        self.revisions.len().saturating_sub(1)
    }
}

impl Author {
    /// Creates a new author with a generated UUID and current timestamp.
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_on: Utc::now(),
        }
    }
}

impl Book {
    /// Creates a new book with a generated UUID and current timestamp.
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
            series_id: None,
            position_in_series: None,
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
    /// Creates a new reading event with a generated UUID and current timestamp.
    pub fn new(book_id: String, event: ReadingEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata::default(),
        }
    }

    /// Creates a new reading event with page progress metadata.
    pub fn with_metadata(book_id: String, event: ReadingEvent, current_page: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata {
                current_page: Some(current_page),
                note: None,
                review_text: None,
            },
        }
    }

    /// Creates an `Update` reading event carrying both a page number and a
    /// free-text note about the reader's progress.
    pub fn with_progress_note(book_id: String, current_page: i32, note: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event: ReadingEvent::Update,
            metadata: ReadingMetadata {
                current_page: Some(current_page),
                note: Some(note),
                review_text: None,
            },
        }
    }

    /// Creates a review event carrying the full review text as of this change.
    ///
    /// `event` must be `CreateReview` or `EditReview`.
    pub fn with_review(book_id: String, event: ReadingEvent, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata {
                current_page: None,
                note: None,
                review_text: Some(text),
            },
        }
    }
}

impl Category {
    /// Creates a new category with a generated UUID and current timestamp.
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_on: Utc::now(),
        }
    }
}

impl Series {
    /// Creates a new series with a generated UUID and current timestamp.
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_on: Utc::now(),
            status: None,
            total_books: None,
        }
    }
}

/// A yearly reading goal: how many books to finish and how many pages to read.
///
/// Serialized as `{"books": 30, "pages": 9000}`. Goals written before pages
/// existed were stored as a bare number; see `Goal`'s `Deserialize` impl.
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct Goal {
    pub books: u32,
    pub pages: u32,
}

/// The two on-disk shapes a goal can have. Untagged, so serde tries the legacy
/// bare number first and falls back to the current object form.
#[derive(Deserialize)]
#[serde(untagged)]
enum GoalRepr {
    Legacy(u32),
    Full { books: u32, pages: u32 },
}

impl<'de> Deserialize<'de> for Goal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match GoalRepr::deserialize(deserializer)? {
            GoalRepr::Legacy(books) => Goal { books, pages: 0 },
            GoalRepr::Full { books, pages } => Goal { books, pages },
        })
    }
}

/// The central data store containing all books, readings, authors, categories, and reviews.
///
/// Persisted as a single JSON file. All collections are keyed by UUID string.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Storage {
    pub books: HashMap<String, Book>,
    pub readings: HashMap<String, Reading>,
    pub authors: HashMap<String, Author>,
    pub categories: HashMap<String, Category>,
    /// Yearly reading goals: year -> books and pages targets.
    /// Uses `#[serde(default)]` for backward compatibility with existing JSON files.
    #[serde(default)]
    pub goals: HashMap<i32, Goal>,
    /// Book series (e.g. "Harry Potter", "A Song of Ice and Fire").
    /// Uses `#[serde(default)]` for backward compatibility with existing JSON files.
    #[serde(default)]
    pub series: HashMap<String, Series>,
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
            goals: HashMap::new(),
            series: HashMap::new(),
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

    pub fn add_series(&mut self, series: Series) -> Option<Series> {
        self.series.insert(series.id.clone(), series)
    }

    pub fn get_series(&self, id: &str) -> Option<&Series> {
        self.series.get(id)
    }

    /// Returns all books that belong to a given series, sorted by position_in_series.
    /// Positions are ordered numerically; books without a position are placed at the end.
    pub fn get_books_in_series(&self, series_id: &str) -> Vec<&Book> {
        let mut books: Vec<&Book> = self
            .books
            .values()
            .filter(|b| b.series_id.as_deref() == Some(series_id))
            .collect();
        books.sort_by(|a, b| compare_positions(a.position_in_series, b.position_in_series));
        books
    }

    /// Returns the series name for a given book, or an empty string if the book has no series
    pub fn series_name_for_book(&self, book: &Book) -> &str {
        book.series_id
            .as_ref()
            .and_then(|id| self.series.get(id))
            .map(|s| s.name.as_str())
            .unwrap_or("")
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

    /// Replays a book's review events into its current review.
    ///
    /// Returns None if the book has never been reviewed.
    pub fn review_for_book(&self, book_id: &str) -> Option<Review> {
        let mut events: Vec<&Reading> = self
            .readings
            .values()
            .filter(|r| r.book_id == book_id)
            .filter(|r| {
                matches!(
                    r.event,
                    ReadingEvent::CreateReview | ReadingEvent::EditReview
                )
            })
            .collect();

        if events.is_empty() {
            return None;
        }

        events.sort_by_key(|r| r.created_on);

        let revisions: Vec<ReviewRevision> = events
            .iter()
            .map(|r| ReviewRevision {
                created_on: r.created_on,
                event: r.event,
                text: r.metadata.review_text.clone().unwrap_or_default(),
            })
            .collect();

        let first = revisions.first()?;
        let last = revisions.last()?;

        Some(Review {
            book_id: book_id.to_string(),
            created_on: first.created_on,
            updated_on: last.created_on,
            text: last.text.clone(),
            revisions,
        })
    }

    /// Every book's review, newest first by creation date.
    ///
    /// Sorted on `created_on` rather than `updated_on` so the listing keeps
    /// its existing order and is sorted by the date column it displays.
    pub fn all_reviews(&self) -> Vec<Review> {
        let book_ids: std::collections::HashSet<&str> = self
            .readings
            .values()
            .filter(|r| {
                matches!(
                    r.event,
                    ReadingEvent::CreateReview | ReadingEvent::EditReview
                )
            })
            .map(|r| r.book_id.as_str())
            .collect();

        let mut reviews: Vec<Review> = book_ids
            .into_iter()
            .filter_map(|book_id| self.review_for_book(book_id))
            .collect();

        reviews.sort_by(|a, b| b.created_on.cmp(&a.created_on));
        reviews
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

    /// The most recent status-bearing event for a book.
    ///
    /// Non-status events (progress updates, review activity) are skipped, so
    /// they never displace the book's actual status.
    pub fn most_recent_reading_event(&self, book_id: &str) -> Option<ReadingEvent> {
        self.readings
            .values()
            .filter(|r| r.book_id == book_id)
            .filter(|r| r.event.affects_status())
            .max_by_key(|r| r.created_on)
            .map(|r| r.event)
    }

    pub fn get_started_books(&self) -> Vec<&Book> {
        self.books
            .values()
            .filter(|book| self.is_book_started(&book.id))
            .collect()
    }

    /// Returns books whose most recent status-bearing event matches the target.
    ///
    /// Only status-bearing events (see [`ReadingEvent::affects_status`]) can be matched.
    /// Passing a non-status event like `Update` always returns an empty vector.
    pub fn get_books_by_most_recent_event(&self, target_event: ReadingEvent) -> Vec<&Book> {
        self.books
            .values()
            .filter(|book| self.most_recent_reading_event(&book.id) == Some(target_event))
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

    /// Returns books the reader gave up on: their most recent event is `Abandoned`.
    pub fn get_abandoned_books(&self) -> Vec<&Book> {
        self.get_books_by_most_recent_event(ReadingEvent::Abandoned)
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

    /// Returns true if the book is currently being read (most recent status-relevant event is Started)
    ///
    /// Note: Update, Bought, WantToRead, and UnmarkedAsWantToRead events are skipped
    /// when determining started/finished status — only Started, Finished and
    /// Abandoned events matter. Finished and Abandoned both end the read-through.
    pub fn is_book_started(&self, book_id: &str) -> bool {
        let mut readings: Vec<_> = self
            .readings
            .values()
            .filter(|r| r.book_id == book_id)
            .collect();

        readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

        for reading in readings {
            match reading.event {
                ReadingEvent::Started => return true,
                ReadingEvent::Finished | ReadingEvent::Abandoned => return false,
                ReadingEvent::Update
                | ReadingEvent::Bought
                | ReadingEvent::WantToRead
                | ReadingEvent::UnmarkedAsWantToRead
                | ReadingEvent::CreateReview
                | ReadingEvent::EditReview => continue,
            }
        }
        false
    }

    pub fn is_book_finished(&self, book_id: &str) -> bool {
        self.most_recent_reading_event(book_id) == Some(ReadingEvent::Finished)
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
        let from = Utc
            .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
            .single()
            .expect("Jan 1 00:00:00 is always a valid UTC date");
        let to = Utc
            .with_ymd_and_hms(year, 12, 31, 23, 59, 59)
            .single()
            .expect("Dec 31 23:59:59 is always a valid UTC date");
        self.get_read_books_by_time_period(from, to)
    }

    /// Sets a yearly reading goal (books to finish and pages to read).
    pub fn set_goal(&mut self, year: i32, books: u32, pages: u32) {
        self.goals.insert(year, Goal { books, pages });
    }

    /// Returns the reading goal for a given year, or None if no goal is set.
    pub fn get_goal(&self, year: i32) -> Option<Goal> {
        self.goals.get(&year).copied()
    }

    /// Removes the reading goal for a given year, returning the previous value if it existed.
    pub fn remove_goal(&mut self, year: i32) -> Option<Goal> {
        self.goals.remove(&year)
    }

    /// Total pages credited to each year across every book.
    ///
    /// Groups readings by book, sorts each book's events chronologically, and
    /// folds in that book's per-year credits. Readings whose book is missing
    /// from storage are skipped, since their page count is unknowable.
    pub fn pages_read_by_year(&self) -> HashMap<i32, u32> {
        let mut by_book: HashMap<&str, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            by_book
                .entry(reading.book_id.as_str())
                .or_default()
                .push(reading);
        }

        let mut totals: HashMap<i32, u32> = HashMap::new();
        for (book_id, mut readings) in by_book {
            let total_pages = match self.books.get(book_id) {
                Some(book) => book.total_pages,
                None => continue,
            };
            readings.sort_by_key(|r| r.created_on);
            for (year, pages) in crate::pages::pages_credited_by_year(&readings, total_pages) {
                *totals.entry(year).or_insert(0) += pages;
            }
        }
        totals
    }

    /// Total pages read in a single year. Returns 0 for years with no reading.
    pub fn pages_read_in_year(&self, year: i32) -> u32 {
        self.pages_read_by_year().get(&year).copied().unwrap_or(0)
    }
}

/// Orders two series positions, placing books without a position last.
pub fn compare_positions(a: Option<i32>, b: Option<i32>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a_pos), Some(b_pos)) => a_pos.cmp(&b_pos),
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

/// What to do with a book whose stored position could not be migrated automatically.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionChoice {
    /// Place the book here, shifting occupants at or after this position up by one.
    Insert(i32),
    /// Place the book here, leaving other books alone. May leave two books sharing
    /// a position — permitted, matching `is_position_occupied`, which only warns.
    Set(i32),
    /// Drop the position. The book stays in the series and sorts last.
    Clear,
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

    /// Asks where a book with an unmigratable position should go.
    ///
    /// `old_value` is the value as written in the file, `suggested` is the
    /// proposed slot (`None` when the old value was non-numeric and offers no
    /// anchor), and `taken` lists positions currently occupied in the series.
    fn prompt_series_position(
        &self,
        book_title: &str,
        series_name: &str,
        old_value: &str,
        suggested: Option<i32>,
        taken: &[i32],
    ) -> Result<PositionChoice, Box<dyn std::error::Error>>;
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
    let mut orphaned_series: Vec<String> = Vec::new(); // book_ids with invalid series_id

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
        if let Some(ref sid) = book.series_id {
            if !storage.series.contains_key(sid) {
                orphaned_series.push(book_id.clone());
            }
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

    // Handle orphaned series_id — silently clear since series is optional
    if !orphaned_series.is_empty() {
        for book_id in &orphaned_series {
            if let Some(book) = storage.books.get_mut(book_id) {
                book.series_id = None;
                book.position_in_series = None;
            }
        }
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

    // Handle missing books — create book with new author and category,
    // then update the orphaned reading's book_id to point to the new book
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
        let new_book_id = book.id.clone();
        storage.add_book(book);

        // Update the reading's book_id to point to the newly created book
        if let Some(reading) = storage.readings.get_mut(&reading_id) {
            reading.book_id = new_book_id;
        }

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

/// Interactively migrates any `position_in_series` that is not a valid non-negative
/// integer. Returns `true` if the file was changed.
///
/// Runs on raw JSON and before deserialization, because once the field is `i32` a
/// file holding "2.5" cannot be loaded at all. The file is rewritten after every
/// answer, so an interrupted migration keeps the answers already given and
/// re-running resumes with the rest.
pub fn migrate_positions(
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<bool, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)?;

    let pending = scan_invalid_positions(&value);
    if pending.is_empty() {
        return Ok(false);
    }

    for fix in pending {
        let choice = match &fix.series_id {
            // No usable series: clear without asking. handle_missing_fields would
            // discard any answer we collected here moments later.
            None => PositionChoice::Clear,
            Some(series_id) => {
                // Recompute what is taken each time — an earlier Insert may have
                // shifted books since the scan.
                let taken = taken_positions(&value, series_id);
                prompter.prompt_series_position(
                    &fix.title,
                    &fix.series_name,
                    &fix.raw,
                    fix.suggested,
                    &taken,
                )?
            }
        };

        apply_position_choice(&mut value, &fix.book_id, fix.series_id.as_deref(), choice);
        write_json_value(storage_path, &value)?;
    }

    Ok(true)
}

/// Applies one migration answer to the raw JSON tree.
fn apply_position_choice(
    value: &mut serde_json::Value,
    book_id: &str,
    series_id: Option<&str>,
    choice: PositionChoice,
) {
    // Shift first, while the book being placed still holds its old invalid value —
    // that value never parses as an integer, so it is never shifted by accident.
    if let (PositionChoice::Insert(position), Some(series_id)) = (choice, series_id) {
        shift_json_positions_from(value, series_id, position, book_id);
    }

    let book = match value
        .get_mut("books")
        .and_then(|books| books.get_mut(book_id))
        .and_then(|book| book.as_object_mut())
    {
        Some(book) => book,
        None => return,
    };

    match choice {
        PositionChoice::Clear => {
            // Removed rather than nulled: the field is skip_serializing_if = is_none.
            book.remove("position_in_series");
        }
        PositionChoice::Insert(position) | PositionChoice::Set(position) => {
            book.insert(
                "position_in_series".to_string(),
                serde_json::Value::from(position),
            );
        }
    }
}

/// Raw-JSON counterpart of `series::shift_positions_from`, for use before
/// deserialization. Increments every valid position `>= from` in the series,
/// skipping `except`.
fn shift_json_positions_from(
    value: &mut serde_json::Value,
    series_id: &str,
    from: i32,
    except: &str,
) {
    let books = match value.get_mut("books").and_then(|b| b.as_object_mut()) {
        Some(books) => books,
        None => return,
    };

    for (book_id, book) in books.iter_mut() {
        if book_id == except || book.get("series_id").and_then(|s| s.as_str()) != Some(series_id) {
            continue;
        }
        let current = match book.get("position_in_series") {
            Some(position) => parse_integral_position(&raw_position_text(position)),
            None => None,
        };
        if let (Some(position), Some(book)) = (current, book.as_object_mut()) {
            if position >= from {
                book.insert(
                    "position_in_series".to_string(),
                    serde_json::Value::from(position + 1),
                );
            }
        }
    }
}

/// Converts legacy `reviews` entries into `CreateReview` reading events.
///
/// Must run before deserialization: serde ignores unknown keys, so once
/// `Storage` lost its `reviews` field a stale file would load cleanly and drop
/// every review without a word.
///
/// Each book keeps only its oldest review, by true chronological instant,
/// not by string comparison of the timestamp; later ones are reported and
/// discarded (ADR 0017). A review with no parseable `created_on` never wins
/// that slot over a dateable sibling, and if it is the *only* review for its
/// book, it is skipped and reported rather than written — an unparseable
/// timestamp on a `Reading` would fail to deserialize the whole storage file
/// on the next load. A backup is written first, since this loses data, and
/// an existing backup from an earlier run is never overwritten — see
/// `next_review_migration_backup_path`.
///
/// Returns `Ok(false)` when there was nothing to migrate, having touched
/// nothing.
pub fn migrate_reviews(storage_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)?;

    // Absent, null, not an object, or empty: nothing to migrate, and nothing
    // written. An empty `reviews` key left behind is harmless — serde ignores
    // unknown keys — and leaving it makes a second run a true no-op.
    let reviews = match value.get("reviews").and_then(|r| r.as_object()) {
        Some(reviews) if !reviews.is_empty() => reviews.clone(),
        _ => return Ok(false),
    };

    // Never overwrite a backup already on disk from an earlier run — it may
    // be the only surviving copy of reviews a previous run discarded.
    let backup_path = next_review_migration_backup_path(storage_path);
    fs::write(&backup_path, &contents)?;
    println!("Backed up the original file to {}", backup_path);

    // Group by book, oldest first.
    let mut by_book: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();
    for review in reviews.values() {
        let book_id = review
            .get("book_id")
            .and_then(|b| b.as_str())
            .unwrap_or_default()
            .to_string();
        by_book.entry(book_id).or_default().push(review.clone());
    }

    let known_books: std::collections::HashSet<String> = value
        .get("books")
        .and_then(|b| b.as_object())
        .map(|books| books.keys().cloned().collect())
        .unwrap_or_default();

    let mut events = serde_json::Map::new();

    for (book_id, mut group) in by_book {
        // Sort by the parsed instant, not the raw string: chrono serializes
        // with `SecondsFormat::AutoSi`, so the number of fractional-second
        // digits varies with the value, and lexical order does not agree
        // with chronological order (e.g. "...100500Z" < "...100Z" as
        // strings, even though .100500s is later than .100s). A review
        // whose timestamp is missing or fails to parse sorts LAST, never
        // first: we cannot establish that an undateable review is the
        // oldest, and preferring a dateable sibling for the "keep" slot
        // keeps the resulting event well-formed (see the check below).
        group.sort_by(|a, b| {
            let a_instant = review_instant(a);
            let b_instant = review_instant(b);
            match (a_instant, b_instant) {
                (Some(a), Some(b)) => a.cmp(&b),
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        let title = value
            .get("books")
            .and_then(|books| books.get(&book_id))
            .and_then(|book| book.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("an unknown book")
            .to_string();

        if !known_books.contains(&book_id) {
            println!(
                "Discarded {} review(s) referencing a book that no longer exists.",
                group.len()
            );
            continue;
        }

        let oldest = group.remove(0);
        if !group.is_empty() {
            println!(
                "Discarded {} later review(s) for \"{}\".",
                group.len(),
                title
            );
        }

        // Never write an event whose `created_on` would not round-trip:
        // `Reading.created_on` is a `DateTime<Utc>`, so an unparseable value
        // here would fail to deserialize the *entire* storage file the next
        // time it is loaded. This can only happen when every review for this
        // book was undateable (the sort above always prefers a dateable one
        // when there is one). The backup written above is the recovery path.
        if review_instant(&oldest).is_none() {
            println!(
                "Skipped the review for \"{}\": its created_on is not a valid timestamp.",
                title
            );
            continue;
        }

        let id = oldest
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or_default()
            .to_string();
        let text = oldest.get("text").and_then(|t| t.as_str()).unwrap_or("");

        events.insert(
            id.clone(),
            serde_json::json!({
                "id": id,
                "created_on": oldest.get("created_on").cloned().unwrap_or(serde_json::Value::Null),
                "book_id": book_id,
                "event": "CreateReview",
                "metadata": { "review_text": text }
            }),
        );
    }

    let root = value
        .as_object_mut()
        .ok_or("storage root is not an object")?;
    let readings = root
        .entry("readings")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    if let Some(readings) = readings.as_object_mut() {
        readings.extend(events);
    }
    root.remove("reviews");

    write_json_value(storage_path, &value)?;
    Ok(true)
}

/// Parses a raw review's `created_on` as an RFC 3339 instant, for ordering
/// reviews by true chronology rather than by string. Returns `None` when the
/// field is missing or fails to parse — callers must treat that as the
/// *least* trustworthy case, not the oldest: we have no evidence about when
/// an undateable review was written, and letting it win the "keep" slot
/// would carry an invalid timestamp into the migrated event.
fn review_instant(review: &serde_json::Value) -> Option<DateTime<FixedOffset>> {
    review
        .get("created_on")
        .and_then(|c| c.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
}

/// Returns the first backup path for `storage_path` that does not already
/// exist, so a second migration run never overwrites a backup left behind by
/// an earlier one — that backup may be the only remaining copy of reviews an
/// earlier run discarded. Tries `<path>.pre-review-migration.bak` first,
/// then `<path>.pre-review-migration.<n>.bak` for increasing `n`.
fn next_review_migration_backup_path(storage_path: &str) -> String {
    let primary = format!("{storage_path}.pre-review-migration.bak");
    if !Path::new(&primary).exists() {
        return primary;
    }
    let mut n = 2;
    loop {
        let candidate = format!("{storage_path}.pre-review-migration.{n}.bak");
        if !Path::new(&candidate).exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Writes a raw JSON tree using the same sorted-key, pretty formatting as
/// `write_storage`, so a migrated file is byte-identical to a normally saved one.
fn write_json_value(
    storage_path: &str,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let sorted = sort_json_value(value.clone());
    fs::write(storage_path, serde_json::to_string_pretty(&sorted)?)?;
    Ok(())
}

/// Loads storage, migrating legacy series positions and repairing any missing
/// references, using the given prompter.
pub fn load_and_repair_storage(
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<Storage, Box<dyn std::error::Error>> {
    // Must precede load_storage: an unmigrated position cannot be deserialized.
    migrate_positions(storage_path, prompter)?;
    // Must also precede it: serde ignores the unknown `reviews` key, so a
    // stale file would otherwise load cleanly and lose every review.
    migrate_reviews(storage_path)?;
    let mut storage = load_storage(storage_path)?;
    handle_missing_fields(&mut storage, storage_path, prompter)?;
    Ok(storage)
}
