use crate::interpreter::value::Value;

// use super::token::Span;

// pub struct Spanned<'a, T> {
//     span: Span<'a>,
//     spanned: T,
// }

pub struct Script {
    pub decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Var(Identifier, Option<Box<Expr>>),
    Func(Identifier, Vec<Identifier>, Box<Stmt>),
    Stmt(Box<Stmt>),
}

#[derive(Debug)]
pub enum Stmt {
    /// Neither consequent or alternative statements should be any kind of declaration.
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Expr(Box<Expr>),
    Block(Vec<Decl>),
    Print(Box<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
    Identifier(Identifier),
    /// Currently the only valid value for `0` is `Identifier`.
    Assignment(Box<Expr>, Box<Expr>),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Unary(Operator, Box<Expr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identifier(String);

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Assign,
    Or,
    And,
    Not,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

impl Operator {
    pub fn is_prefix(self) -> bool {
        use Operator::*;

        matches!(self, Not | Sub)
    }

    pub fn is_infix(self) -> bool {
        use Operator::*;

        matches!(
            self,
            Assign | Or | And | Eq | Ne | Lt | Gt | Le | Ge | Add | Sub | Mul | Div | Mod | Exp
        )
    }

    pub(super) fn prefix_binding_power(self) -> Option<((), u8)> {
        use Operator::*;

        let bp = match self {
            Not => ((), 7),
            Sub => ((), 19),
            _ => return None,
        };

        Some(bp)
    }

    pub(super) fn infix_binding_power(self) -> Option<(u8, u8)> {
        use Operator::*;

        let bp = match self {
            Assign => (2, 1),
            Or => (3, 4),
            And => (5, 6),
            Eq | Ne => (9, 10),
            Lt | Gt | Le | Ge => (11, 12),
            Add | Sub => (13, 14),
            Mul | Div | Mod => (15, 16),
            Exp => (18, 17),
            _ => return None,
        };

        Some(bp)
    }

    // pub(super) fn postfix_binding_power(self) -> Option<(u8, ())> {
    //     use Operator::*;

    //     let bp = match self {
    //         _ => return None,
    //     };

    //     Some(bp)
    // }
}
