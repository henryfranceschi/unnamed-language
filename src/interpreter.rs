use thiserror::Error;

use self::{environment::Environment, value::Value};
use crate::compiler::parser::ast::{Decl, Expr, Operator, Stmt, Script};

mod environment;
pub mod object;
pub mod value;

/// Basic treewalk interpreter, will be replaced later by something more efficient.
#[derive(Debug, Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn interpret(&mut self, script: &Script) -> Result<(), RuntimeError> {
        for decl in &script.decls {
            self.decl(decl)?;
        }

        Ok(())
    }

    fn decl(&mut self, decl: &Decl) -> Result<(), RuntimeError> {
        match decl {
            Decl::Var(name, init_expr) => {
                let value = if let Some(init_expr) = init_expr {
                    self.expr(init_expr)?
                } else {
                    Value::Nil
                };

                self.environment.define(name.as_ref(), value);
            }
            Decl::Stmt(stmt) => self.stmt(stmt)?,
        }

        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(decls) => {
                self.environment.push();
                for decl in decls {
                    self.decl(decl)?;
                }
                self.environment.pop();
            }
            Stmt::Expr(expr) => {
                self.expr(expr)?;
            }
            Stmt::If(predicate, consequent, alternative) => {
                if self.expr(predicate)?.is_truthy() {
                    self.stmt(consequent)?;
                } else if let Some(alternative) = alternative {
                    self.stmt(alternative)?;
                }
            }
        }

        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(*val),
            Expr::Identifier(name) => self
                .environment
                .get(name.as_ref())
                .ok_or(RuntimeError::UndefinedVariable),
            Expr::Assignment(target, expr) => {
                let right = self.expr(expr)?;
                if let Expr::Identifier(name) = target.as_ref() {
                    self.environment
                        .set(name.as_ref(), right)
                        .ok_or(RuntimeError::UndefinedVariable)?;

                    Ok(right)
                } else {
                    unimplemented!()
                }
            }
            Expr::Binary(op, left, right) if *op == Operator::Or || *op == Operator::And => {
                let left = self.expr(left)?;
                let mut short_circuit = left.is_truthy();
                // For the 'and' operator we want to short circuit if the left
                // operand is not truthy.
                if *op == Operator::And {
                    short_circuit = !short_circuit;
                }

                if short_circuit {
                    Ok(left)
                } else {
                    let right = self.expr(right)?;
                    Ok(right)
                }
            }
            Expr::Binary(op, left, right) => {
                let left = self.expr(left)?;
                let right = self.expr(right)?;

                let value = match op {
                    Operator::Eq => Value::Bool(left == right),
                    Operator::Ne => Value::Bool(left != right),
                    Operator::Lt => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Bool(left < right)
                    }
                    Operator::Gt => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Bool(left > right)
                    }
                    Operator::Le => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Bool(left <= right)
                    }
                    Operator::Ge => {
                        let (left, right) = check_number_operands(&left, &right)?;
                        Value::Bool(left >= right)
                    }
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
                    _ => unreachable!(),
                };

                Ok(value)
            }
            Expr::Unary(op, expr) => {
                let right = self.expr(expr)?;
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
#[derive(Debug, Error, Clone, Copy)]
pub enum RuntimeError {
    #[error("unsupported operand type")]
    InvalidOperand,
    #[error("division by zero is undefined")]
    DivisionByZero,
    #[error("variable is not defined")]
    UndefinedVariable,
}
