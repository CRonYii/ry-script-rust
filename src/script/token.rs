use std::collections::{HashSet, HashMap};
use std::fmt::Display;
use std::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Identifier,
    Sign,
    Number,
    String,
    PlusSign,
    MinusSign,
    MultiplySign,
    DivideSign,
    ModulusSign,
    NotSign,
    AssignmentSign,
    EqualSign,
    InequalSign,
    GreaterThanSign,
    LessThanSign,
    GreaterThanEqualSign,
    LessThanEqualSign,
    Dot,
    Comma,
    SemiColon,
    LeftParenthese,
    RightParenthese,
    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub r#type: TokenType,
    pub value: String,
}

impl TokenType {
    pub fn entity(self, value: &String) -> Token {
        Token {
            r#type: self,
            value: value.clone()
        }
    }
}

pub struct TokenSign {
    signs_map: HashMap<&'static str, TokenType>,
    signs_chars_set: HashSet<char>,
}

impl TokenSign {
    pub fn init() ->TokenSign {
        let mut set = TokenSign {
            signs_map: HashMap::new(),
            signs_chars_set: HashSet::new()
        };
        set.signs_map.insert("+", TokenType::PlusSign);
        set.signs_map.insert("-", TokenType::MinusSign);
        set.signs_map.insert("*", TokenType::MultiplySign);
        set.signs_map.insert("/", TokenType::DivideSign);
        set.signs_map.insert("%", TokenType::ModulusSign);
        set.signs_map.insert("!", TokenType::NotSign);
        set.signs_map.insert("=", TokenType::AssignmentSign);
        set.signs_map.insert("==", TokenType::EqualSign);
        set.signs_map.insert("!=", TokenType::InequalSign);
        set.signs_map.insert(">", TokenType::GreaterThanSign);
        set.signs_map.insert("<", TokenType::LessThanSign);
        set.signs_map.insert("<=", TokenType::GreaterThanEqualSign);
        set.signs_map.insert("<=", TokenType::LessThanEqualSign);
        set.signs_map.insert(".", TokenType::Dot);
        set.signs_map.insert(",", TokenType::Comma);
        set.signs_map.insert(";", TokenType::SemiColon);
        set.signs_map.insert("(", TokenType::LeftParenthese);
        set.signs_map.insert(")", TokenType::RightParenthese);
        for (sign, _) in &set.signs_map {
            for ch in sign.chars() {
                set.signs_chars_set.insert(ch);
            }
        }
        set
    }

    pub fn get_sign_type(&self, sign: &str) -> Option<TokenType> {
        match self.signs_map.get(sign) {
            Some(token) => Some(*token),
            None => None
        }
    }

    pub fn is_valid_sign(&self, sign: &str) -> bool {
        self.signs_map.contains_key(sign)
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
            if token.value == ";" {
                writeln!(f)?
            }
        }
        writeln!(f)?;
        for token in &self.0 {
            write!(f, "{:?} ", token.r#type)?;
            if token.value == ";" {
                writeln!(f)?
            }
        }
        Ok(())
    }
}