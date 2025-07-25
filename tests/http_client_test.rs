use bookmon::lookup::http_client::HttpClient;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_get_book_by_isbn() {
    let client = HttpClient::new();

    // List of test cases with detailed author checks where needed
    let test_cases = vec![
        ("9780142410349", "Fantastic Mr Fox", "Roald Dahl", None),
        (
            "9780008164966",
            "One, Two, Buckle My Shoe",
            "Agatha Christie",
            Some(AuthorDetails {
                personal_name: "Agatha Christie",
                birth_date: "15 September 1890",
                death_date: "12 January 1976",
            }),
        ),
        (
            "9780241970775",
            "The Big Sleep",
            "Raymond Chandler",
            Some(AuthorDetails {
                personal_name: "Chandler, Raymond",
                birth_date: "1888",
                death_date: "1959",
            }),
        ),
        (
            "9780008129576",
            "Sad Cypress",
            "Agatha Christie",
            Some(AuthorDetails {
                personal_name: "Agatha Christie",
                birth_date: "15 September 1890",
                death_date: "12 January 1976",
            }),
        ),
    ];

    for (isbn, expected_title, expected_author, author_details) in test_cases {
        let result = client.get_book_by_isbn(isbn).await;
        assert!(result.is_ok(), "Failed to fetch book with ISBN {}", isbn);

        let book = result.unwrap();
        assert!(book.is_some(), "No book found for ISBN {}", isbn);
        let book = book.unwrap();
        assert_eq!(book.title, expected_title);
        assert!(
            !book.authors.is_empty(),
            "No authors found for ISBN {}",
            isbn
        );

        let author = &book.authors[0];
        assert!(author.name.contains(expected_author));

        // Check detailed author information if provided
        if let Some(details) = author_details {
            if let Some(personal_name) = author.personal_name.as_ref() {
                assert_eq!(personal_name, details.personal_name);
            }
            if let Some(birth_date) = author.birth_date.as_ref() {
                assert_eq!(birth_date, details.birth_date);
            }
            if let Some(death_date) = author.death_date.as_ref() {
                assert_eq!(death_date, details.death_date);
            }
        }

        // Add a 1-second delay between requests to be nice to the API
        sleep(Duration::from_secs(1)).await;
    }
}

struct AuthorDetails {
    personal_name: &'static str,
    birth_date: &'static str,
    death_date: &'static str,
}
