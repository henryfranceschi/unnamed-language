use std::collections::HashMap;

use crate::parser::ast::{Expr, Operator, Stmt};

use self::value::Value;

mod value;

/// Basic treewalk interpreter, will be replaced later by something more efficient.
pub struct Interpreter {
    globals: HashMap<String, Value>, 
}

impl Interpreter {
    pub fn interpret_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block(_) => todo!(),
            Stmt::Expr(_) => todo!(),
        }
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(val) => Value::Number(*val),
            Expr::Binary(op, left, right) => {
                let left = self.interpret_expr(left);
                let right = self.interpret_expr(right);

                todo!()
            },
            Expr::Unary(op, expr) => {
                let value = self.interpret_expr(expr);
                match op {
                    Operator::Not => todo!(),
                    Operator::Sub => todo!(),
                    _ => unreachable!()
                }
            },
            _ => todo!()
        }
    }
}

