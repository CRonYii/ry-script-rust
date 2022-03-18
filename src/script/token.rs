#[derive(Debug)]
enum TokenType {
    Identifier
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
}