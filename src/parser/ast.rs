#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
    VarDecl(String, Option<Box<Expr>>),
}

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
    Identifier(String),
    Assign(String, Box<Expr>),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Unary(Operator, Box<Expr>),
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
}
