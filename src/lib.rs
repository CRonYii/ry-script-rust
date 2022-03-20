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

pub fn init_math_script_parser() -> Result<ScriptParser, String> {
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
    let grammar_set = GrammarSet::from(grammars, terminal_symbols)?;
    let lr_parser = LRParser::lr0_parser(grammar_set)?;
    #[cfg(debug_assertions)]
    println!("{}", lr_parser);
    Ok(ScriptParser {
        lexer: Lexer::new(),
        lr_parser,
    })
}