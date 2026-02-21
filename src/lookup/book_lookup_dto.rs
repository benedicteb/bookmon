#[derive(Debug, Clone)]
pub struct AuthorDTO {
    pub name: String,
    pub personal_name: Option<String>,
    pub birth_date: Option<String>,
    pub death_date: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookLookupDTO {
    pub title: String,
    pub authors: Vec<AuthorDTO>,
    pub description: Option<String>,
    pub isbn: String,
    pub publish_date: Option<String>,
    pub cover_url: Option<String>,
    /// Series name from ISBN lookup (e.g. "Harry Potter")
    pub series_name: Option<String>,
    /// Position within the series (e.g. "1", "2.5" for novellas)
    pub series_position: Option<String>,
}
