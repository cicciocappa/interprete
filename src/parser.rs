use crate::{
    expr::Expr,
    scanner::{LiteralValue, ParseError, Token, TokenType},
    stmt::Stmt,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) ->  Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)

    }
    fn comparison(&mut self) ->  Result<Expr, ParseError>  {
        let mut expr = self.term()?;
        while self.match_token(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }
    fn term(&mut self) ->   Result<Expr, ParseError>   {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr) 
    }
    fn factor(&mut self) ->   Result<Expr, ParseError>   {
        let mut expr = self.unary()?;
        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr) 
    }
    fn unary(&mut self) ->   Result<Expr, ParseError>   {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }
    fn primary(&mut self) ->   Result<Expr, ParseError>   {
        if self.match_token(&[TokenType::False]) {
            Ok(Expr::Literal(Some(LiteralValue::Boolean(false))))
        } else if self.match_token(&[TokenType::True]) {
            Ok(Expr::Literal(Some(LiteralValue::Boolean(true))))
        } else if self.match_token(&[TokenType::Nil]) {
            Ok(Expr::Literal(None))
        } else if self.match_token(&[TokenType::Number, TokenType::String]) {
            match self.previous().token_type {
                TokenType::Number => {
                    if let Ok(num) = self.previous().lexeme.parse::<f64>() {
                        Ok(Expr::Literal(Some(LiteralValue::Number(num))))
                    } else {
                        Err(ParseError::UnexpectedToken(self.previous().clone(), "Invalid number format".to_string()))
                    }
                }
                TokenType::String => Ok(Expr::Literal(Some(LiteralValue::String(
                    self.previous().lexeme[1..self.previous().lexeme.len() - 1].to_string(),
                )))),
                _ => unreachable!(),
            }
        } else if self.match_token(&[TokenType::This]) {
            Ok(Expr::This(self.previous().clone()))
        }
        else if self.match_token(&[TokenType::Super]) {
            let keyword = self.previous().clone();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
            Ok(Expr::Super(keyword, method))
        }
        else if self.match_token(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous().clone()))
        } else if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(ParseError::UnexpectedToken(self.peek().clone(), "Expect expression.".to_string()))
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        if types.iter().any(|&t| self.check(t)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::ExpectedToken(token_type, self.peek().clone()))
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
