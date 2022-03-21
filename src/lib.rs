pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{never_reducer, value_reducer, ASTNode, RuntimeValue};
use script::grammar::TerminalSymbolDef;
use script::runner::{GrammarRule, ReducerArg, ScriptRunner};
use script::token::{Token, TokenType};

impl RuntimeValue for Value {}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
}

impl std::convert::From<Token> for Value {
    fn from(token: Token) -> Self {
        match token.r#type {
            TokenType::Integer => Value::Integer(token.value.parse().unwrap()),
            TokenType::Float => Value::Float(token.value.parse().unwrap()),
            _ => panic!("Parser Error: Unexpcted token {:?} cannot be converted to a value", token.r#type),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(num) => write!(f, "{}", num)?,
            Value::Float(num) => write!(f, "{}", num)?,
        }
        Ok(())
    }
}

impl Value {
    pub fn int(self) -> Result<i64, String> {
        match self {
            Value::Integer(val) => Ok(val),
            Value::Float(val) => Ok(val as i64),
            // _ => Err(format!("Runtime Error: Cannot cast {:?} to int", self)),
        }
    }

    pub fn float(self) -> Result<f64, String> {
        match self {
            Value::Integer(val) => Ok(val as f64),
            Value::Float(val) => Ok(val),
            // _ => Err(format!("Runtime Error: Cannot cast {:?} to int", self)),
        }
    }
}

impl Value {
    fn mul(self, rhs: Value) -> Result<Value, String> {
        match self {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Ok(Value::Integer(lhs * rhs)),
                Value::Float(rhs) => Ok(Value::Float((lhs as f64) * rhs)),
            },
            Value::Float(lhs) => {
                let val = lhs * rhs.float()?;
                Ok(Value::Float(val))
            }
        }
    }

    fn add(self, rhs: Value) -> Result<Value, String> {
        match self {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Ok(Value::Integer(lhs + rhs)),
                Value::Float(rhs) => Ok(Value::Float((lhs as f64) + rhs)),
            },
            Value::Float(lhs) => {
                let val = lhs + rhs.float()?;
                Ok(Value::Float(val))
            }
        }
    }
}

fn multiply_reducer(mut args: ReducerArg<Value>) -> ASTNode<Value> {
    ASTNode::ActionExpression(
        "a * b",
        Box::new(move || match (args.eval_skip(1)?, args.eval()?) {
            (ASTNode::Value(lhs), ASTNode::Value(rhs)) => {
                #[cfg(feature = "debug_ast")]
                println!("eval {:?} * {:?}", lhs, rhs);
                Ok(ASTNode::Value(lhs.mul(rhs)?))
            }
            _ => panic!("Parse Error: Reducer expected value but non-value were given"),
        }),
    )
}

fn add_reducer(mut args: ReducerArg<Value>) -> ASTNode<Value> {
    ASTNode::ActionExpression(
        "a + b",
        Box::new(move || match (args.eval_skip(1)?, args.eval()?) {
            (ASTNode::Value(lhs), ASTNode::Value(rhs)) => {
                #[cfg(feature = "debug_ast")]
                println!("eval {:?} + {:?}", lhs, rhs);
                Ok(ASTNode::Value(lhs.add(rhs)?))
            }
            _ => panic!("Parse Error: Reducer expected value but non-value were given"),
        }),
    )
}

pub fn init_math_script_parser() -> Result<ScriptRunner<Value>, String> {
    let grammars: Vec<GrammarRule<Value>> = vec![
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
