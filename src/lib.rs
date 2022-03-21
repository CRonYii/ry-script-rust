pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{never_reducer, value_reducer, ASTNode};
use script::grammar::TerminalSymbolDef;
use script::runner::{GrammarRule, ReducerArg, ScriptRunner};
use script::token::TokenType;

fn multiply_reducer(mut args: ReducerArg) -> ASTNode {
    ASTNode::ActionExpression(
        "a * b",
        Box::new(move || {
            let (lhs, rhs) = (args.eval_skip(1)?, args.eval()?);
            #[cfg(feature = "debug_ast")]
            println!("eval {:?} * {:?}", lhs, rhs);
            match lhs {
                ASTNode::Integer(lhs) => {
                    let val = lhs * rhs.int()?;
                    Ok(ASTNode::Integer(val))
                }
                ASTNode::Float(lhs) => {
                    let val = lhs * rhs.float()?;
                    Ok(ASTNode::Float(val))
                }
                _ => Err(format!(
                    "Runtime Error: {:?} does not support multiplication",
                    lhs
                )),
            }
        }),
    )
}

fn add_reducer(mut args: ReducerArg) -> ASTNode {
    ASTNode::ActionExpression(
        "a + b",
        Box::new(move || {
            let (lhs, rhs) = (args.eval_skip(1)?, args.eval()?);
            #[cfg(feature = "debug_ast")]
            println!("eval {:?} + {:?}", lhs, rhs);
            match lhs {
                ASTNode::Integer(lhs) => {
                    let val = lhs + rhs.int()?;
                    Ok(ASTNode::Integer(val))
                }
                ASTNode::Float(lhs) => {
                    let val = lhs + rhs.float()?;
                    Ok(ASTNode::Float(val))
                }
                _ => Err(format!(
                    "Runtime Error: {:?} does not support addition",
                    lhs
                )),
            }
        }),
    )
}

pub fn init_math_script_parser() -> Result<ScriptRunner, String> {
    let grammars: Vec<GrammarRule> = vec![
        GrammarRule("B -> S EOF", never_reducer),
        GrammarRule("S -> A1", value_reducer),
        GrammarRule("A1 -> A2", value_reducer),
        GrammarRule("A1 -> A1 + A2", add_reducer),
        GrammarRule("A2 -> A3", value_reducer),
        GrammarRule("A2 -> A2 * A3", multiply_reducer),
        GrammarRule("A3 -> Val", value_reducer),
        GrammarRule("Val -> num", value_reducer),
        GrammarRule("num -> int", value_reducer),
        GrammarRule("num -> float", value_reducer),
        GrammarRule("Val -> ( A1 )", |mut args| args.nth_val(1)),
    ];
    let terminal_symbols = [
        TerminalSymbolDef("(", TokenType::LeftParenthese),
        TerminalSymbolDef(")", TokenType::RightParenthese),
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("+", TokenType::Plus),
        TerminalSymbolDef("int", TokenType::Integer),
        TerminalSymbolDef("float", TokenType::Float),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];

    Ok(ScriptRunner::from(grammars, &terminal_symbols)?)
}
