/*
Abstract Syntax Tree
*/

use std::fmt::{Debug, Display};

use super::{runner::ReducerArg, token::Token};

pub type ExpressionReducer<T> = fn(args: ReducerArg<T>) -> ASTNode<T>;

pub fn never_reducer<T: RuntimeValue>(_: ReducerArg<T>) -> ASTNode<T> {
    panic!("Reach a reducer that should never be reached")
}

pub fn value_reducer<T: RuntimeValue>(mut args: ReducerArg<T>) -> ASTNode<T> {
    args.val()
}

pub trait RuntimeValue: Debug + Display + From<Token> {}

pub enum ASTNode<T: RuntimeValue> {
    Token(Token),
    ActionExpression(&'static str, Box<dyn FnMut() -> Result<ASTNode<T>, String>>),
    Value(T),
}

impl<T: RuntimeValue> ASTNode<T> {
    pub fn evaluate(self) -> Result<ASTNode<T>, String> {
        match self {
            ASTNode::ActionExpression(_, mut action) => action(),
            ASTNode::Token(token) => Ok(ASTNode::Value(T::from(token))),
            _ => Ok(self),
        }
    }
}

impl<T: RuntimeValue> Display for ASTNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{}", val)?,
        }
        Ok(())
    }
}

impl<T: RuntimeValue> Debug for ASTNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{:?}", val)?,
        }
        Ok(())
    }
}
