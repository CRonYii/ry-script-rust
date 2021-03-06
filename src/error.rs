pub type Result<T, E> = std::result::Result<T, ScriptError<E>>;

pub enum ScriptError<E> {
    Runtime(E),
    Grammar(GrammarError),
    Lexer(LexerError),
    Parse(ParseError),
    Syntax(SyntaxError),
}

impl<E> From<E> for ScriptError<E> {
    fn from(error: E) -> Self {
        ScriptError::Runtime(error)
    }
}

impl<E: RuntimeError> std::fmt::Display for ScriptError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Grammar(error) => write!(f, "GrammarError: {}", error),
            ScriptError::Lexer(error) => write!(f, "LexerError: {}", error),
            ScriptError::Parse(error) => write!(f, "ParseError: {}", error),
            ScriptError::Syntax(error) => write!(f, "{:?}", error),
            ScriptError::Runtime(error) => write!(f, "RuntimeError: {}", error),
        }
    }
}

impl<E: RuntimeError> std::fmt::Debug for ScriptError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Grammar(error) => write!(f, "GrammarError: {}", error),
            ScriptError::Lexer(error) => write!(f, "LexerError: {}", error),
            ScriptError::Parse(error) => write!(f, "ParseError: {}", error),
            ScriptError::Syntax(error) => write!(f, "{:?}", error),
            ScriptError::Runtime(error) => write!(f, "RuntimeError: {}", error),
        }
    }
}

pub enum GrammarError {
    Error(&'static str),
    InvalidGrammarText(&'static str),
    InvalidSymbol(&'static str),
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::Error(msg) => write!(f, "{}", msg),
            GrammarError::InvalidGrammarText(grammar) => write!(f, "Invalid grammar {}", grammar),
            GrammarError::InvalidSymbol(symbol) => write!(f, "Invalid symbol {}", symbol),
        }
    }
}

pub enum LexerError {
    Error(&'static str),
    UnexpectedToken(char),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::Error(msg) => write!(f, "{}", msg),
            LexerError::UnexpectedToken(ch) => write!(f, "Unexpected token {}", ch),
        }
    }
}

impl<E> From<LexerError> for ScriptError<E>
where
    E: RuntimeError,
{
    fn from(error: LexerError) -> Self {
        ScriptError::Lexer(error)
    }
}

pub enum ParseError {
    Error(&'static str),
    IncorrectParseResult,
    UnexpectedSymbol(String),
    GrammarDoesNotExist(usize),
    StateDoesNotExist(usize),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Error(msg) => write!(f, "{}", msg),
            ParseError::UnexpectedSymbol(symbol) => write!(f, "Unexpected symbol {}", symbol),
            ParseError::GrammarDoesNotExist(rule) => write!(f, "Grammar rule {} does not exist", rule),
            ParseError::StateDoesNotExist(state) => write!(f, "state {} does not exist", state),
            ParseError::IncorrectParseResult => write!(f, "AST evaluation final result is not a value"),
        }
    }
}

#[derive(Debug)]
pub enum SyntaxError {
    SyntaxError
}

impl<E> From<GrammarError> for ScriptError<E>
where
    E: RuntimeError,
{
    fn from(error: GrammarError) -> Self {
        ScriptError::Grammar(error)
    }
}

impl<E> From<ParseError> for ScriptError<E>
where
    E: RuntimeError,
{
    fn from(error: ParseError) -> Self {
        ScriptError::Parse(error)
    }
}

impl<E> From<SyntaxError> for ScriptError<E>
where
    E: RuntimeError,
{
    fn from(error: SyntaxError) -> Self {
        ScriptError::Syntax(error)
    }
}

pub trait RuntimeError: std::fmt::Display {}
