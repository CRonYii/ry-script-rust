use super::ast::{ASTNode, ExpressionReducer, RuntimeValue};
use super::error::{ParseError, RuntimeError, SyntaxError};
use super::grammar::{GrammarSet, Symbol, TerminalSymbolDef};
use super::lexer::Lexer;
use super::lrparser::{LRParser, TransitionAction};
use super::token::{LexerTokenMap, ParserToken, SpecialTokenMap, Tokens};

pub struct ScriptRunner<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> {
    lexer: Lexer<T>,
    lr_parser: LRParser<T>,
    reducer: Vec<ExpressionReducer<ENV, T, R, E>>,
}

pub struct GrammarRule<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError>(
    pub &'static str,
    pub ExpressionReducer<ENV, T, R, E>,
);

impl<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> ScriptRunner<ENV, T, R, E> {
    pub fn new(
        grammars: Vec<GrammarRule<ENV, T, R, E>>,
        token_map: LexerTokenMap<T>,
        operator: &[TerminalSymbolDef<T>],
        keyword: &[TerminalSymbolDef<T>],
    ) -> super::error::Result<ScriptRunner<ENV, T, R, E>, E> {
        let mut terminal_symbols = vec![
            TerminalSymbolDef("id", token_map.identifier),
            TerminalSymbolDef("str", token_map.string),
            TerminalSymbolDef("int", token_map.integer),
            TerminalSymbolDef("float", token_map.float),
            TerminalSymbolDef("EOF", token_map.eof),
        ];
        for &symbol in operator {
            terminal_symbols.push(symbol);
        }
        for &symbol in keyword {
            terminal_symbols.push(symbol);
        }
        let grammar_set = GrammarSet::new(&grammars, &terminal_symbols, token_map.eof)?;
        let lr_parser = LRParser::lr0(grammar_set)?;
        #[cfg(feature = "debug_lrparser")]
        println!("{}", lr_parser);
        let special_token_map = SpecialTokenMap::new(operator, keyword);
        Ok(ScriptRunner {
            lexer: Lexer::new(token_map, special_token_map),
            lr_parser,
            reducer: grammars.into_iter().map(|g| g.1).collect(),
        })
    }

    pub fn run(&mut self, env: &mut ENV, input: &str) -> super::error::Result<R, E> {
        let tokens = self.lexer.parse(input)?;
        #[cfg(feature = "debug_lexer")]
        println!("{}", tokens);
        let execution_result = match self.lr_parse(tokens)?.evaluate(env)? {
            ASTNode::Value(value) => value,
            _ => return Err(ParseError::IncorrectParseResult.into()),
        };
        Ok(execution_result)
    }

