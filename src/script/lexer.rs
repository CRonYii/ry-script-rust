use super::{token::{Token, TokenMap, TokenType, Tokens}, error::LexerError};

#[derive(Debug, PartialEq, Eq)]
enum LexerState {
    Normal,
    Identifier,
    Sign,
    Integer,
    Float,
    String,
    Comment,
    Error,
    End,
}

struct LexerResult {
    state: LexerState,
    create: Option<TokenType>, // Token type
    buffer: bool,
    move_cursor: bool,
}

pub struct Lexer {
    state: LexerState,
    buffer: String,
    tokens: Vec<Token>,
    signs: TokenMap,
}

impl Lexer {
    const ERROR_RESULT: LexerResult = LexerResult {
        state: LexerState::Error,
        create: None,
        buffer: false,
        move_cursor: false,
    };

    fn handle_normal_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result = match ch {
            '_' | _ if ch.is_alphabetic() => LexerResult {
                state: LexerState::Identifier,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            _ if ch.is_whitespace() => LexerResult {
                state: LexerState::Normal,
                create: None,
                buffer: false,
                move_cursor: true,
            },
            _ if self.signs.is_valid_sign_character(ch) => LexerResult {
                state: LexerState::Sign,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Integer,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            '"' => LexerResult {
                state: LexerState::String,
                create: None,
                buffer: false,
                move_cursor: true,
            },
            '#' => LexerResult {
                state: LexerState::Comment,
                create: None,
                buffer: false,
                move_cursor: true,
            },
            '\0' => LexerResult {
                state: LexerState::End,
                create: Some(TokenType::EOF),
                buffer: false,
                move_cursor: false,
            },
            _ => return Err(LexerError::UnexpectedToken(ch)),
        };
        Ok(result)
    }

    fn handle_identifier_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result = match ch {
            '_' | _ if ch.is_ascii_alphanumeric() => LexerResult {
                state: LexerState::Identifier,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            _ if self.signs.is_keyword(&self.buffer) => LexerResult {
                state: LexerState::Normal,
                create: self.signs.get_keyword_type(&self.buffer),
                buffer: false,
                move_cursor: false,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Identifier),
                buffer: false,
                move_cursor: false,
            },
        };
        Ok(result)
    }

    fn handle_sign_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let mut tmp = self.buffer.clone();
        tmp.push(ch);
        let result = if self.signs.is_valid_sign(&tmp) {
            LexerResult {
                state: LexerState::Sign,
                create: None,
                buffer: true,
                move_cursor: true,
            }
        } else if let Some(token) = self.signs.get_sign_type(&self.buffer) {
            LexerResult {
                state: LexerState::Normal,
                create: Some(token),
                buffer: false,
                move_cursor: false,
            }
        } else {
            return Err(LexerError::UnexpectedToken(ch));
        };
        Ok(result)
    }

    fn handle_integer_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result = match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Integer,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            '.' => LexerResult {
                state: LexerState::Float,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Integer),
                buffer: false,
                move_cursor: false,
            },
        };
        Ok(result)
    }

    fn handle_float_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result = match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Float,
                create: None,
                buffer: true,
                move_cursor: true,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Float),
                buffer: false,
                move_cursor: false,
            },
        };
        Ok(result)
    }

    fn handle_string_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result= match ch {
            '\0' => return Err(LexerError::Error("Unexpected EOF")),
            '"' => LexerResult {
                // TODO implement escape character \"
                state: LexerState::Normal,
                create: Some(TokenType::String),
                buffer: false,
                move_cursor: true,
            },
            _ => LexerResult {
                state: LexerState::String,
                create: None,
                buffer: true,
                move_cursor: true,
            },
        };
        Ok(result)
    }

    fn handle_comment_state(&self, ch: char) -> Result<LexerResult, LexerError> {
        let result = match ch {
            '\0' | '\n' => LexerResult {
                state: LexerState::Normal,
                create: None,
                buffer: false,
                move_cursor: true,
            },
            _ => LexerResult {
                state: LexerState::Comment,
                create: None,
                buffer: false,
                move_cursor: true,
            },
        };
        Ok(result)
    }

    pub fn new() -> Lexer {
        Lexer {
            state: LexerState::Normal,
            buffer: String::new(),
            tokens: Vec::new(),
            signs: TokenMap::init(),
        }
    }

    fn reset(&mut self) {
        self.state = LexerState::Normal;
        self.buffer.clear();
        self.tokens.clear();
    }

    pub fn parse(&mut self, input: &str) -> Result<Tokens, LexerError> {
        self.reset();
        let mut iter = input.chars();
        let mut move_cursor = false;
        let mut next_char: char = match iter.next() {
            Some(ch) => ch,
            None => return Err(LexerError::Error("Empty input string")),
        };
        while self.state != LexerState::End && self.state != LexerState::Error {
            if move_cursor {
                next_char = match iter.next() {
                    Some(ch) => ch,
                    None => '\0',
                };
            }
            move_cursor = self.parse_char(next_char)?;
        }
        match self.state {
            LexerState::End => Ok(Tokens(std::mem::take(&mut self.tokens))),
            _ => Err(LexerError::Error("Lexer is not at END State when parsing finished")),
        }
    }

    fn parse_char(&mut self, ch: char) -> Result<bool, LexerError> {
        #[cfg(feature = "debug_lexer")]
        println!("{:?} -> {:?}", self.state, ch);
        let res = match self.state {
            LexerState::Normal => self.handle_normal_state(ch)?,
            LexerState::Identifier => self.handle_identifier_state(ch)?,
            LexerState::Sign => self.handle_sign_state(ch)?,
            LexerState::Integer => self.handle_integer_state(ch)?,
            LexerState::Float => self.handle_float_state(ch)?,
            LexerState::String => self.handle_string_state(ch)?,
            LexerState::Comment => self.handle_comment_state(ch)?,
            LexerState::Error | LexerState::End => Lexer::ERROR_RESULT,
        };
        self.state = res.state;
        match res.create {
            Some(token) => {
                self.tokens.push(token.entity(std::mem::take(&mut self.buffer)));
                self.buffer.clear();
            }
            None => (),
        }
        if res.buffer == true {
            self.buffer.push(ch);
        }
        Ok(res.move_cursor)
    }
}
