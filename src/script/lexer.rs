use super::token::Token;

#[derive(PartialEq, Eq)]
struct LexerHandler {
    name: &'static str,
    handle: fn(ch: char) -> LexerResult,
}

struct LexerResult {
    state: LexerHandler,
    create: Option<fn(buffer: &String) -> Token>, // Token type
    buffer: bool,
    move_cursor: bool,
    error: Option<String>,
}

struct Lexer {
    state: LexerHandler,
    buffer: String,
    tokens: Vec<Token>,
    error: String,
}

impl Lexer {
    const ERROR_RESULT: LexerResult = LexerResult {
        state: Lexer::ERROR_STATE,
        create: None,
        buffer: false,
        move_cursor: false,
        error: None,
    };

    const NORMAL_STATE: LexerHandler = LexerHandler {
        name: "normal",
        handle: |ch: char| match ch {
            '_' | _ if ch.is_alphabetic() => LexerResult {
                state: Lexer::IDENTIFIER_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ if ch.is_whitespace() => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ if ch != '#' && ch != '"' && ch.is_ascii_punctuation() => LexerResult {
                // TODO: only accept valid script signs
                state: Lexer::SIGN_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ if ch.is_ascii_digit() => LexerResult {
                state: Lexer::INTEGER_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            '"' => LexerResult {
                state: Lexer::STRING_STATE,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            '#' => LexerResult {
                state: Lexer::COMMENT_STATE,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            '\0' => LexerResult {
                state: Lexer::END_STATE,
                create: Some(Token::end),
                buffer: false,
                move_cursor: false,
                error: None,
            },
            _ => LexerResult {
                error: Some(format!("Syntax Error: Unexpected character {}", ch)),
                ..Lexer::ERROR_RESULT
            },
        },
    };

    const IDENTIFIER_STATE: LexerHandler = LexerHandler {
        name: "identifier",
        handle: |ch: char| match ch {
            '_' | _ if ch.is_ascii_alphanumeric() => LexerResult {
                state: Lexer::IDENTIFIER_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: Some(Token::identifier),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        },
    };

    // TODO: only accept valid script signs
    const SIGN_STATE: LexerHandler = LexerHandler {
        name: "sign",
        handle: |ch: char| match ch {
            _ if ch.is_ascii_punctuation() => LexerResult {
                state: Lexer::SIGN_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: Some(Token::sign),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        },
    };

    const INTEGER_STATE: LexerHandler = LexerHandler {
        name: "number",
        handle: |ch: char| match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: Lexer::INTEGER_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            '.' => LexerResult {
                state: Lexer::FLOAT_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: Some(Token::number),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        },
    };

    const FLOAT_STATE: LexerHandler = LexerHandler {
        name: "number",
        handle: |ch: char| match ch {
            _ if ch.is_ascii_digit() => LexerResult {
                state: Lexer::FLOAT_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: Some(Token::number),
                buffer: false,
                move_cursor: false,
                error: None,
            },
        },
    };

    const STRING_STATE: LexerHandler = LexerHandler {
        name: "string",
        handle: |ch: char| match ch {
            '\0' => LexerResult {
                error: Some(format!("Syntax Error: Unexpected EOF")),
                ..Lexer::ERROR_RESULT
            },
            '"' => LexerResult { // TODO implement escape character \"
                state: Lexer::NORMAL_STATE,
                create: Some(Token::string),
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::STRING_STATE,
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
        },
    };

    const COMMENT_STATE: LexerHandler = LexerHandler {
        name: "comment",
        handle: |ch: char| match ch {
            '\0' | '\n' => LexerResult {
                state: Lexer::NORMAL_STATE,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
            _ => LexerResult {
                state: Lexer::COMMENT_STATE,
                create: None,
                buffer: false,
                move_cursor: true,
                error: None,
            },
        },
    };

    const END_STATE: LexerHandler = LexerHandler {
        name: "end",
        handle: |_| Lexer::ERROR_RESULT,
    };

    const ERROR_STATE: LexerHandler = LexerHandler {
        name: "error",
        handle: |_| Lexer::ERROR_RESULT,
    };

    pub fn new() -> Lexer {
        Lexer {
            state: Lexer::NORMAL_STATE,
            buffer: String::new(),
            tokens: Vec::new(),
            error: "No error".to_owned(),
        }
    }
    pub fn parse(&mut self, ch: char) -> bool {
        dbg!(self.state.name, ch);
        let res = (self.state.handle)(ch);
        self.state = res.state;
        match res.error {
            Some(err) => self.error = err,
            None => (),
        }
        match res.create {
            Some(token) => {
                self.tokens.push(token(&self.buffer));
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

pub fn lexer_parse(input: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new();
    let mut iter = input.chars();
    let mut move_cursor = false;
    let mut next_char: char = match iter.next() {
        Some(ch) => ch,
        None => return Err("Empty input".to_owned()),
    };
    while lexer.state != Lexer::END_STATE && lexer.state != Lexer::ERROR_STATE {
        if move_cursor {
            next_char = match iter.next() {
                Some(ch) => ch,
                None => '\0',
            };
        }
        move_cursor = lexer.parse(next_char);
    }
    match lexer.state {
        Lexer::END_STATE => Ok(lexer.tokens),
        _ => Err(lexer.error),
    }
}
