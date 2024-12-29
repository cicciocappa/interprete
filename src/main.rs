mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod environment;

use interpreter::{Interpreter, RuntimeError};
use parser::Parser;
use scanner::{ParseError, Scanner};
use std::{
    env, fs,
    io::{self, BufRead},
    process,
};

// Define your generic error type
#[derive(Debug)]
pub enum InterpreterError {
    Parse(ParseError),
    Runtime(RuntimeError),
}

// Implement the `Display` trait for better error messages
impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Parse(err) => write!(f, "Parse error: {}", err),
            InterpreterError::Runtime(err) => write!(f, "Runtime error: {}", err),
        }
    }
}

// Implement `From` trait for automatic conversion
impl From<ParseError> for InterpreterError {
    fn from(err: ParseError) -> Self {
        InterpreterError::Parse(err)
    }
}

impl From<RuntimeError> for InterpreterError {
    fn from(err: RuntimeError) -> Self {
        InterpreterError::Runtime(err)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        println!("Error: Too many arguments");
        process::exit(64);
    }
}

fn run_prompt() {
    let stdin = io::stdin();
    // Lock the standard input handle and wrap it in a buffered reader
    let handle = stdin.lock();
    let reader = io::BufReader::new(handle);

    // Iterate over the lines of input
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                // Process the line
                let exec = run(line);
                if let Err(e) = exec {
                    println!("{e}");
                };
            }
            Err(e) => {
                // Handle the error
                eprintln!("Error reading line: {}", e);
                break; // Exit the loop on error
            }
        }
    }
}

fn run_file(file_path: &str) {
    match fs::read_to_string(file_path) {
        Ok(source) => {
            let exec = run(source);
            if let Err(e) = exec {
                println!("{e}");
                process::exit(65)
            };
        }
        Err(error) => {
            eprintln!("Error reading file: {}", error);
        }
    }
}

fn run(source: String) -> Result<(), InterpreterError> {
    let mut interpreter = Interpreter::new();
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    interpreter.interpret(&program)?;
    Ok(())
}
