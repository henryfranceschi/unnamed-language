use crate::lookahead::Lookahead;
use self::scanner::Scanner;

mod cursor;
pub mod scanner;
pub mod token;

struct Parser<'a> {
    iter: Lookahead<2, Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { iter: Lookahead::new(Scanner::new(source)) }
    }
}
