use bookmon::lookup::providers::openlibrary::parse_series_string;

#[test]
fn test_parse_series_string_with_position() {
    let (name, position) = parse_series_string("Harry Potter #1");
    assert_eq!(name, "Harry Potter");
    assert_eq!(position, Some("1".to_string()));
}

#[test]
fn test_parse_series_string_with_higher_position() {
    let (name, position) = parse_series_string("A Song of Ice and Fire #5");
    assert_eq!(name, "A Song of Ice and Fire");
    assert_eq!(position, Some("5".to_string()));
}

#[test]
fn test_parse_series_string_without_position() {
    let (name, position) = parse_series_string("OXFORD WORLD'S CLASSICS");
    assert_eq!(name, "OXFORD WORLD'S CLASSICS");
    assert_eq!(position, None);
}

#[test]
fn test_parse_series_string_empty() {
    let (name, position) = parse_series_string("");
    assert_eq!(name, "");
    assert_eq!(position, None);
}

#[test]
fn test_parse_series_string_with_hash_in_name() {
    // Edge case: series name contains # but not as a position marker
    let (name, position) = parse_series_string("Series with #hashtag in name");
    // The regex should only match trailing # followed by a number
    assert_eq!(name, "Series with #hashtag in name");
    assert_eq!(position, None);
}

#[test]
fn test_parse_series_string_trims_whitespace() {
    let (name, position) = parse_series_string("  Harry Potter  #3  ");
    assert_eq!(name, "Harry Potter");
    assert_eq!(position, Some("3".to_string()));
}

#[test]
fn test_parse_series_string_position_with_no_space() {
    let (name, position) = parse_series_string("Harry Potter#1");
    assert_eq!(name, "Harry Potter");
    assert_eq!(position, Some("1".to_string()));
}

#[test]
fn test_parse_series_string_with_fractional_position() {
    let (name, position) = parse_series_string("Kingkiller Chronicle #2.5");
    assert_eq!(name, "Kingkiller Chronicle");
    assert_eq!(position, Some("2.5".to_string()));
}
