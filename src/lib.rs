pub mod book;
pub mod storage;
pub mod reading;
pub mod config;
pub mod author;
pub mod category;
pub mod lookup {
    pub mod http_client;
    pub mod book_lookup_dto;
    pub mod providers;
}

pub use lookup::providers::BookProvider;
pub use lookup::providers::ProviderManager;
