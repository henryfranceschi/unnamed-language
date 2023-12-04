use std::{collections::HashMap, str::CharIndices};

use once_cell::sync::Lazy;

use crate::{
    lookahead::Lookahead,
    parser::token::{Span, TokenKind},
};

use super::token::Token;

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
        self.start = self.end;

        let kind = match self.advance() {
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
        while predicate(self.lookahead(0)) {
            self.advance();
            advanced = true;
        }

        advanced
    }

    fn advance_while_eq(&mut self, chr: char) -> bool {
        self.advance_while(|c| c == chr)
    }

    fn identifier(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');
        static KEYWORDS: Lazy<HashMap<&str, TokenKind>> = Lazy::new(|| {
            HashMap::from([
                ("true", TokenKind::True),
                ("false", TokenKind::False),
                ("nil", TokenKind::Nil),
                ("let", TokenKind::Let),
                ("mut", TokenKind::Mut),
                ("fun", TokenKind::Func),
                ("class", TokenKind::Class),
                ("this", TokenKind::This),
                ("return", TokenKind::Return),
                ("for", TokenKind::For),
                ("while", TokenKind::While),
                ("if", TokenKind::If),
                ("else", TokenKind::Else),
            ])
        });

        Ok(self.token(TokenKind::Identifier))
    }

    fn string(&mut self) -> Result<Token<'a>, ScanError<'a>> {
        // Consume everything until we find a closing quote or we reach the end of the source.
        while self.lookahead(0) != '"' {
            self.advance();
        }

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

#[derive(Debug)]
pub struct ScanError<'a> {
    pub message: String,
    pub span: Span<'a>,
}

impl<'a> ScanError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { message, span }
    }
}
