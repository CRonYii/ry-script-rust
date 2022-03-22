/*
Abstract Syntax Tree
*/

use std::fmt::{Debug, Display};

use super::error::RuntimeError;
use super::{runner::ReducerArg, token::Token};

pub type ExpressionReducer<T, E> = fn(args: ReducerArg<T, E>) -> ASTNode<T, E>;

pub fn never_reducer<T: RuntimeValue, E: RuntimeError>(_: ReducerArg<T, E>) -> ASTNode<T, E> {
    panic!("Reach a reducer that should never be reached")
}

pub fn value_reducer<T: RuntimeValue, E: RuntimeError>(
    mut args: ReducerArg<T, E>,
) -> ASTNode<T, E> {
    args.val()
}

pub trait RuntimeValue: Debug + Display + From<Token> {}

pub enum ASTNode<T: RuntimeValue, E: RuntimeError> {
    Token(Token),
    ActionExpression(&'static str, Box<dyn FnMut() -> Result<ASTNode<T, E>, E>>),
    Value(T),
}

impl<T: RuntimeValue, E: RuntimeError> ASTNode<T, E> {
    pub fn evaluate(self) -> Result<ASTNode<T, E>, E> {
        match self {
            ASTNode::ActionExpression(_, mut action) => action(),
            ASTNode::Token(token) => Ok(ASTNode::Value(T::from(token))),
            _ => Ok(self),
        }
    }
}

impl<T: RuntimeValue, E: RuntimeError> Display for ASTNode<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{}", val)?,
        }
        Ok(())
    }
}

impl<T: RuntimeValue, E: RuntimeError> Debug for ASTNode<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{:?}", val)?,
        }
        Ok(())
    }
}
