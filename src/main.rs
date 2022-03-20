use std::io;

mod math;
mod script;

#[cfg(test)]
mod math_tests;

use script::grammar::{GrammarSet, TerminalSymbolDef};
use script::lrparser::LRParser;
use script::token::TokenType;

pub use crate::math::matrix::*;
pub use crate::script::lexer::Lexer;

pub struct ScriptParser {
    lexer: Lexer,
    lr_parser: LRParser,
}

impl ScriptParser {
    pub fn parse(&mut self, input: &String) -> Result<(), String> {
        let tokens = self.lexer.parse(input)?;
        #[cfg(debug_assertions)]
        println!("{}", tokens);
        self.lr_parser.parse(tokens)?;
        Ok(())
    }
}

fn init_math_script_parser() -> ScriptParser {
    let grammars = vec![
        "S -> E EOF",
        "E -> E * B",
        "E -> E + B",
        "E -> B",
        "B -> id",
        "B -> num",
    ];
    let terminal_symbols = vec![
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("+", TokenType::Plus),
        TerminalSymbolDef("id", TokenType::Identifier),
        TerminalSymbolDef("num", TokenType::Number),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];
    let grammar_set = match GrammarSet::from(grammars, terminal_symbols) {
        Ok(grammar_set) => grammar_set,
        Err(error) => panic!("Grammar parser error: {}", error),
    };
    let lr_parser = LRParser::lr0_parser(grammar_set).unwrap();
    #[cfg(debug_assertions)]
    println!("{}", lr_parser);
    ScriptParser {
        lexer: Lexer::new(),
        lr_parser,
    }
}

fn main() {
    use std::io::Write;
    let mut lr_parser = init_math_script_parser();
    let mut line = String::new();
    loop {
        print!("> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(error) => eprintln!("stdout error: {}", error),
        }
        let _ = io::stdin().read_line(&mut line).unwrap();
        if line.starts_with(".exit") {
            break;
        }
        match lr_parser.parse(&line) {
            Ok(_) => (),
            Err(err) => eprintln!("{}", err),
        }
        line.clear();
    }
}
