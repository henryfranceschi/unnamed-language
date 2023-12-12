use std::str::CharIndices;

use crate::{
    lookahead::Lookahead,
    parser::token::{Span, TokenKind},
};

use super::{token::Token, cursor::Cursor};

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
