use super::token::{Token, TokenMap, TokenType, Tokens};

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
    error: Option<String>,
}

pub struct Lexer {
    state: LexerState,
    buffer: String,
    tokens: Vec<Token>,
    signs: TokenMap,
    error: String,
}

impl Lexer {
    const ERROR_RESULT: LexerResult = LexerResult {
        state: LexerState::Error,
        create: None,
        buffer: false,
        move_cursor: false,
        error: None,
    };

    fn handle_normal_state(&self, ch: char) -> LexerResult {
        match ch {
            '_' | _ if ch.is_alphabetic() => LexerResult {
                state: LexerState::Identifier,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ if ch.is_whitespace() => LexerResult {
                state: LexerState::Normal,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ if self.signs.is_valid_sign_character(ch) => LexerResult {
                state: LexerState::Sign,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Integer,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            '"' => LexerResult {
                state: LexerState::String,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            '#' => LexerResult {
                state: LexerState::Comment,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            '\0' => LexerResult {
                state: LexerState::End,
                create: Some(TokenType::EOF),
                buffer: false,
                move_cursor: false,
                error: None,
            },
            _ => LexerResult {
                error: Some(format!("Syntax error: Unexpected token {}", ch)),
                ..Lexer::ERROR_RESULT
            },
        }
    }

    fn handle_identifier_state(&self, ch: char) -> LexerResult {
        match ch {
            '_' | _ if ch.is_ascii_alphanumeric() => LexerResult {
                state: LexerState::Identifier,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ if self.signs.is_keyword(&self.buffer) => LexerResult {
                state: LexerState::Normal,
                create: self.signs.get_keyword_type(&self.buffer),
                buffer: false,
                move_cursor: false,
                error: None,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Identifier),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        }
    }

    fn handle_sign_state(&self, ch: char) -> LexerResult {
        let mut tmp = self.buffer.clone();
        tmp.push(ch);
        if self.signs.is_valid_sign(&tmp) {
            LexerResult {
                state: LexerState::Sign,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            }
        } else if let Some(token) = self.signs.get_sign_type(&self.buffer) {
            LexerResult {
                state: LexerState::Normal,
                create: Some(token),
                buffer: false,
                move_cursor: false,
                error: None,
            }
        } else {
            LexerResult {
                error: Some(format!("Syntax error: Unexpected token {}", self.buffer)),
                ..Lexer::ERROR_RESULT
            }
        }
    }

    fn handle_integer_state(&self, ch: char) -> LexerResult {
        match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Integer,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            '.' => LexerResult {
                state: LexerState::Float,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Integer),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        }
    }

    fn handle_float_state(&self, ch: char) -> LexerResult {
        match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: LexerState::Float,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: LexerState::Normal,
                create: Some(TokenType::Float),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        }
    }

    fn handle_string_state(&self, ch: char) -> LexerResult {
        match ch {
            '\0' => LexerResult {
                error: Some(format!("Syntax error: Unexpected EOF")),
                ..Lexer::ERROR_RESULT
            },
            '"' => LexerResult {
                // TODO implement escape character \"
                state: LexerState::Normal,
                create: Some(TokenType::String),
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: LexerState::String,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
        }
    }

    fn handle_comment_state(&self, ch: char) -> LexerResult {
        match ch {
            '\0' | '\n' => LexerResult {
                state: LexerState::Normal,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: LexerState::Comment,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
        }
    }

    pub fn new() -> Lexer {
        Lexer {
            state: LexerState::Normal,
            buffer: String::new(),
            tokens: Vec::new(),
            signs: TokenMap::init(),
            error: "No error".to_owned(),
        }
    }

    fn reset(&mut self) {
        self.state = LexerState::Normal;
        self.buffer.clear();
        self.tokens.clear();
    }

    pub fn parse(&mut self, input: &str) -> Result<Tokens, String> {
        self.reset();
        let mut iter = input.chars();
        let mut move_cursor = false;
        let mut next_char: char = match iter.next() {
            Some(ch) => ch,
            None => return Err("Lexer error: Empty input".to_owned()),
        };
        while self.state != LexerState::End && self.state != LexerState::Error {
            if move_cursor {
                next_char = match iter.next() {
                    Some(ch) => ch,
                    None => '\0',
                };
            }
            move_cursor = self.parse_char(next_char);
        }
        match self.state {
            LexerState::End => Ok(Tokens(std::mem::take(&mut self.tokens))),
            _ => Err(std::mem::take(&mut self.error)),
        }
    }

    fn parse_char(&mut self, ch: char) -> bool {
        #[cfg(feature = "debug_lexer")]
        println!("{:?} -> {:?}", self.state, ch);
        let res = match self.state {
            LexerState::Normal => self.handle_normal_state(ch),
            LexerState::Identifier => self.handle_identifier_state(ch),
            LexerState::Sign => self.handle_sign_state(ch),
            LexerState::Integer => self.handle_integer_state(ch),
            LexerState::Float => self.handle_float_state(ch),
            LexerState::String => self.handle_string_state(ch),
            LexerState::Comment => self.handle_comment_state(ch),
            LexerState::Error | LexerState::End => Lexer::ERROR_RESULT,
        };
        self.state = res.state;
        match res.error {
            Some(err) => self.error = err,
            None => (),
        }
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
        res.move_cursor
    }
}
