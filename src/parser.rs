use self::{
    ast::{Expr, Operator, Stmt},
    scanner::Scanner,
    token::{Span, Token, TokenKind},
};

pub mod ast;
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

    fn peek(&mut self) -> Token<'a> {
        if self.peeked.is_some() {
            self.peeked.unwrap()
        } else {
            let token = self.next_token();
            self.peeked.replace(token);
            token
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

    /// Advances if next token equals `expected`, otherwise returns `ParseError`
    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError<'a>> {
        let token = self.peek();
        if token.kind() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(&token, format!("expected '{:?}'", expected)))
        }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.stmt()
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        if self.peek().kind() == TokenKind::LBrace {
            self.block_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn block_stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.expect(TokenKind::LBrace)?;

        let mut statments = vec![];
        loop {
            let token = self.peek();
            if matches!(token.kind(), TokenKind::Eof | TokenKind::RBrace) {
                break;
            }

            statments.push(self.stmt()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::Block(statments))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        let expr = self.expr()?;

        // Consume semicolon.
        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Expr(Box::new(expr)))
    }

    fn expr(&mut self) -> Result<Expr, ParseError<'a>> {
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
                self.expect(TokenKind::RParen)?;
                expr
            }
            _ => {
                // The only remaining types of tokens valid in prefix position are those
                // representing prefix operators.
                let operator: Operator = token.try_into()?;
                if let Some(((), r_bp)) = operator.prefix_binding_power() {
                    Expr::Unary(operator, Box::new(self.expr_bp(r_bp)?))
                } else {
                    // Unexpected token.
                    todo!("error reporting");
                }
            }
        };

        loop {
            // We only continue if the peeked token is a valid operator.
            let Ok(operator): Result<Operator, _> = self.peek().try_into() else {
                break;
            };

            // Handle infix case.
            if let Some((l_bp, r_bp)) = operator.infix_binding_power() {
                if l_bp < min_bp {
                    break;
                }

                // We only advance if the peeked token is a valid infix operator, otherwise we
                // leave the token to be handled elsewhere.
                self.advance();
                expr = Expr::Binary(operator, Box::new(expr), Box::new(self.expr_bp(r_bp)?));
                continue;
            }

            break;
        }

        Ok(expr)
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

impl<'a> TryFrom<Token<'a>> for Operator {
    type Error = ParseError<'a>;

    fn try_from(token: Token<'a>) -> Result<Self, Self::Error> {
        let op = match token.kind() {
            TokenKind::Equal => Self::Assign,
            TokenKind::Or => Self::Or,
            TokenKind::And => Self::And,
            TokenKind::Not => Self::Not,
            TokenKind::EqualEqual => Self::Eq,
            TokenKind::BangEqual => Self::Ne,
            TokenKind::Less => Self::Lt,
            TokenKind::Greater => Self::Gt,
            TokenKind::LessEqual => Self::Le,
            TokenKind::GreaterEqual => Self::Ge,
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Sub,
            TokenKind::Star => Self::Mul,
            TokenKind::Slash => Self::Div,
            TokenKind::Percent => Self::Mod,
            TokenKind::StarStar => Self::Exp,
            _ => {
                let message = format!("unexpected token: {:?}", token);
                return Err(ParseError {
                    span: token.span(),
                    message,
                });
            }
        };

        Ok(op)
    }
}
