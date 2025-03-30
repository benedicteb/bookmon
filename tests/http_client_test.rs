use bookmon::http_client::HttpClient;

#[tokio::test]
async fn test_get_book_by_isbn() {
    let client = HttpClient::new();
    let result = client.get_book_by_isbn("9780142410349").await;  // ISBN for Fantastic Mr. Fox
    assert!(result.is_ok());
    
    let book = result.unwrap();
    assert_eq!(book.title, "Fantastic Mr Fox");  // Note: Title might not have the period
    assert!(!book.authors.is_empty());
    
    // Verify author data from search results
    let author = &book.authors[0];
    assert!(author.name.as_ref().unwrap().contains("Roald Dahl"));
} 