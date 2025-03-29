pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1)
    }
} 