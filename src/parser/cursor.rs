use std::str::Chars;

pub const EOF_CHAR: char = '\0';

#[derive(Debug)]
pub struct Cursor<'a> {
    iter: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(s: &'a str) -> Cursor<'a> {
        Cursor { iter: s.chars() }
    }

    /// The number of bytes left.
    pub fn bytes_remaining(&self) -> usize {
        self.iter.as_str().len()
    }

    pub fn advance(&mut self) -> char {
        self.iter.next().unwrap_or(EOF_CHAR)
    }

    pub fn lookahead(&self, n: usize) -> char {
        self.iter.clone().nth(n).unwrap_or(EOF_CHAR)
    }

    pub fn is_at_end(&self) -> bool {
        self.lookahead(0) == EOF_CHAR
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cursor::{Cursor, EOF_CHAR};

    #[test]
    fn offset() {
        let mut c = Cursor::new("one");
        assert_eq!(c.bytes_remaining(), 3);
        c.advance();
        assert_eq!(c.bytes_remaining(), 2);
        c.advance();
        assert_eq!(c.bytes_remaining(), 1);
        c.advance();
        assert_eq!(c.bytes_remaining(), 0);

        let mut c = Cursor::new("");
        assert_eq!(c.bytes_remaining(), 0);
        c.advance();
        assert_eq!(c.bytes_remaining(), 0);
    }

    #[test]
    fn advance() {
        let mut c = Cursor::new("one");
        assert_eq!(c.advance(), 'o');
        assert_eq!(c.advance(), 'n');
        assert_eq!(c.advance(), 'e');
        assert_eq!(c.advance(), EOF_CHAR);

        let mut c = Cursor::new("");
        assert_eq!(c.advance(), EOF_CHAR);
    }

    #[test]
    fn lookahead() {
        let mut c = Cursor::new("one");
        assert_eq!(c.lookahead(0), 'o');
        assert_eq!(c.advance(), 'o');

        let c = Cursor::new("");
        assert_eq!(c.lookahead(0), EOF_CHAR);
    }

    #[test]
    fn is_at_end() {
        let c = Cursor::new("");
        assert!(c.is_at_end());
    }
}
