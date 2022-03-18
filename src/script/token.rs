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