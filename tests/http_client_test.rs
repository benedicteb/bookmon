use bookmon::http_client::HttpClient;

#[tokio::test]
async fn test_get_book_by_isbn() {
    let client = HttpClient::new();
    let result = client.get_book_by_isbn("OL7353617M").await;
    assert!(result.is_ok());
    
    let book = result.unwrap();
    assert_eq!(book.title, "Fantastic Mr. Fox");
    assert!(!book.authors.is_empty());
    assert_eq!(book.number_of_pages, Some(96));

    // Verify author data
    let author = &book.authors[0];
    assert_eq!(author.name, Some("Roald Dahl".to_string()));
    assert_eq!(author.personal_name, Some("Dahl, Roald.".to_string()));
    assert!(author.bio.is_some());
    assert_eq!(author.birth_date, Some("13 September 1916".to_string()));
    assert_eq!(author.death_date, Some("23 November 1990".to_string()));
} 