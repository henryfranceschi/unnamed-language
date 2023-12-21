use crate::parser::token::{Span, TokenKind};

use super::{cursor::Cursor, token::Token};

#[derive(Debug)]
pub struct Scanner<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    pub fn scan(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        while self.cursor.lookahead(0).is_ascii_whitespace() {
            self.cursor.advance();
        }

        self.cursor.reset_start_index();

        let kind = match (self.cursor.advance(), self.cursor.lookahead(0)) {
            (c, _) if is_identifier_start(c) => self.identifier(),

            ('0'..='9', _) => {
                self.number();
                TokenKind::Number
            }

            ('"', _) => {
                self.string()?;
                TokenKind::String
            }

            ('{', _) => TokenKind::LBrace,
            ('}', _) => TokenKind::RBrace,
            ('(', _) => TokenKind::LParen,
            (')', _) => TokenKind::RParen,
            ('[', _) => TokenKind::LBrack,
            (']', _) => TokenKind::RBrack,
            (';', _) => TokenKind::Semicolon,
            (',', _) => TokenKind::Comma,
            ('.', _) => TokenKind::Period,
            ('-', '=') => {
                self.cursor.advance();
                TokenKind::MinusEqual
            }
            ('*', '=') => {
                self.cursor.advance();
                TokenKind::StarEqual
            }
            ('*', '*') => {
                self.cursor.advance();
                TokenKind::StarStar
            }
            ('/', '=') => {
                self.cursor.advance();
                TokenKind::SlashEqual
            }
            ('%', '=') => {
                self.cursor.advance();
                TokenKind::PercentEqual
            }
            ('=', '=') => {
                self.cursor.advance();
                TokenKind::EqualEqual
            }
            ('!', '=') => {
                self.cursor.advance();
                TokenKind::BangEqual
            }
            ('<', '=') => {
                self.cursor.advance();
                TokenKind::LessEqual
            }
            ('>', '=') => {
                self.cursor.advance();
                TokenKind::GreaterEqual
            }
            ('+', _) => TokenKind::Plus,
            ('-', _) => TokenKind::Minus,
            ('*', _) => TokenKind::Star,
            ('/', _) => TokenKind::Slash,
            ('%', _) => TokenKind::Percent,
            ('=', _) => TokenKind::Equal,
            ('<', _) => TokenKind::Less,
            ('>', _) => TokenKind::Greater,

            (Cursor::EOF_CHAR, _) => TokenKind::Eof,

            (c, _) => {
                let message = format!("unexpected character '{c}'");
                return Err(ScanError::new(message, self.cursor.reset_span()));
            }
        };

        Ok(Token::new(self.cursor.reset_span(), kind))
    }

    fn identifier(&mut self) -> TokenKind {
        while is_identifier_continue(self.cursor.lookahead(0)) {
            self.cursor.advance();
        }

        TokenKind::keyword_kind_from_str(self.cursor.span().slice())
            .unwrap_or(TokenKind::Identifier)
    }

    fn string(&mut self) -> Result<(), ScanError<'a>> {
        // Consume everything until we find a closing quote or we reach the end of the source.
        while !self.cursor.is_at_end() && self.cursor.lookahead(0) != '"' {
            self.cursor.advance();
        }

        if self.cursor.lookahead(0) != '"' {
            let message = "expected closing quotes".to_owned();
            Err(ScanError::new(message, self.cursor.reset_span()))
        } else {
            self.cursor.advance();
            Ok(())
        }
    }

    fn number(&mut self) {
        // Scan ingegral part.
        while self.cursor.lookahead(0).is_ascii_digit() {
            self.cursor.advance();
        }

        // Scan optional fractional part.
        // Because we are able to call methods on values directly we only want to consume the dot
        // if it is followed by numeric characters. Otherwise we leave the characters to be
        // consumed as separate tokens.
        if self.cursor.lookahead(0) == '.' && self.cursor.lookahead(1).is_ascii_digit() {
            self.cursor.advance();

            while self.cursor.lookahead(0).is_ascii_digit() {
                self.cursor.advance();
            }
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
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
    fn scan_eof() {
        use TokenKind::*;

        let src = "";
        assert_eq!(Scanner::new(src).scan(), Ok(t!(src, 0, 0, Eof)))
    }

    #[test]
    fn scan_number() {
        use TokenKind::*;
        let src = "256.log2()";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.scan(), Ok(t!(src, 0, 3, Number)));
        assert_eq!(scanner.scan(), Ok(t!(src, 3, 4, Period)));
        assert_eq!(scanner.scan(), Ok(t!(src, 4, 8, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 8, 9, LParen)));
        assert_eq!(scanner.scan(), Ok(t!(src, 9, 10, RParen)));

        let src = "12.34";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.scan(), Ok(t!(src, 0, 5, Number)));
    }

    #[test]
    fn scan_var_decl() {
        use TokenKind::*;
        let src = "let x = 10;";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.scan(), Ok(t!(src, 0, 3, Let)));
        assert_eq!(scanner.scan(), Ok(t!(src, 4, 5, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 6, 7, Equal)));
        assert_eq!(scanner.scan(), Ok(t!(src, 8, 10, Number)));
        assert_eq!(scanner.scan(), Ok(t!(src, 10, 11, Semicolon)));
        assert_eq!(scanner.scan(), Ok(t!(src, 11, 11, Eof)));
    }

    #[test]
    fn scan_fun_decl() {
        use TokenKind::*;
        let src = "func add(x, y) { return x + y; }";
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.scan(), Ok(t!(src, 0, 4, Func)));
        assert_eq!(scanner.scan(), Ok(t!(src, 5, 8, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 8, 9, LParen)));
        assert_eq!(scanner.scan(), Ok(t!(src, 9, 10, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 10, 11, Comma)));
        assert_eq!(scanner.scan(), Ok(t!(src, 12, 13, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 13, 14, RParen)));
        assert_eq!(scanner.scan(), Ok(t!(src, 15, 16, LBrace)));
        assert_eq!(scanner.scan(), Ok(t!(src, 17, 23, Return)));
        assert_eq!(scanner.scan(), Ok(t!(src, 24, 25, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 26, 27, Plus)));
        assert_eq!(scanner.scan(), Ok(t!(src, 28, 29, Identifier)));
        assert_eq!(scanner.scan(), Ok(t!(src, 29, 30, Semicolon)));
        assert_eq!(scanner.scan(), Ok(t!(src, 31, 32, RBrace)));
    }
}
