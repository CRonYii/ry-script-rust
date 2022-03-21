use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::vec::Vec;

use super::ast::ASTNode;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TokenType {
    Identifier,
    Integer,
    Float,
    String,
    If,
    Else,
    While,
    Break,
    Continue,
    Function,
    Return,
    True,
    False,
    None,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Not,
    Assignment,
    Equal,
    Inequal,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Dot,
    Comma,
    SemiColon,
    LeftParenthese,
    RightParenthese,
    LeftCurlyBracket,
    RightCurlyBracket,
    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub r#type: TokenType,
    pub value: String,
}

impl Token {
    pub fn value(self) -> ASTNode {
        match self.r#type {
            TokenType::Integer => ASTNode::Integer(self.value.parse().unwrap()),
            TokenType::Float => ASTNode::Float(self.value.parse().unwrap()),
            _ => panic!("{:?} cannot be converted to a value", self.r#type),
        }
    }
}

impl TokenType {
    pub fn entity(self, value: String) -> Token {
        Token {
            r#type: self,
            value,
        }
    }
}

pub struct TokenMap {
    operator_map: HashMap<&'static str, TokenType>,
    keyword_map: HashMap<&'static str, TokenType>,
    signs_chars_set: HashSet<char>,
}

impl TokenMap {
    pub fn init() -> TokenMap {
        let mut set = TokenMap {
            operator_map: HashMap::new(),
            keyword_map: HashMap::new(),
            signs_chars_set: HashSet::new(),
        };
        // operators
        set.operator_map.insert("+", TokenType::Plus);
        set.operator_map.insert("-", TokenType::Minus);
        set.operator_map.insert("*", TokenType::Multiply);
        set.operator_map.insert("/", TokenType::Divide);
        set.operator_map.insert("%", TokenType::Modulus);
        set.operator_map.insert("!", TokenType::Not);
        set.operator_map.insert("=", TokenType::Assignment);
        set.operator_map.insert("==", TokenType::Equal);
        set.operator_map.insert("!=", TokenType::Inequal);
        set.operator_map.insert(">", TokenType::GreaterThan);
        set.operator_map.insert("<", TokenType::LessThan);
        set.operator_map.insert("<=", TokenType::GreaterThanEqual);
        set.operator_map.insert("<=", TokenType::LessThanEqual);
        set.operator_map.insert(".", TokenType::Dot);
        set.operator_map.insert(",", TokenType::Comma);
        set.operator_map.insert(";", TokenType::SemiColon);
        set.operator_map.insert("(", TokenType::LeftParenthese);
        set.operator_map.insert(")", TokenType::RightParenthese);
        set.operator_map.insert("{", TokenType::LeftCurlyBracket);
        set.operator_map.insert("}", TokenType::RightCurlyBracket);
        for (sign, _) in &set.operator_map {
            for ch in sign.chars() {
                set.signs_chars_set.insert(ch);
            }
        }
        // keyword
        set.keyword_map.insert("if", TokenType::If);
        set.keyword_map.insert("else", TokenType::Else);
        set.keyword_map.insert("while", TokenType::While);
        set.keyword_map.insert("break", TokenType::Break);
        set.keyword_map.insert("continue", TokenType::Continue);
        set.keyword_map.insert("fn", TokenType::Function);
        set.keyword_map.insert("return", TokenType::Return);
        set.keyword_map.insert("true", TokenType::True);
        set.keyword_map.insert("false", TokenType::False);
        set.keyword_map.insert("null", TokenType::None);
        set
    }

    pub fn get_sign_type(&self, sign: &str) -> Option<TokenType> {
        match self.operator_map.get(sign) {
            Some(token) => Some(*token),
            None => None,
        }
    }

    pub fn get_keyword_type(&self, sign: &str) -> Option<TokenType> {
        match self.keyword_map.get(sign) {
            Some(token) => Some(*token),
            None => None,
        }
    }

    pub fn is_keyword(&self, sign: &str) -> bool {
        self.keyword_map.contains_key(sign)
    }

    pub fn is_valid_sign(&self, sign: &str) -> bool {
        self.operator_map.contains_key(sign)
    }

    pub fn is_valid_sign_character(&self, ch: char) -> bool {
        self.signs_chars_set.contains(&ch)
    }
}

pub struct Tokens(pub Vec<Token>);

impl Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in &self.0 {
            write!(f, "{} ", token.value)?;
            match token.r#type {
                TokenType::SemiColon
                | TokenType::LeftCurlyBracket
                | TokenType::RightCurlyBracket => writeln!(f)?,
                _ => (),
            }
        }
        writeln!(f)?;
        for token in &self.0 {
            write!(f, "{:?} ", token.r#type)?;
            match token.r#type {
                TokenType::SemiColon
                | TokenType::LeftCurlyBracket
                | TokenType::RightCurlyBracket => writeln!(f)?,
                _ => (),
            }
        }
        Ok(())
    }
}
