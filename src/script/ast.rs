/*
Abstract Syntax Tree
*/

use std::fmt::{Debug, Display};

use super::{runner::ReducerArg, token::Token};

pub type ExpressionReducer = fn(args: ReducerArg) -> ASTNode;

pub fn never_reducer(_: ReducerArg) -> ASTNode {
    panic!("Reach a reducer that should never be reached")
}

pub fn value_reducer(mut args: ReducerArg) -> ASTNode {
    args.val()
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(num) => write!(f, "{}", num)?,
            Value::Float(num) => write!(f, "{}", num)?,
        }
        Ok(())
    }
}

pub enum ASTNode {
    Token(Token),
    ActionExpression(&'static str, Box<dyn FnMut() -> Result<ASTNode, String>>),
    Value(Value),
}

impl ASTNode {
    pub fn evaluate(self) -> Result<ASTNode, String> {
        match self {
            ASTNode::ActionExpression(_, mut action) => action(),
            _ => Ok(self),
        }
    }

    pub fn value(self) -> ASTNode {
        match self {
            ASTNode::Token(token) => ASTNode::Value(token.value()),
            _ => self,
        }
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{}", val)?,
        }
        Ok(())
    }
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{:?}", val)?,
        }
        Ok(())
    }
}
