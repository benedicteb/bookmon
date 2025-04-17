use std::error::Error;
use crate::lookup::providers::ProviderManager;
use crate::lookup::book_lookup_dto::BookLookupDTO;

pub struct HttpClient {
    provider_manager: ProviderManager,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            provider_manager: ProviderManager::new()
        }
    }

    pub async fn get_book_by_isbn(&self, isbn: &str) -> Result<Option<BookLookupDTO>, Box<dyn Error>> {
        self.provider_manager.get_book_by_isbn(isbn).await
    }
} 