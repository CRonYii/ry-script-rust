/*
Abstract Syntax Tree
*/

use std::fmt::Display;

use super::token::Token;

pub type GrammarReducer = Vec<ExpressionReducer>;
pub type ExpressionReducer = fn(args: Vec<ASTNode>) -> ASTNode;

pub enum ASTNode {
    Token(Token),
    ActionExpression(&'static str, Box<dyn Fn() -> ASTNode>),
    Integer(i64),
}

impl ASTNode {
    pub fn value(self) -> ASTNode {
        match self {
            ASTNode::Token(token) => token.value(),
            _ => self,
        }
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Token(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ActionExpression(name, _) => write!(f, "{:?}", name)?,
            ASTNode::Integer(num) => write!(f, "{}", num)?,
        }
        Ok(())
    }
}
