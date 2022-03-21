pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{never_reducer, ASTNode};
use script::grammar::TerminalSymbolDef;
use script::runner::{GrammarRule, ScriptRunner};
use script::token::TokenType;

pub fn multiply_reducer(args: Vec<ASTNode>) -> ASTNode {
    ASTNode::ActionExpression(
        "a * b",
        Box::new(move || {
            if let [left, _, right] = &args[..] {
                match left {
                    ASTNode::Integer(left) => Ok(ASTNode::Integer(left * right.int())),
                    _ => Err(format!("{} does not support multiplication", left)),
                }
            } else {
                Err(format!("multiply reducer expects exactly 3 args"))
            }
        }),
    )
}

pub fn add_reducer(args: Vec<ASTNode>) -> ASTNode {
    ASTNode::ActionExpression(
        "a + b",
        Box::new(move || {
            if let [left, _, right] = &args[..] {
                match left {
                    ASTNode::Integer(left) => Ok(ASTNode::Integer(left * right.int())),
                    _ => Err(format!("{} does not support addition", left)),
                }
            } else {
                Err(format!("add reducer expects exactly 3 args"))
            }
        }),
    )
}

pub fn value_reducer(mut args: Vec<ASTNode>) -> ASTNode {
    args.pop().unwrap().value()
}

pub fn init_math_script_parser() -> Result<ScriptRunner, String> {
    let grammars: Vec<GrammarRule> = vec![
        GrammarRule("S -> E EOF", never_reducer),
        GrammarRule("E -> E * B", multiply_reducer),
        GrammarRule("E -> E + B", add_reducer),
        GrammarRule("E -> B", value_reducer),
        GrammarRule("B -> int", value_reducer),
    ];
    let terminal_symbols = [
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("+", TokenType::Plus),
        TerminalSymbolDef("int", TokenType::Integer),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];

    Ok(ScriptRunner::from(grammars, &terminal_symbols)?)
}
