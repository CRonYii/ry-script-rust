use std::{collections::HashSet, fmt::Display};
use std::vec::Vec;

#[derive(Debug)]
pub enum TokenType {
    Identifier,
    Sign,
    Number,
    String,
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
    signs_set: HashSet<&'static str>,
    signs_chars_set: HashSet<char>,
}

impl TokenSign {
    pub fn init() ->TokenSign {
        let mut set = TokenSign {
            signs_set: HashSet::from(["+","-","*","/","%","!","=","==","!=",">","<",">=","<=",";",".","(",")"]),
            signs_chars_set: HashSet::new()
        };
        for sign in &set.signs_set {
            for ch in sign.chars() {
                set.signs_chars_set.insert(ch);
            }
        }
        set
    }

    pub fn is_valid_sign(&self, sign: &str) -> bool {
        self.signs_set.contains(sign)
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