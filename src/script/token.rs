#[derive(Debug)]
enum TokenType {
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

impl Token {
    pub fn identifier(value: &String) -> Token {
        Token {
            r#type: TokenType::Identifier,
            value: value.clone()
        }
    }
    pub fn sign(value: &String) -> Token {
        Token {
            r#type: TokenType::Sign,
            value: value.clone()
        }
    }
    pub fn number(value: &String) -> Token {
        Token {
            r#type: TokenType::Number,
            value: value.clone()
        }
    }
    pub fn string(value: &String) -> Token {
        Token {
            r#type: TokenType::String,
            value: value.clone()
        }
    }
    pub fn end(_: &String) -> Token {
        Token {
            r#type: TokenType::End,
            value: "EOF".to_owned()
        }
    }
}