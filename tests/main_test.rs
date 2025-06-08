#[test]
fn test_hello_world() {
    // This is a basic test that we can expand later
    // Currently, we're just testing that the program compiles
    assert!(true);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(2 + 2, 4);
    }
}
