use self::scanner::Scanner;

mod cursor;
pub mod scanner;
pub mod token;

pub struct Parser<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
        }
    }
}
