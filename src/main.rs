mod expr;
mod parser;
mod scanner;
mod stmt;
use parser::Parser;
use scanner::{ParseError, Scanner};
use std::{
    env, fs,
    io::{self, BufRead},
    process,
};

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
                let _ = run(line);
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
            if exec.is_err() {
                process::exit(65)
            };
        }
        Err(error) => {
            eprintln!("Error reading file: {}", error);
        }
    }
}

fn run(source: String) -> Result<(), ParseError> {
    let mut scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            let stm = parser.parse();
            println!("{:?}", stm);
            Ok(())
        }
        Err(error) => {
            eprintln!("{error}",);
            Err(error)
        }
    }

    // For now, just print the tokens.
}
