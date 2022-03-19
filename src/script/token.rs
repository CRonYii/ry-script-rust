use std::collections::HashSet;

#[derive(Debug)]
pub enum TokenType {
    Identifier,
    Sign,
    Number,
    String,
    End,
}

#[derive(Debug)]
pub struct Token {
    r#type: TokenType,
    value: String,
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