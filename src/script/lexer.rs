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
            _ if ch.is_ascii_digit() => LexerResult {
                state: Lexer::NORMAL_STATE, // NUMBER_STATE
                create: None,
                buffer: true,
                move_cursor: true,
                error: None,
            },
            '\0' => LexerResult {
                state: Lexer::END_STATE,
                create: None,
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
            Some(token) =>  {
                self.tokens.push(token(&self.buffer)); // TODO: maybe can use &str with lifetime 
                self.buffer.clear();
            },
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
