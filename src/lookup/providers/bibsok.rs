use crate::lookup::book_lookup_dto::{AuthorDTO, BookLookupDTO};
use crate::lookup::providers::BookProvider;
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;

const HOSTNAME: &str = "https://bibsok.no";

pub struct BibsokProvider {
    pub client: reqwest::Client,
}

impl Default for BibsokProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BibsokProvider {
    pub fn new() -> Self {
        Self {
            client: super::create_http_client(),
        }
    }

    fn parse_html(&self, html: &str) -> Result<BookLookupDTO, Box<dyn Error>> {
        let document = Html::parse_document(html);

        // Selectors for different parts of the page
        let book_selector =
            Selector::parse(".c-post--simple").expect("valid CSS selector: .c-post--simple");
        let title_selector =
            Selector::parse(".o-adaptive-title").expect("valid CSS selector: .o-adaptive-title");
        let author_selector = Selector::parse(".u-inlineblock[lang=nb]")
            .expect("valid CSS selector: .u-inlineblock[lang=nb]");
        let year_selector = Selector::parse("span").expect("valid CSS selector: span");
        let cover_selector =
            Selector::parse(".c-post__bilde div").expect("valid CSS selector: .c-post__bilde div");

        // Get the first book result
        let book_element = document
            .select(&book_selector)
            .next()
            .ok_or("No book found in the search results")?;

        // Extract title
        let title = book_element
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        // Extract authors
        let authors = book_element
            .select(&author_selector)
            .flat_map(|e| {
                let name = e.text().collect::<String>();
                name.split('&')
                    .map(|name| name.trim())
                    .filter(|name| !name.is_empty())
                    .map(|name| AuthorDTO {
                        name: name.to_string(),
                        personal_name: None,
                        birth_date: None,
                        death_date: None,
                        bio: None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Extract year
        let year = book_element
            .select(&year_selector)
            .find(|e| {
                let text = e.text().collect::<String>();
                text.chars().all(|c| c.is_ascii_digit()) && text.len() == 4
            })
            .map(|e| e.text().collect::<String>());

        // Extract cover URL
        let cover_url = book_element.select(&cover_selector).next().and_then(|e| {
            e.value().attr("style").and_then(|style| {
                let re = Regex::new(r"background-image:url\('([^']+)'\)")
                    .expect("valid static regex for background-image URL extraction");
                re.captures(style).map(|caps| caps[1].to_string())
            })
        });

        Ok(BookLookupDTO {
            title,
            authors,
            description: None, // Bibsok doesn't provide descriptions in the search results
            isbn: String::new(), // Will be set by the caller
            publish_date: year,
            cover_url,
        })
    }
}

#[async_trait]
impl BookProvider for BibsokProvider {
    fn name(&self) -> &'static str {
        "Bibsok"
    }

    async fn get_book_by_isbn(&self, isbn: &str) -> Result<Option<BookLookupDTO>, Box<dyn Error>> {
        let url = format!(
            "{}/?mode=vt&hpid=3276004&pubsok_txt_0={}&pubsok_kval_0=/IS&avgr_bn=&avgr_medier=&avgr_spraak=&aarfra=&aartil=",
            HOSTNAME, isbn
        );

        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;

        let mut book = self.parse_html(&html)?;
        book.isbn = isbn.to_string();

        Ok(Some(book))
    }
}
