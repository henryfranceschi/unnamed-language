use std::collections::HashMap;

use crate::parser::ast::{Expr, Operator, Stmt};

use self::value::Value;

mod environment;
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
            Stmt::VarDecl(name, init_expr) => {
                let value = if let Some(init_expr) = init_expr {
                    match self.interpret_expr(init_expr) {
                        Ok(value) => {
                            println!("value: {:?}", value);
                            value
                        }
                        Err(err) => {
                            eprintln!("runtime error: {}", err.message());
                            return;
                        }
                    }
                } else {
                    Value::Nil
                };

                self.globals.insert(name.to_owned(), value);
            }
        };
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(*val),
            Expr::Identifier(name) => {
                if let Some(value) = self.globals.get(name) {
                    Ok(*value)
                } else {
                    Err(RuntimeError::UndefinedVariable)
                }
            }
            Expr::Assignment(target, expr) => {
                let right = self.interpret_expr(expr)?;
                if let Expr::Identifier(name) = target.as_ref() {
                    if self.globals.contains_key(name) {
                        self.globals.insert(name.to_owned(), right);
                        Ok(right)
                    } else {
                        Err(RuntimeError::UndefinedVariable)
                    }
                } else {
                    unimplemented!()
                }
            }
            Expr::Binary(Operator::Or, left, right) => {
                let left = self.interpret_expr(left)?;
                if left.is_truthy() {
                    Ok(left)
                } else {
                    let right = self.interpret_expr(right)?;
                    Ok(right)
                }
            }
            Expr::Binary(Operator::And, left, right) => {
                let left = self.interpret_expr(left)?;
                if !left.is_truthy() {
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
