use std::collections::HashMap;
use std::io::*;

mod math;
mod script;

#[cfg(test)]
mod math_tests;

use script::grammar::GrammarSet;
use script::lrparser::lr0_parse;
use script::token::TokenType;

pub use crate::math::matrix::*;
pub use crate::script::lexer::Lexer;

fn init_math_script_parser() {
    let grammars = vec![
        "S -> E EOF",
        "E -> E * B",
        "E -> E + B",
        "E -> B",
        "B -> id",
        "B -> num",
    ];
    let mut terminal_symbols = HashMap::new();
    terminal_symbols.insert("*", TokenType::Multiply);
    terminal_symbols.insert("+", TokenType::Plus);
    terminal_symbols.insert("id", TokenType::Identifier);
    terminal_symbols.insert("num", TokenType::Number);
    terminal_symbols.insert("EOF", TokenType::EOF);
    let grmmar_set = match GrammarSet::from(grammars, terminal_symbols) {
        Ok(grammar_set) => grammar_set,
        Err(error) => panic!("Grammar parser error: {}", error),
    };
    lr0_parse(&grmmar_set).unwrap();
}

fn main() {
    init_math_script_parser();
    let mut lexer = Lexer::new();
    let mut line = String::new();
    loop {
        print!("> ");
        match stdout().flush() {
            Ok(_) => (),
            Err(error) => eprintln!("stdout error: {}", error),
        }
        let _ = stdin().read_line(&mut line).unwrap();
        if line.starts_with(".exit") {
            break;
        }
        match lexer.parse(&line) {
            Ok(tokens) => {
                #[cfg(debug_assertions)]
                println!("{}", tokens)
            }
            Err(err) => eprintln!("{}", err),
        }
        line.clear();
    }
}
