use std::str::Chars;

use super::token::Span;

/// `Cursor` iterates over its `input` string, it keeps a `start_index` which is the begining of the
/// current span in the input.
#[derive(Debug)]
pub struct Cursor<'a> {
    input: &'a str,
    iter: Chars<'a>,
    start: usize,
}

impl<'a> Cursor<'a> {
    pub const EOF_CHAR: char = '\0';

    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            input,
            iter: input.chars(),
            start: 0,
        }
    }

    pub fn advance(&mut self) -> char {
        self.iter.next().unwrap_or(Self::EOF_CHAR)
    }

    pub fn lookahead(&self, n: usize) -> char {
        self.iter.clone().nth(n).unwrap_or(Self::EOF_CHAR)
    }

    /// Returns the number of bytes left.
    pub fn bytes_remaining(&self) -> usize {
        self.iter.as_str().len()
    }

    pub fn is_at_end(&self) -> bool {
        self.lookahead(0) == Self::EOF_CHAR
    }

    pub fn start_index(&self) -> usize {
        self.start
    }

    /// Returns the current index in the input and the end index of the current span
    /// (exclusive).
    pub fn current_index(&self) -> usize {
        self.input.len() - self.bytes_remaining()
    }

    /// Resets the starting index and returns the old value.
    pub fn reset_start_index(&mut self) -> usize {
        let offset = self.start;
        self.start = self.current_index();
        offset
    }

    /// Returns the current span.
    pub fn span(&self) -> Span<'a> {
        Span::new(self.input, self.start_index(), self.current_index())
    }

    /// Returns the current span and resets the starting index.
    pub fn reset_span(&mut self) -> Span<'a> {
        Span::new(self.input, self.reset_start_index(), self.current_index())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cursor::Cursor;

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
        assert_eq!(c.advance(), Cursor::EOF_CHAR);

        let mut c = Cursor::new("");
        assert_eq!(c.advance(), Cursor::EOF_CHAR);
    }

    #[test]
    fn lookahead() {
        let mut c = Cursor::new("one");
        assert_eq!(c.lookahead(0), 'o');
        assert_eq!(c.advance(), 'o');

        let c = Cursor::new("");
        assert_eq!(c.lookahead(0), Cursor::EOF_CHAR);
    }

    #[test]
    fn is_at_end() {
        let c = Cursor::new("");
        assert!(c.is_at_end());
    }
}
