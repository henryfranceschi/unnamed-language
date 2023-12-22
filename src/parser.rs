use crate::interpreter::value::Value;

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
        match self.peeked {
            Some(token) => token,
            None => {
                let token = self.next_token();
                self.peeked.replace(token);
                token
            }
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

    /// Advances if next token equals `expected`, otherwise returns `ParseError`.
    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError<'a>> {
        let token = self.peek();
        if token.kind() == expected {
            self.advance();
            Ok(())
        } else {
            let message = if expected.is_variable_length() || expected == TokenKind::Eof {
                format!("expected {}", expected)
            } else {
                format!("expected '{}'", expected)
            };

            Err(ParseError::new(&token, message))
        }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.stmt()
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        let kind = self.peek().kind();
        if kind == TokenKind::LBrace {
            self.block_stmt()
        } else if kind == TokenKind::Let {
            self.var_decl()
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

    fn var_decl(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.expect(TokenKind::Let)?;

        let token = self.advance();
        let name = token.span().slice().to_owned();

        let init_expr = if self.peek().kind() == TokenKind::Equal {
            self.advance();
            Some(Box::new(self.expr()?))
        } else {
            None
        };

        // Consume semicolon.
        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::VarDecl(name, init_expr))
    }

    fn expr(&mut self) -> Result<Expr, ParseError<'a>> {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError<'a>> {
        let token = self.advance();
        let mut expr = match token.kind() {
            TokenKind::Identifier => Expr::Identifier(token.span().slice().to_owned()),
            TokenKind::Number => {
                let Ok(number) = token.span().slice().parse() else {
                    return Err(ParseError::new(
                        &token,
                        "failed to parse numeric literal".to_owned(),
                    ));
                };

                Expr::Literal(Value::Number(number))
            }
            TokenKind::String => {
                let slice = token.span().slice();
                // For now we just remove the surrounding quotes.
                // Expr::String(slice[1..slice.len() - 1].to_owned())

                todo!()
            }
            TokenKind::False => Expr::Literal(Value::Bool(false)),
            TokenKind::True => Expr::Literal(Value::Bool(true)),
            TokenKind::Nil => Expr::Literal(Value::Nil),
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
    pub fn new(token: &Token<'a>, message: String) -> Self {
        Self {
            span: token.span(),
            message,
        }
    }

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
