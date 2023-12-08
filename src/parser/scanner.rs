use std::str::CharIndices;

use crate::{
    lookahead::Lookahead,
    parser::token::{Span, TokenKind},
};

use super::token::Token;

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    iter: Lookahead<2, CharIndices<'a>>,
    start: usize,
    end: usize,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>, ScanError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.scan())
    }
}

impl<'a> Scanner<'a> {
    const EOF_CHAR: char = '\0';

    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            iter: Lookahead::new(source.char_indices()),
            start: 0,
            end: 0,
        }
    }

    pub fn scan(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        self.advance_while(|c| c.is_ascii_whitespace());
        let c = self.advance();
        self.start = self.end;

        let kind = match c {
            /* Identifiers */
            c if c.is_alphabetic() || c == '_' => {
                return self.identifier();
            }

            /* Literals */
            c if c.is_ascii_digit() => {
                return self.number();
            }

            '"' => {
                return self.string();
            }

            /* Punctuators. */
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBrack,
            ']' => TokenKind::RBrack,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Period,

            /* Arithmetic operators. */
            '+' => {
                if self.advance_if_eq('=') {
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.advance_if_eq('=') {
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.advance_if_eq('*') {
                    TokenKind::StarStar
                } else if self.advance_if_eq('=') {
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.advance_if_eq('=') {
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                if self.advance_if_eq('=') {
                    TokenKind::PercentEqual
                } else {
                    TokenKind::Percent
                }
            }

            /* Comparison operators. */
            '=' => {
                if self.advance_if_eq('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }

            '<' => {
                if self.advance_if_eq('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }

            '>' => {
                if self.advance_if_eq('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }

            Self::EOF_CHAR => TokenKind::Eof,

            c => {
                return Err(self.error(format!("unexpected character '{c}'")));
            }
        };

        Ok(self.token(kind))
    }

    fn lookahead(&mut self, n: usize) -> char {
        self.iter
            .lookahead(n)
            .map(|(_, ch)| *ch)
            .unwrap_or(Self::EOF_CHAR)
    }

    fn advance(&mut self) -> char {
        if let Some((idx, chr)) = self.iter.next() {
            self.end = idx;
            chr
        } else {
            Self::EOF_CHAR
        }
    }

    fn advance_if<F>(&mut self, predicate: F) -> bool
    where
        F: FnOnce(char) -> bool + Copy,
    {
        if predicate(self.lookahead(0)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance_if_eq(&mut self, chr: char) -> bool {
        self.advance_if(|c| c == chr)
    }

    fn advance_while<F>(&mut self, predicate: F) -> bool
    where
        F: FnOnce(char) -> bool + Copy,
    {
        let mut advanced = false;
        while !self.is_at_end() && predicate(self.lookahead(0)) {
            self.advance();
            advanced = true;
        }

        advanced
    }

    fn advance_while_eq(&mut self, chr: char) -> bool {
        self.advance_while(|c| c == chr)
    }

    fn current(&self) -> &'a str {
        &self.source[self.start..=self.end]
    }

    fn identifier(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');

        Ok(self.token(
            TokenKind::keyword_kind_from_str(self.current()).unwrap_or(TokenKind::Identifier),
        ))
    }

    fn string(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        // Consume everything until we find a closing quote or we reach the end of the source.
        self.advance_while(|c| c != '"');

        if !self.advance_if_eq('"') {
            Err(self.error("expected closing quotes".to_owned()))
        } else {
            Ok(self.token(TokenKind::String))
        }
    }

    fn number(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        // Scan ingegral part.
        self.advance_while(|c| c.is_ascii_digit());

        // Scan optional fractional part.
        // Because we are able to call methods on values directly we only want to consume the dot
        // if it is followed by numeric characters. Otherwise we leave the characters to be
        // consumed as separate tokens.
        if self.lookahead(0) == '.' && self.lookahead(1).is_ascii_digit() {
            self.advance();

            while self.lookahead(0).is_ascii_digit() {
                self.advance();
            }
        }

        Ok(self.token(TokenKind::Number))
    }

    fn is_at_end(&mut self) -> bool {
        self.lookahead(0) == Self::EOF_CHAR
    }

    fn token(&mut self, kind: TokenKind) -> Token<'a> {
        let token = Token::new(self.span(), kind);
        self.start = self.end;

        token
    }

    fn span(&self) -> Span<'a> {
        Span::new(self.source, self.start, self.end)
    }

    fn error(&self, message: String) -> ScanError<'a> {
        ScanError::new(message, self.span())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScanError<'a> {
    pub message: String,
    pub span: Span<'a>,
}

impl<'a> ScanError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { message, span }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scanner, Span, Token, TokenKind};

    macro_rules! t {
        ($src:expr, $start:expr, $end:expr, $kind:expr) => {
            Token::new(Span::new($src, $start, $end), $kind)
        };
    }

    #[test]
    fn number() {
        use TokenKind::*;
        let src = "256.log2()";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 0, 2, Number)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 3, 3, Period)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 4, 7, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 8, 8, LParen)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 9, 9, RParen)));
        
        let src = "12.34";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 0, 4, Number)));
    }

    #[test]
    fn var_decl() {
        use TokenKind::*;
        let src = "let x = 10;";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 0, 2, Let)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 4, 4, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 6, 6, Equal)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 8, 9, Number)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 10, 10, Semicolon)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 10, 10, Eof)));
    }

    #[test]
    fn fun_decl() {
        use TokenKind::*;
        let src = "func add(x, y) { return x + y; }";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 0, 3, Func)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 5, 7, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 8, 8, LParen)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 9, 9, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 10, 10, Comma)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 12, 12, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 13, 13, RParen)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 15, 15, LBrace)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 17, 22, Return)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 24, 24, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 26, 26, Plus)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 28, 28, Identifier)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 29, 29, Semicolon)));
        assert_eq!(scanner.next().unwrap(), Ok(t!(src, 31, 31, RBrace)));
    }
}
