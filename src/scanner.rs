use std::collections::HashMap;
use std::sync::LazyLock;

pub static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    //println!("Initializing shared HashMap!");
    let mut map = HashMap::new();
    map.insert("and", TokenType::And);
    map.insert("class", TokenType::Class);
    map.insert("else", TokenType::Else);
    map.insert("false", TokenType::False);
    map.insert("for", TokenType::For);
    map.insert("fun", TokenType::Fun);
    map.insert("if", TokenType::If);
    map.insert("nil", TokenType::Nil);
    map.insert("or", TokenType::Or);
    map.insert("print", TokenType::Print);
    map.insert("return", TokenType::Return);
    map.insert("super", TokenType::Super);
    map.insert("this", TokenType::This);
    map.insert("true", TokenType::True);
    map.insert("var", TokenType::Var);
    map.insert("while", TokenType::While);
    map
});

// Define an error type for scanner errors.
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedToken(Token, String),
    ExpectedToken(TokenType, Token),
    UnterminatedString(usize),
    EndOfFile,
    // Add more specific parsing errors as needed
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedCharacter(character, line) => {
                write!(f, "Line {}: Unexpected character '{}'", line, character)
            }
            ParseError::UnexpectedToken(token, message) => {
                write!(
                    f,
                    "Line {}: Unexpected token '{}': {}",
                    token.line, token.lexeme, message
                )
            }
            ParseError::ExpectedToken(expected, found) => {
                write!(
                    f,
                    "Line {}: Expected token '{:?}', but found '{}'",
                    found.line, expected, found.lexeme
                )
            }
            ParseError::UnterminatedString(line) => {
                write!(f, "Line {}: Unterminated string", line)
            }
            ParseError::EndOfFile => write!(f, "Unexpected end of file"),
        }
    }
}

impl std::error::Error for ParseError {}

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
    pub literal: Option<LiteralValue>,
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<LiteralValue>,
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
pub enum LiteralValue {
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
                Ok(Some(token)) => tokens.push(token), // Add the token to the vector of tokens
                Ok(None) => continue,
                Err(error) => return Err(error), // If there is an error, return the error.
            };
        }
        tokens.push(Token::new(TokenType::Eof, String::new(), None, self.line));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<Option<Token>, ParseError> {
        let c = self.advance();
        match c {
            '(' => Ok(Some(self.create_token(TokenType::LeftParen))),
            ')' => Ok(Some(self.create_token(TokenType::RightParen))),
            '{' => Ok(Some(self.create_token(TokenType::LeftBrace))),
            '}' => Ok(Some(self.create_token(TokenType::RightBrace))),
            ',' => Ok(Some(self.create_token(TokenType::Comma))),
            '.' => Ok(Some(self.create_token(TokenType::Dot))),
            '-' => Ok(Some(self.create_token(TokenType::Minus))),
            '+' => Ok(Some(self.create_token(TokenType::Plus))),
            ';' => Ok(Some(self.create_token(TokenType::Semicolon))),
            '*' => Ok(Some(self.create_token(TokenType::Star))),
            '!' => {
                if self.match_next('=') {
                    Ok(Some(self.create_token(TokenType::BangEqual)))
                } else {
                    Ok(Some(self.create_token(TokenType::Bang)))
                }
            }
            '=' => {
                if self.match_next('=') {
                    Ok(Some(self.create_token(TokenType::EqualEqual)))
                } else {
                    Ok(Some(self.create_token(TokenType::Equal)))
                }
            }
            '<' => {
                if self.match_next('=') {
                    Ok(Some(self.create_token(TokenType::LessEqual)))
                } else {
                    Ok(Some(self.create_token(TokenType::Less)))
                }
            }
            '>' => {
                if self.match_next('=') {
                    Ok(Some(self.create_token(TokenType::GreaterEqual)))
                } else {
                    Ok(Some(self.create_token(TokenType::Greater)))
                }
            }
            '/' => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(None)
                } else {
                    Ok(Some(self.create_token(TokenType::Slash)))
                }
            }
            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
                Ok(None)
            }
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number()
                } else if is_alpha(c) {
                    self.identifier()
                } else {
                    Err(ParseError::UnexpectedCharacter(c, self.line)) // Example of error handling
                }
            }
        }
    }

    fn identifier(&mut self) -> Result<Option<Token>, ParseError> {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token_type = *KEYWORDS.get(text).unwrap_or(&TokenType::Identifier);
        Ok(Some(self.create_token(token_type)))
    }

    fn number(&mut self) -> Result<Option<Token>, ParseError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current].parse().unwrap();
        Ok(Some(self.create_token_with_literal(
            TokenType::Number,
            Some(LiteralValue::Number(value)),
        )))
    }

    fn string(&mut self) -> Result<Option<Token>, ParseError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(ParseError::UnterminatedString(self.line)); //
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value = self.source[self.start + 1..self.current - 1].to_owned();
        Ok(Some(self.create_token_with_literal(
            TokenType::String,
            Some(LiteralValue::String(value)),
        )))
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
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

    fn create_token_with_literal(
        &self,
        token_type: TokenType,
        literal: Option<LiteralValue>,
    ) -> Token {
        let lexeme = self.source[self.start..self.current].to_owned();
        Token::new(token_type, lexeme, literal, self.line)
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
