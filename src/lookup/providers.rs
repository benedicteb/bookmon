pub mod bibsok;
pub mod openlibrary;

use crate::lookup::book_lookup_dto::BookLookupDTO;
use async_trait::async_trait;
use std::error::Error;

const USER_AGENT: &str = concat!("bookmon/", env!("CARGO_PKG_VERSION"));

pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to create HTTP client")
}

#[async_trait]
pub trait BookProvider {
    fn name(&self) -> &'static str;
    async fn get_book_by_isbn(&self, isbn: &str) -> Result<Option<BookLookupDTO>, Box<dyn Error>>;
}

pub use bibsok::BibsokProvider;
pub use openlibrary::OpenLibraryProvider;

pub struct ProviderManager {
    providers: Vec<Box<dyn BookProvider>>,
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderManager {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn BookProvider>> = vec![
            Box::new(OpenLibraryProvider {
                client: create_http_client(),
            }),
            Box::new(BibsokProvider {
                client: create_http_client(),
            }),
        ];
        Self { providers }
    }

    pub async fn get_book_by_isbn(
        &self,
        isbn: &str,
    ) -> Result<Option<BookLookupDTO>, Box<dyn Error>> {
        let mut errors = Vec::new();

        for provider in &self.providers {
            match provider.get_book_by_isbn(isbn).await {
                Ok(Some(book)) => return Ok(Some(book)),
                Ok(None) => continue,
                Err(e) => errors.push((provider.name(), e)),
            }
        }

        if !errors.is_empty() {
            for (provider_name, error) in errors {
                eprintln!("Error from provider {}: {}", provider_name, error);
            }
        }

        Ok(None)
    }
}
