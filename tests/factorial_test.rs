use bookmon::factorial;

#[test]
fn test_factorial_zero() {
    assert_eq!(factorial(0), 1, "Factorial of 0 should be 1");
}

#[test]
fn test_factorial_one() {
    assert_eq!(factorial(1), 1, "Factorial of 1 should be 1");
}

#[test]
fn test_factorial_small_numbers() {
    assert_eq!(factorial(3), 6, "Factorial of 3 should be 6");
    assert_eq!(factorial(4), 24, "Factorial of 4 should be 24");
    assert_eq!(factorial(5), 120, "Factorial of 5 should be 120");
}

#[test]
fn test_factorial_larger_number() {
    assert_eq!(factorial(10), 3628800, "Factorial of 10 should be 3,628,800");
} 