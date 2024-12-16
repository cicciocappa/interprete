// Define an error type for scanner errors.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(line: usize, message: String) -> Self {
        ParseError { line, message }
    }
}

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    // Implement ToString
    pub fn to_string(&self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }

    // New method to check if the token is of a specific type.
    pub fn is_type(&self, token_type: TokenType) -> bool {
        self.token_type == token_type
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(token) => tokens.push(token), // Add the token to the vector of tokens
                Err(error) => return Err(error), // If there is an error, return the error.
            };
        }
        tokens.push(Token::new(TokenType::Eof, String::new(), None, self.line));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<Token, ParseError> {
        let c = self.advance();
        match c {
            '(' => Ok(self.create_token(TokenType::LeftParen)),
            ')' => Ok(self.create_token(TokenType::RightParen)),
            _ => {
                Err(ParseError::new(
                    self.line,
                    format!("Unexpected character: {}", c),
                )) // Example of error handling
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        match c {
            Some(c) => c,
            None => ' ',
        }
    }

    fn create_token(&self, token_type: TokenType) -> Token {
        self.create_token_with_literal(token_type, None)
    }

    fn create_token_with_literal(&self, token_type: TokenType, literal: Option<Literal>) -> Token{
        let lexeme = self.source[self.start..self.current].to_owned();
        Token::new(token_type, lexeme, literal, self.line)
    }
}
