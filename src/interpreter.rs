use crate::{
    expr::{self, Expr},
    scanner::{LiteralValue, Token, TokenType},
};

// Define an error type for scanner errors.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    DivisionByZero(Token),
    UndefinedVariable(Token),
    UnexpectedType(Token, String),
   
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::DivisionByZero(token) => {
                write!(f, "Line {}: Runtime Error:Division by zero: {}", token.line, token.lexeme)
            }
            RuntimeError::UndefinedVariable(token) => {
                write!(f, "Line {}: Runtime Error: Undefined variable '{}'", token.line, token.lexeme)
            }
            RuntimeError::UnexpectedType(token, message) => {
                write!(
                    f,
                    "Line {}: Runtime Error: Unexpected type for '{}': {}",
                    token.line, token.lexeme, message
                )
            }
            
        }
    }
}

impl std::error::Error for RuntimeError {}

pub struct Interpreter {}

impl Interpreter {
    fn evaluate(&mut self, expression: &Expr) -> Result<LiteralValue, RuntimeError> {
        match expression {
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
                    TokenType::Bang => {}
                    _ => !unreachable!(),
                }
            }
            _=> !unreachable!(),
        }
    }
}
