/*
Abstract Syntax Tree
*/

use std::fmt::{Debug, Display};

use super::error::RuntimeError;
use super::runner::ReducerArg;
use super::token::{ParserToken, Token};

pub type ExpressionReducer<ENV, T, R, E> =
    fn(args: ReducerArg<ENV, T, R, E>) -> ASTNode<ENV, T, R, E>;

pub fn never_reducer<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError>(
    _: ReducerArg<ENV, T, R, E>,
) -> ASTNode<ENV, T, R, E> {
    panic!("Reach a reducer that should never be reached")
}

pub fn value_reducer<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError>(
    mut args: ReducerArg<ENV, T, R, E>,
) -> ASTNode<ENV, T, R, E> {
    args.val()
}

pub trait RuntimeValue<T: ParserToken<T>>: Debug + Display + From<Token<T>> {}

pub enum ASTNode<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> {
    Token(Token<T>),
    ActionExpression(
        &'static str,
        Box<dyn FnMut(&mut ENV) -> Result<ASTNode<ENV, T, R, E>, E>>,
    ),
    Value(R),
}

impl<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> ASTNode<ENV, T, R, E> {
    pub fn evaluate(self, env: &mut ENV) -> Result<ASTNode<ENV, T, R, E>, E> {
        match self {
            ASTNode::ActionExpression(_, mut action) => action(env),
            ASTNode::Token(token) => Ok(ASTNode::Value(R::from(token))),
            _ => Ok(self),
        }
    }
}

impl<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> Display
    for ASTNode<ENV, T, R, E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{}", val)?,
        }
        Ok(())
    }
}

impl<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> Debug for ASTNode<ENV, T, R, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Value(val) => write!(f, "{:?}", val)?,
        }
        Ok(())
    }
}
