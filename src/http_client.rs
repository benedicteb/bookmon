use serde::{Deserialize, Serialize};
use std::error::Error;

const USER_AGENT: &str = concat!("bookmon/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenLibraryBook {
    pub title: String,
    pub authors: Vec<Author>,
    pub publish_date: Option<String>,
    pub publishers: Option<Vec<String>>,
    pub number_of_pages: Option<i32>,
    pub isbn_10: Option<Vec<String>>,
    pub isbn_13: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_date: Option<String>,
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
        let author: Author = response.json().await?;
        Ok(author)
    }

    pub async fn get_book_by_isbn(&self, isbn: &str) -> Result<OpenLibraryBook, Box<dyn Error>> {
        let url = format!("https://openlibrary.org/books/{}.json", isbn);
        let response = self.client.get(&url).send().await?;
        let mut book: OpenLibraryBook = response.json().await?;

        // Fetch author data for each author
        let mut authors_with_data = Vec::new();
        for author in book.authors {
            match self.fetch_author_data(&author.key).await {
                Ok(author_data) => authors_with_data.push(author_data),
                Err(e) => {
                    eprintln!("Warning: Failed to fetch author data for {}: {}", author.key, e);
                    authors_with_data.push(author);
                }
            }
        }
        book.authors = authors_with_data;
        Ok(book)
    }
} 