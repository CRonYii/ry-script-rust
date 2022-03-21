/*
Abstract Syntax Tree
*/

use std::fmt::Display;

use super::token::Token;

pub type GrammarReducer = Vec<ExpressionReducer>;
pub type ExpressionReducer = fn (args: Vec<ASTNode>) -> ASTNode;

#[derive(Debug)]
pub enum ASTNode {
    ValueNode(Token),
    ExpressionNode(Vec<ASTNode>),
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::ValueNode(token) => write!(f, "{:?}", token.r#type)?,
            ASTNode::ExpressionNode(nodes) => {
                write!(f, "(")?;
                for node in nodes {
                    write!(f, "{},", node)?;
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}
