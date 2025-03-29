pub mod config;
pub mod storage;
pub mod book;
pub mod category;

pub fn factorial(n: u32) -> u32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
} 