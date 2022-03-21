pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{never_reducer, value_reducer, ASTNode};
use script::grammar::TerminalSymbolDef;
use script::runner::{GrammarRule, ReducerArg, ScriptRunner};
use script::token::TokenType;

pub fn multiply_reducer(mut args: ReducerArg) -> ASTNode {
    ASTNode::ActionExpression(
        "a * b",
        Box::new(move || {
            let (lhs, _, rhs) = (args.eval()?, args.skip(), args.eval()?);
            #[cfg(feature = "debug_ast")]
            println!("eval {} * {}", lhs, rhs);
            match lhs {
                ASTNode::Integer(lhs) => {
                    let val = lhs * rhs.int()?;
                    Ok(ASTNode::Integer(val))
                },
                ASTNode::Float(lhs) => {
                    let val = lhs * rhs.float()?;
                    Ok(ASTNode::Float(val))
                },
                _ => Err(format!("Runtime Error: {} does not support multiplication", lhs))
            }
        }),
    )
}

pub fn init_math_script_parser() -> Result<ScriptRunner, String> {
    let grammars: Vec<GrammarRule> = vec![
        GrammarRule("S -> E EOF", never_reducer),
        GrammarRule("E -> E * E", multiply_reducer),
        GrammarRule("E -> int", value_reducer),
        GrammarRule("E -> float", value_reducer),
    ];
    let terminal_symbols = [
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("int", TokenType::Integer),
        TerminalSymbolDef("float", TokenType::Float),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];

    Ok(ScriptRunner::from(grammars, &terminal_symbols)?)
}
