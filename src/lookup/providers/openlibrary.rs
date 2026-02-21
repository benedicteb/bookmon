use crate::lookup::book_lookup_dto::{AuthorDTO, BookLookupDTO};
use crate::lookup::providers::BookProvider;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::LazyLock;

const HOSTNAME: &str = "https://openlibrary.org";

/// Compiled once on first use — matches series strings like "Harry Potter #1" or "Kingkiller #2.5".
static SERIES_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+?)\s*#(\d+(?:\.\d+)?)\s*$").expect("valid static regex"));

/// Parses a series string from OpenLibrary's Edition API.
///
/// Examples:
///   "Harry Potter #1" -> ("Harry Potter", Some("1"))
///   "Kingkiller Chronicle #2.5" -> ("Kingkiller Chronicle", Some("2.5"))
///   "OXFORD WORLD'S CLASSICS" -> ("OXFORD WORLD'S CLASSICS", None)
///   "" -> ("", None)
pub fn parse_series_string(s: &str) -> (String, Option<String>) {
    let s = s.trim();
    if let Some(caps) = SERIES_REGEX.captures(s) {
        let name = caps[1].trim().to_string();
        let position = Some(caps[2].to_string());
        (name, position)
    } else {
        (s.to_string(), None)
    }
}

/// Edition data from OpenLibrary's ISBN API.
#[derive(Debug, Serialize, Deserialize)]
struct OpenLibraryEdition {
    #[serde(default)]
    series: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenLibraryBook {
    title: String,
    #[serde(default)]
    authors: Vec<Author>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_description")]
    description: Option<String>,
    #[serde(default)]
    first_publish_date: Option<String>,
    #[serde(default)]
    covers: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextValue {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}

fn deserialize_description<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Object(map) => {
            if let Some(value) = map.get("value") {
                if let Some(text) = value.as_str() {
                    Ok(Some(text.to_string()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        serde_json::Value::Null => Ok(None),
        _ => Err(D::Error::custom("unexpected description format")),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkAuthor {
    #[serde(rename = "author")]
    pub data: AuthorRef,
    #[serde(rename = "type")]
    pub role_type: RoleType,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthorRef {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RoleType {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuller_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_bio")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

fn deserialize_bio<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Object(map) => {
            if let Some(value) = map.get("value") {
                if let Some(text) = value.as_str() {
                    Ok(Some(text.to_string()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        serde_json::Value::Null => Ok(None),
        _ => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Link {
    pub title: Option<String>,
    pub url: String,
    #[serde(rename = "type")]
    pub type_: LinkType,
}

#[derive(Debug, Serialize, Deserialize)]
struct LinkType {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    num_found: i32,
    docs: Vec<SearchDoc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SearchDoc {
    key: String,
    title: String,
    author_key: Vec<String>,
    author_name: Vec<String>,
    first_publish_year: Option<i32>,
}

pub struct OpenLibraryProvider {
    pub client: reqwest::Client,
}

impl OpenLibraryProvider {
    async fn fetch_author_data(&self, author_key: &str) -> Result<Author, Box<dyn Error>> {
        let url = format!("{}{}.json", HOSTNAME, author_key);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let author: Author = serde_json::from_str(&response_text)?;
        Ok(author)
    }

    async fn search_for_book(&self, isbn: &str) -> Result<SearchDoc, Box<dyn Error>> {
        let url = format!("{}/search.json?q={}", HOSTNAME, isbn);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let search_response: SearchResponse = serde_json::from_str(&response_text)?;

        if search_response.num_found == 0 {
            return Err("No books found for the given ISBN".into());
        }

        Ok(search_response.docs[0].clone())
    }

    async fn fetch_work_data(&self, work_key: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!("{}{}.json", HOSTNAME, work_key);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        Ok(serde_json::from_str(&response_text)?)
    }

    async fn process_author_data(
        &self,
        work_authors: &[WorkAuthor],
        author_names: &[String],
    ) -> Result<Vec<Author>, Box<dyn Error>> {
        let mut authors = Vec::new();
        for (work_author, name) in work_authors.iter().zip(author_names.iter()) {
            let mut author = Author {
                key: work_author.data.key.clone(),
                name: Some(name.clone()),
                personal_name: None,
                fuller_name: None,
                bio: None,
                birth_date: None,
                death_date: None,
                alternate_names: None,
                links: None,
            };

            if let Ok(detailed_author) = self.fetch_author_data(&work_author.data.key).await {
                author.personal_name = detailed_author.personal_name;
                author.fuller_name = detailed_author.fuller_name;
                author.bio = detailed_author.bio;
                author.birth_date = detailed_author.birth_date;
                author.death_date = detailed_author.death_date;
                author.alternate_names = detailed_author.alternate_names;
                author.links = detailed_author.links;
            }
            authors.push(author);
        }
        Ok(authors)
    }

    /// Fetches edition data by ISBN to get series information.
    /// Returns None if the edition is not found or has no series data.
    async fn fetch_edition_series(
        &self,
        isbn: &str,
    ) -> Result<Option<(String, Option<String>)>, Box<dyn Error>> {
        let url = format!("{}/isbn/{}.json", HOSTNAME, isbn);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let response_text = response.text().await?;
        let edition: OpenLibraryEdition = serde_json::from_str(&response_text)?;

        if let Some(series_list) = edition.series {
            if let Some(first_series) = series_list.first() {
                let (name, position) = parse_series_string(first_series);
                if !name.is_empty() {
                    return Ok(Some((name, position)));
                }
            }
        }

        Ok(None)
    }

    fn convert_to_dto(
        &self,
        book: OpenLibraryBook,
        authors: Vec<Author>,
        isbn: &str,
        series_info: Option<(String, Option<String>)>,
    ) -> BookLookupDTO {
        let (series_name, series_position) = match series_info {
            Some((name, pos)) => (Some(name), pos),
            None => (None, None),
        };

        BookLookupDTO {
            title: book.title,
            authors: authors
                .into_iter()
                .map(|a| AuthorDTO {
                    name: a.name.unwrap_or_default(),
                    personal_name: a.personal_name,
                    birth_date: a.birth_date,
                    death_date: a.death_date,
                    bio: a.bio,
                })
                .collect(),
            description: book.description,
            isbn: isbn.to_string(),
            publish_date: book.first_publish_date,
            cover_url: book.covers.and_then(|c| {
                c.first()
                    .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id))
            }),
            series_name,
            series_position,
        }
    }
}

#[async_trait]
impl BookProvider for OpenLibraryProvider {
    fn name(&self) -> &'static str {
        "OpenLibrary"
    }

    async fn get_book_by_isbn(&self, isbn: &str) -> Result<Option<BookLookupDTO>, Box<dyn Error>> {
        // Search for the book to get its work key
        let search_result = self.search_for_book(isbn).await?;
        let work_key = search_result.key;

        // Fetch and process work data
        let mut work_response = self.fetch_work_data(&work_key).await?;

        // Process authors
        let authors = if let Some(work_authors) = work_response.get("authors") {
            let work_authors: Vec<WorkAuthor> = serde_json::from_value(work_authors.clone())?;
            self.process_author_data(&work_authors, &search_result.author_name)
                .await?
        } else {
            Vec::new()
        };

        // Remove authors from work response to avoid conflicts
        if let Some(obj) = work_response.as_object_mut() {
            obj.remove("authors");
        }

        // Fetch edition data for series info (best-effort, don't fail on error)
        let series_info = self.fetch_edition_series(isbn).await.unwrap_or(None);

        // Parse book data and convert to DTO
        let book: OpenLibraryBook = serde_json::from_value(work_response)?;
        Ok(Some(self.convert_to_dto(book, authors, isbn, series_info)))
    }
}