    fn lr_parse(&self, tokens: Tokens<T>) -> super::error::Result<ASTNode<ENV, T, R, E>, E> {
        /* parse stack initial state 0 */
        let mut parse_stack = Vec::from([0]);
        let mut ast_stack = Vec::<ASTNode<ENV, T, R, E>>::new();
        let mut iter = tokens.0.into_iter();
        let mut token = match iter.next() {
            Some(token) => token,
            None => return Err(SyntaxError::SyntaxError.into()),
        };
        while parse_stack.len() != 0 {
            let state = match parse_stack.last() {
                Some(&state) => state,
                None => return Err(ParseError::Error("stack is empty when peek").into()),
            };
            let action = self
                .lr_parser
                .get_action(state, &Symbol::Terminal(token.r#type))?;
            #[cfg(feature = "debug_lrparser")]
            println!(
                "{:?} -> [{}] {}\n  AST stack{:?}",
                token.r#type, state, action, ast_stack
            );
            match action {
                TransitionAction::Shift(state) => {
                    parse_stack.push(*state);
                    /* push AST stack */
                    #[cfg(feature = "debug_lrparser")]
                    println!("  Shift [{:?}] -> {}", token, state);
                    ast_stack.push(ASTNode::Token(token));
                    token = match iter.next() {
                        Some(token) => token,
                        None => return Err(SyntaxError::SyntaxError.into()),
                    };
                }
                TransitionAction::Reduce(rule_number) => {
                    let rule_idx = rule_number - 1;
                    let grammar = match self.lr_parser.grammar_set.grammars.get(rule_idx) {
                        Some(grammar) => grammar,
                        None => return Err(ParseError::GrammarDoesNotExist(*rule_number).into()),
                    };
                    /* pop AST stack to form AST params and the push a new AST expression */
                    if grammar.rvals.len() > parse_stack.len() {
                        return Err(ParseError::Error("stack does not have enough items").into());
                    }
                    /* Pop rvals.len() items */
                    let remains = ast_stack.len() - grammar.rvals.len();
                    let params = ast_stack.drain(remains..).rev().collect();
                    let args = ReducerArg::new(params);
                    let ast_node = self.reducer[rule_idx](args);
                    #[cfg(feature = "debug_lrparser")]
                    println!("  Reduce [{}. {}] -> {}", rule_number, grammar, ast_node);
                    ast_stack.push(ast_node);
                    let remains = parse_stack.len() - grammar.rvals.len();
                    parse_stack.truncate(remains);
                    /* Perform GOTO */
                    let state = match parse_stack.last() {
                        Some(&state) => state,
                        None => return Err(ParseError::Error("stack is empty when goto").into()),
                    };
                    let goto_state = match self.lr_parser.get_action(state, &grammar.lval) {
                        Ok(TransitionAction::Goto(state)) => *state,
                        _ => return Err(ParseError::Error("goto action does not exist").into()),
                    };
                    parse_stack.push(goto_state);
                }
                TransitionAction::Accept => {
                    /* AST stack should have exactly 1 item left, which is the returned expression */
                    if let Some(expr) = ast_stack.pop() {
                        if ast_stack.len() != 0 {
                            return Err(ParseError::Error(
                                "accepted but ast stack still has items left",
                            )
                            .into());
                        }
                        return Ok(expr);
                    } else {
                        return Err(ParseError::Error("accepted but ast stack is empty").into());
                    }
                }
                TransitionAction::Goto(_) => {
                    return Err(ParseError::Error("Unexpected goto action").into())
                }
            }
        }
        Err(SyntaxError::SyntaxError.into())
    }
}

pub struct ReducerArg<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> {
    args: Vec<ASTNode<ENV, T, R, E>>,
}

impl<ENV, T: ParserToken<T>, R: RuntimeValue<T>, E: RuntimeError> ReducerArg<ENV, T, R, E> {
    fn new(args: Vec<ASTNode<ENV, T, R, E>>) -> Self {
        Self { args }
    }

    pub fn eval(&mut self, env: &mut ENV) -> Result<ASTNode<ENV, T, R, E>, E> {
        self.args.pop().unwrap().evaluate(env)
    }

    pub fn nth_eval(&mut self, env: &mut ENV, n: usize) -> Result<ASTNode<ENV, T, R, E>, E> {
        self.nth_node(n).evaluate(env)
    }

    pub fn eval_skip(&mut self, env: &mut ENV, n: usize) -> Result<ASTNode<ENV, T, R, E>, E> {
        let node = self.eval(env);
        self.skip_n(n);
        node
    }

    pub fn val(&mut self) -> ASTNode<ENV, T, R, E> {
        self.args.pop().unwrap()
    }

    pub fn nth_val(&mut self, n: usize) -> ASTNode<ENV, T, R, E> {
        self.nth_node(n)
    }

    pub fn val_skip(&mut self, n: usize) -> ASTNode<ENV, T, R, E> {
        let node = self.val();
        self.skip_n(n);
        node
    }

    fn nth_node(&mut self, n: usize) -> ASTNode<ENV, T, R, E> {
        for _ in 0..n {
            self.skip()
        }
        self.args.pop().unwrap()
    }

    pub fn skip(&mut self) {
        self.args.pop();
    }

    pub fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.skip()
        }
    }
}
