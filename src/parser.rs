use self::{
    scanner::Scanner,
    token::{Span, Token, TokenKind},
};

mod cursor;
pub mod scanner;
pub mod token;

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    peeked: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            peeked: None,
        }
    }

    fn advance(&mut self) -> Token<'a> {
        match self.peeked.take() {
            Some(token) => token,
            None => self.next_token(),
        }
    }

    fn peek(&mut self) -> &Token<'a> {
        if self.peeked.is_some() {
            self.peeked.as_ref().unwrap()
        } else {
            let token = self.next_token();
            self.peeked.replace(token);
            self.peeked.as_ref().unwrap()
        }
    }

    /// Scans until the scanner returns a token, reporting all errors.
    fn next_token(&mut self) -> Token<'a> {
        loop {
            match self.scanner.scan() {
                Ok(token) => return token,
                // Report scanning errors.
                Err(_) => todo!(),
            }
        }
    }

    pub fn expr(&mut self) -> Result<Expr, ParseError<'a>> {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError<'a>> {
        let token = self.advance();
        let mut expr = match token.kind() {
            TokenKind::Identifier => Expr::Identifier(token.span().slice().to_owned()),
            TokenKind::Number => {
                let slice = token.span().slice();
                let number: f64 = slice.parse().unwrap();
                Expr::Number(number)
            }
            TokenKind::String => {
                let span = token.span().slice();
                // For now we just remove the surrounding quotes.
                Expr::String(span[1..span.len() - 1].to_owned())
            }
            // Grouping
            TokenKind::LParen => {
                let expr = self.expr_bp(0)?;
                if self.advance().kind() != TokenKind::RParen {
                    todo!("error reporting");
                }

                expr
            }
            _ => {
                // The only remaining types of tokens valid in prefix position are those
                // representing prefix operators.
                if let Some(((), r_bp)) = Self::prefix_binding_power(&token) {
                    Expr::Unary {
                        operator: token.kind(),
                        operand: Box::new(self.expr_bp(r_bp)?),
                    }
                } else {
                    // Unexpected token.
                    todo!("error reporting");
                }
            }
        };

        loop {
            let token = self.peek();
            if token.is_eof() {
                break;
            }

            if let Some((l_bp, r_bp)) = Self::infix_binding_power(token) {
                if l_bp < min_bp {
                    break;
                }

                let token = self.advance();

                expr = Expr::Binary {
                    operator: token.kind(),
                    left_operand: Box::new(expr),
                    right_operand: Box::new(self.expr_bp(r_bp)?),
                };

                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn prefix_binding_power(token: &Token) -> Option<((), u8)> {
        use TokenKind::*;

        let bp = match token.kind() {
            Not => ((), 7),
            Minus => ((), 19),
            _ => return None,
        };

        Some(bp)
    }

    fn infix_binding_power(token: &Token) -> Option<(u8, u8)> {
        use TokenKind::*;
        let bp = match token.kind() {
            Equal => (2, 1),
            Or => (3, 4),
            And => (5, 6),
            EqualEqual | BangEqual => (9, 10),
            Less | Greater | LessEqual | GreaterEqual => (11, 12),
            Plus | Minus => (13, 14),
            Star | Slash | Percent => (15, 16),
            StarStar => (18, 17),
            _ => return None,
        };

        Some(bp)
    }
}

#[derive(Debug)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: String,
}

impl<'a> ParseError<'a> {
    pub fn span(&self) -> Span<'a> {
        self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub enum Expr {
    Identifier(String),
    String(String),
    Number(f64),
    Binary {
        operator: TokenKind,
        left_operand: Box<Expr>,
        right_operand: Box<Expr>,
    },
    Unary {
        operator: TokenKind,
        operand: Box<Expr>,
    },
}
        }
    }
}
