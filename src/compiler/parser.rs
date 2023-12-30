use crate::interpreter::value::Value;

use self::{
    ast::{Decl, Expr, Operator, Script, Stmt},
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

    fn advance_if(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind() == kind {
            self.advance();
            true
        } else {
            false
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
    fn expect(&mut self, expected: TokenKind) -> Result<Token<'a>, ParseError<'a>> {
        let token = self.peek();
        if token.kind() == expected {
            Ok(self.advance())
        } else {
            let quote_maybe = |k: TokenKind| {
                if k.is_variable_length() || k == TokenKind::Eof {
                    k.to_string()
                } else {
                    format!("'{k}'")
                }
            };

            let message = format!(
                "expected {} got {}",
                quote_maybe(expected),
                quote_maybe(token.kind())
            );

            Err(ParseError::new(&token, message))
        }
    }

    pub fn parse(&mut self) -> Result<Script, ParseError<'a>> {
        self.script()
    }

    fn script(&mut self) -> Result<Script, ParseError<'a>> {
        let mut decls = vec![];
        while self.peek().kind() != TokenKind::Eof {
            decls.push(self.decl()?);
        }

        Ok(Script { decls })
    }

    fn decl(&mut self) -> Result<Decl, ParseError<'a>> {
        let kind = self.peek().kind();
        match kind {
            TokenKind::Let => self.var_decl(),
            TokenKind::Func => self.func_decl(),
            _ => Ok(Decl::Stmt(Box::new(self.stmt()?))),
        }
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        match self.peek().kind() {
            TokenKind::LBrace => self.block_stmt(),
            TokenKind::If => self.if_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn block_stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.expect(TokenKind::LBrace)?;

        let mut declarations = vec![];
        loop {
            if matches!(self.peek().kind(), TokenKind::Eof | TokenKind::RBrace) {
                break;
            }

            declarations.push(self.decl()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::Block(declarations))
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        self.expect(TokenKind::If)?;
        let predicate = self.expr()?;
        let consequent = self.stmt()?;
        let alternative = if self.advance_if(TokenKind::Else) {
            Some(self.stmt()?)
        } else {
            None
        };

        Ok(Stmt::If(
            Box::new(predicate),
            Box::new(consequent),
            alternative.map(Box::new),
        ))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParseError<'a>> {
        let expr = self.expr()?;

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Expr(Box::new(expr)))
    }

    fn var_decl(&mut self) -> Result<Decl, ParseError<'a>> {
        self.expect(TokenKind::Let)?;

        let name = self.expect(TokenKind::Identifier)?.slice().into();
        let init_expr = if self.advance_if(TokenKind::Equal) {
            Some(Box::new(self.expr()?))
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Decl::Var(name, init_expr))
    }

    fn func_decl(&mut self) -> Result<Decl, ParseError<'a>> {
        self.expect(TokenKind::Func)?;

        let name = self.expect(TokenKind::Identifier)?.slice().into();

        self.expect(TokenKind::LParen)?;
        let mut params = vec![];
        while !matches!(self.peek().kind(), TokenKind::Eof | TokenKind::RParen) {
            params.push(self.expect(TokenKind::Identifier)?.slice().into());
        }
        self.expect(TokenKind::RParen)?;

        Ok(Decl::Func(name, params, Box::new(self.block_stmt()?)))
    }

    fn expr(&mut self) -> Result<Expr, ParseError<'a>> {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError<'a>> {
        let token = self.advance();
        let mut expr = match token.kind() {
            TokenKind::Identifier => Expr::Identifier(token.slice().into()),
            TokenKind::Number => Expr::Literal(Value::Number(token.slice().parse().unwrap())),
            TokenKind::String => {
                todo!();
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

        // We only continue if the peeked token is a valid operator.
        while let Ok(operator) = Operator::try_from(self.peek()) {
            // Handle infix case.
            if let Some((l_bp, r_bp)) = operator.infix_binding_power() {
                if l_bp < min_bp {
                    break;
                }

                // We only advance if the peeked token is a valid infix operator, otherwise we
                // leave the token to be handled elsewhere.
                self.advance();
                if min_bp == 0 && operator == Operator::Assign {
                    expr = Expr::Assignment(Box::new(expr), Box::new(self.expr()?));
                } else {
                    expr = Expr::Binary(operator, Box::new(expr), Box::new(self.expr_bp(r_bp)?));
                }

                continue;
            } else {
                break;
            }
        }

        if self.peek().kind() == TokenKind::Equal {
            todo!("invalid assignment target");
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
