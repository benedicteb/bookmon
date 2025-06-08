pub mod author;
pub mod book;
pub mod category;
pub mod config;
pub mod reading;
pub mod storage;
pub mod lookup {
    pub mod book_lookup_dto;
    pub mod http_client;
    pub mod providers;
}

pub use lookup::providers::BookProvider;
pub use lookup::providers::ProviderManager;
