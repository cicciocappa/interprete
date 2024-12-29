use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    environment::Environment,
    expr::{self, Expr},
    scanner::{LiteralValue, Token, TokenType},
    stmt::Stmt,
};

// Define an error type for scanner errors.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    DivisionByZero(Token),
    UndefinedVariable(Token),
    UnexpectedType(Token, String),
    InvalidOperand(Token, String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::DivisionByZero(token) => {
                write!(
                    f,
                    "Line {}: Runtime Error:Division by zero: {}",
                    token.line, token.lexeme
                )
            }
            RuntimeError::UndefinedVariable(token) => {
                write!(
                    f,
                    "Line {}: Runtime Error: Undefined variable '{}'",
                    token.line, token.lexeme
                )
            }
            RuntimeError::UnexpectedType(token, message) => {
                write!(
                    f,
                    "Line {}: Runtime Error: Unexpected type for '{}': {}",
                    token.line, token.lexeme, message
                )
            }
            RuntimeError::InvalidOperand(token, message) => {
                write!(
                    f,
                    "Line {}: Runtime Error: Invalid operand for '{}': {}",
                    token.line, token.lexeme, message
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), RuntimeError> {
        match statement {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", self.stringify(value));
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => LiteralValue::Nil,
                };
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block(statements) => {
                let previous = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                ))));
                let result = self.execute_block(statements);
                self.environment = previous; // Restore previous environment
                result
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let value = self.evaluate(condition)?;
                if self.is_truthy(&value) {
                    self.execute(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.execute(else_stmt)
                } else {
                    Ok(())
                }
            }
            Stmt::While(condition, body) => {
                loop {
                    let value = self.evaluate(condition)?;
                    if !self.is_truthy(&value) {
                        break;
                    }
                    self.execute(body)?;
                }

                Ok(())
            }
            _ => !unreachable!(),
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<LiteralValue, RuntimeError> {
        //println!("Evaluating: {expression:?}");
        match expression {
            Expr::Literal(value) => Ok(value.clone().unwrap_or(LiteralValue::Nil)),
            Expr::Grouping(expr) => self.evaluate(expr),

            Expr::Unary(operator, right) => {
                let right_val = self.evaluate(&right)?;
                match operator.token_type {
                    TokenType::Minus => match right_val {
                        LiteralValue::Number(num) => Ok(LiteralValue::Number(-num)),
                        _ => Err(RuntimeError::UnexpectedType(
                            operator.clone(),
                            "Operand must be a number.".to_string(),
                        )),
                    },
                    TokenType::Bang => Ok(LiteralValue::Boolean(!self.is_truthy(&right_val))),
                    _ => !unreachable!(),
                }
            }
            Expr::Variable(name) => self.lookup_variable(name),
            Expr::Assignment(name, value) => {
                let evaluated_value = self.evaluate(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name, evaluated_value.clone())?;
                Ok(evaluated_value)
            }
            Expr::Logical(left, operator, right) => {
                let left_val = self.evaluate(left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                } else if !self.is_truthy(&left_val) {
                    return Ok(left_val);
                }

                self.evaluate(right)
            }
            Expr::Binary(left, operator, right) => {
                let left_val = self.evaluate(&left)?;
                let right_val = self.evaluate(&right)?;
                match operator.token_type {
                    TokenType::Minus => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Number(a - b))
                    }
                    TokenType::Slash => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        if b == 0.0 {
                            Err(RuntimeError::DivisionByZero(operator.clone()))
                        } else {
                            Ok(LiteralValue::Number(a / b))
                        }
                    }
                    TokenType::Star => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Number(a * b))
                    }

                    TokenType::Plus => match (&left_val, &right_val) {
                        (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                            Ok(LiteralValue::Number(l + r))
                        }
                        (LiteralValue::String(l), LiteralValue::String(r)) => {
                            Ok(LiteralValue::String(format!("{}{}", l, r)))
                        }
                        _ => {
                            return Err(RuntimeError::InvalidOperand(
                                operator.clone(),
                                "Operands must be two numbers or two strings.".to_string(),
                            ))
                        }
                    },
                    TokenType::Greater => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Boolean(a > b))
                    }
                    TokenType::GreaterEqual => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Boolean(a >= b))
                    }
                    TokenType::Less => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Boolean(a < b))
                    }
                    TokenType::LessEqual => {
                        let (a, b) = self.check_number_operands(operator, &left_val, &right_val)?;
                        Ok(LiteralValue::Boolean(a >= b))
                    }
                    TokenType::BangEqual => {
                        Ok(LiteralValue::Boolean(!self.is_equal(&left_val, &right_val)))
                    }
                    TokenType::EqualEqual => {
                        Ok(LiteralValue::Boolean(self.is_equal(&left_val, &right_val)))
                    }

                    _ => !unreachable!(),
                }
            }
            _ => !unreachable!(),
        }
    }
    fn lookup_variable(&self, name: &Token) -> Result<LiteralValue, RuntimeError> {
        self.environment.borrow().get(name)
    }
    fn check_number_operands(
        &self,
        operator: &Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<(f64, f64), RuntimeError> {
        match (left, right) {
            (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok((*a, *b)),
            _ => Err(RuntimeError::UnexpectedType(
                operator.clone(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    fn is_truthy(&self, value: &LiteralValue) -> bool {
        match value {
            LiteralValue::Nil => false,
            LiteralValue::Boolean(b) => *b,
            /*LiteralValue::String(s) => !s.is_empty(),*/
            _ => true,
        }
    }

    fn is_equal(&self, a: &LiteralValue, b: &LiteralValue) -> bool {
        match (a, b) {
            (LiteralValue::Nil, LiteralValue::Nil) => true,
            (LiteralValue::Number(na), LiteralValue::Number(nb)) => na == nb,
            (LiteralValue::String(sa), LiteralValue::String(sb)) => sa == sb,
            (LiteralValue::Boolean(ba), LiteralValue::Boolean(bb)) => ba == bb,
            _ => false,
        }
    }

    fn stringify(&self, value: LiteralValue) -> String {
        match value {
            LiteralValue::Nil => "nil".to_string(),
            LiteralValue::Number(n) => format!("{}", n),
            LiteralValue::String(s) => s,
            LiteralValue::Boolean(b) => format!("{}", b),
        }
    }
}
