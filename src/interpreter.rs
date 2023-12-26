use self::{environment::Environment, value::Value};
use crate::compiler::parser::ast::{Expr, Operator, Stmt};

mod environment;
pub mod value;

/// Basic treewalk interpreter, will be replaced later by something more efficient.
#[derive(Debug, Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn interpret_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.environment.push();
                for stmt in stmts {
                    self.interpret_stmt(stmt)?;
                }
                self.environment.pop();
            }
            Stmt::Expr(expr) => {
                self.interpret_expr(expr)?;
            }
            Stmt::VarDecl(name, init_expr) => {
                let value = if let Some(init_expr) = init_expr {
                    self.interpret_expr(init_expr)?
                } else {
                    Value::Nil
                };

                self.environment.define(name, value);
            }
            Stmt::If(predicate, consequent, alternative) => {
                if self.interpret_expr(predicate)?.is_truthy() {
                    self.interpret_stmt(consequent)?;
                } else if let Some(alternative) = alternative {
                    self.interpret_stmt(alternative)?;
                }
            }
        }

        Ok(())
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(*val),
            Expr::Identifier(name) => self
                .environment
                .get(name)
                .ok_or(RuntimeError::UndefinedVariable),
            Expr::Assignment(target, expr) => {
                let right = self.interpret_expr(expr)?;
                if let Expr::Identifier(name) = target.as_ref() {
                    self.environment
                        .set(name, right)
                        .ok_or(RuntimeError::UndefinedVariable)
                } else {
                    unimplemented!()
                }
            }
            Expr::Binary(op, left, right) if *op == Operator::Or || *op == Operator::And => {
                let left = self.interpret_expr(left)?;
                let mut short_circuit = left.is_truthy();
                // For the 'and' operator we want to short circuit if the left
                // operand is not truthy.
                if *op == Operator::And {
                    short_circuit = !short_circuit;
                }

                if short_circuit {
                    Ok(left)
                } else {
                    let right = self.interpret_expr(right)?;
                    Ok(right)
                }
            }
            Expr::Binary(op, left, right) => {
                let left = self.interpret_expr(left)?;
                let right = self.interpret_expr(right)?;

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
    UndefinedVariable,
}

impl RuntimeError {
    fn message(self) -> &'static str {
        match self {
            RuntimeError::InvalidOperand => "unsupported operand type",
            RuntimeError::DivisionByZero => "division by zero is undefined",
            RuntimeError::UndefinedVariable => "variable is not defined",
        }
    }
}
