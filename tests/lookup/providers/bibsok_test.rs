use bookmon::lookup::providers::bibsok::BibsokProvider;
use bookmon::BookProvider;

#[tokio::test]
#[ignore] // Live API test — run with `cargo test -- --ignored`
async fn test_bibsok_lookup() {
    let provider = BibsokProvider::new();
    let isbn = "9788293671381";

    let result = provider.get_book_by_isbn(isbn).await;
    assert!(result.is_ok(), "Failed to look up book");

    let book = result.unwrap().expect("No book found");

    // Verify title
    assert_eq!(book.title, "Nullpunkt");

    // Verify authors
    assert_eq!(book.authors.len(), 2);
    assert_eq!(book.authors[0].name.trim(), "Jørn Lier Horst");
    assert_eq!(book.authors[1].name.trim(), "Thomas Enger");

    // Verify publication year
    assert_eq!(book.publish_date, Some("2018".to_string()));

    // Verify ISBN
    assert_eq!(book.isbn, isbn);

    // Verify cover URL exists
    assert!(book.cover_url.is_some());
    assert!(book.cover_url.unwrap().contains("krydder.bib.no"));
}
