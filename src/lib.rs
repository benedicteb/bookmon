pub mod author;
pub mod book;
pub mod category;
pub mod config;
pub mod diff;
pub mod editor;
pub mod goal;
pub mod pages;
pub mod reading;
pub mod review;
pub mod series;
pub mod storage;
pub mod table;
pub mod lookup {
    pub mod book_lookup_dto;
    pub mod http_client;
    pub mod providers;
}

pub use lookup::providers::BookProvider;
pub use lookup::providers::ProviderManager;
