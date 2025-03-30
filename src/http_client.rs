use serde::{Deserialize, Serialize};
use std::error::Error;

const USER_AGENT: &str = concat!("bookmon/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenLibraryBook {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_description")]
    pub description: Option<String>,
    #[serde(default)]
    pub first_publish_date: Option<String>,
    #[serde(default)]
    pub covers: Option<Vec<i64>>,
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
pub struct Author {
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
pub struct Link {
    pub title: Option<String>,
    pub url: String,
    #[serde(rename = "type")]
    pub type_: LinkType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkType {
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

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    async fn fetch_author_data(&self, author_key: &str) -> Result<Author, Box<dyn Error>> {
        let url = format!("https://openlibrary.org{}.json", author_key);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let author: Author = serde_json::from_str(&response_text)?;
        Ok(author)
    }

    async fn search_for_book(&self, isbn: &str) -> Result<SearchDoc, Box<dyn Error>> {
        let url = format!("https://openlibrary.org/search.json?q={}", isbn);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let search_response: SearchResponse = serde_json::from_str(&response_text)?;

        if search_response.num_found == 0 {
            return Err("No books found for the given ISBN".into());
        }

        Ok(search_response.docs[0].clone())
    }

    pub async fn get_book_by_isbn(&self, isbn: &str) -> Result<OpenLibraryBook, Box<dyn Error>> {
        // First search for the book to get its work key
        let search_result = self.search_for_book(isbn).await?;
        
        // Extract the work key from the search result
        let work_key = search_result.key;
        
        // Fetch the book data using the work key
        let url = format!("https://openlibrary.org{}.json", work_key);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        
        // Parse the work response
        let mut work_response: serde_json::Value = serde_json::from_str(&response_text)?;
        
        // Extract authors from the work response
        let authors = if let Some(work_authors) = work_response.get("authors") {
            let work_authors: Vec<WorkAuthor> = serde_json::from_value(work_authors.clone())?;
            let mut authors = Vec::new();
            for (work_author, name) in work_authors.iter().zip(search_result.author_name.iter()) {
                // First create a basic author with the name from search
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

                // Then try to fetch detailed author data
                match self.fetch_author_data(&work_author.data.key).await {
                    Ok(detailed_author) => {
                        // Keep the name from search but use other fields from detailed data
                        author.personal_name = detailed_author.personal_name;
                        author.fuller_name = detailed_author.fuller_name;
                        author.bio = detailed_author.bio;
                        author.birth_date = detailed_author.birth_date;
                        author.death_date = detailed_author.death_date;
                        author.alternate_names = detailed_author.alternate_names;
                        author.links = detailed_author.links;
                    }
                    Err(_) => {
                        // Silently continue with basic author data
                    }
                }
                authors.push(author);
            }
            authors
        } else {
            Vec::new()
        };
        
        // Remove the authors field from the work response to avoid conflicts
        if let Some(obj) = work_response.as_object_mut() {
            obj.remove("authors");
        }
        
        // Parse the rest of the book data
        let mut book: OpenLibraryBook = serde_json::from_value(work_response)?;
        book.authors = authors;
        
        Ok(book)
    }
} 