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

pub enum ASTNode {
    Token(Token),
    ActionExpression(&'static str, Box<dyn FnMut() -> Result<ASTNode, String>>),
    Integer(i64),
    Float(f64),
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
            ASTNode::Token(token) => token.value(),
            _ => self,
        }
    }

    pub fn int(self) -> Result<i64, String> {
        match self {
            ASTNode::Integer(val) => Ok(val),
            ASTNode::Float(val) => Ok(val as i64),
            _ => Err(format!("Runtime Error: Cannot cast {:?} to int", self)),
        }
    }

    pub fn float(self) -> Result<f64, String> {
        match self {
            ASTNode::Integer(val) => Ok(val as f64),
            ASTNode::Float(val) => Ok(val),
            _ => Err(format!("Runtime Error: Cannot cast {:?} to int", self)),
        }
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Integer(num) => write!(f, "{}", num)?,
            ASTNode::Float(num) => write!(f, "{}", num)?,
        }
        Ok(())
    }
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Integer(num) => write!(f, "int({})", num)?,
            ASTNode::Float(num) => write!(f, "float({})", num)?,
        }
        Ok(())
    }
}
