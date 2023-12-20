use std::collections::HashMap;

use crate::parser::ast::{Expr, Operator, Stmt};

use self::value::Value;

pub mod value;

/// Basic treewalk interpreter, will be replaced later by something more efficient.
#[derive(Debug, Default)]
pub struct Interpreter {
    globals: HashMap<String, Value>,
}

impl Interpreter {
    pub fn interpret_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.interpret_stmt(stmt);
                }
            }
            Stmt::Expr(expr) => match self.interpret_expr(expr) {
                Ok(value) => {
                    println!("value: {:?}", value);
                }
                Err(err) => {
                    eprintln!("runtime error: {}", err.message());
                }
            },
        };
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(*val),
            Expr::Binary(op, left, right) => {
                let left = self.interpret_expr(left)?;
                let right = self.interpret_expr(right)?;

                let value = match op {
                    Operator::Add => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Number(left + right)
                    }
                    Operator::Sub => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Number(left - right)
                    }
                    Operator::Mul => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Number(left * right)
                    }
                    Operator::Div => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        if right == 0.0 {
                            return Err(RuntimeError::DivisionByZero);
                        }
                        Value::Number(left / right)
                    }
                    Operator::Mod => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        if right == 0.0 {
                            return Err(RuntimeError::DivisionByZero);
                        }
                        Value::Number(left % right)
                    }
                    Operator::Exp => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Number(left.powf(right))
                    }
                    _ => todo!(),
                };

                Ok(value)
            }
            Expr::Unary(op, expr) => {
                let right = self.interpret_expr(expr)?;
                let value = match op {
                    Operator::Not => {
                        if let Value::Bool(b) = right {
                            Value::Bool(!b)
                        } else {
                            return Err(RuntimeError::InvalidOperand);
                        }
                    }
                    Operator::Sub => {
                        if let Value::Number(n) = right {
                            Value::Number(-n)
                        } else {
                            return Err(RuntimeError::InvalidOperand);
                        }
                    }
                    _ => unreachable!(),
                };

                Ok(value)
            }
            _ => todo!(),
        }
    }
}

pub fn check_number_operands(a: &Value, b: &Value) -> Result<(f64, f64), RuntimeError> {
    if let (Value::Number(a), Value::Number(b)) = (a, b) {
        Ok((*a, *b))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// Currently we just keep track of which type of error occured, we need to change this so it
// contains a span so we can report to the user where the error occured.
#[derive(Debug, Clone, Copy)]
pub enum RuntimeError {
    InvalidOperand,
    DivisionByZero,
}

impl RuntimeError {
    fn message(self) -> &'static str {
        match self {
            RuntimeError::InvalidOperand => "unsupported operand type",
            RuntimeError::DivisionByZero => "division by zero is undefined",
        }
    }
}
