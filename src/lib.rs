pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{ASTNode, ExpressionReducer, GrammarReducer};
use script::grammar::{GrammarSet, Symbol, TerminalSymbolDef};
use script::lexer::Lexer;
use script::lrparser::{LRParser, TransitionAction};
use script::token::{Token, TokenType, Tokens};

pub struct ScriptParser {
    lexer: Lexer,
    lr_parser: LRParser,
    reducer: GrammarReducer,
}

impl ScriptParser {
    pub fn from(
        lr_parser: LRParser,
        reducer: GrammarReducer,
    ) -> Result<ScriptParser, &'static str> {
        if reducer.len() != lr_parser.grammar_set.grammars.len() - 1 {
            return Err(
                "Parser should have the same number of reducers as the number of grammer rules (starter grammar does not count).",
            );
        }
        Ok(ScriptParser {
            lexer: Lexer::new(),
            lr_parser,
            reducer,
        })
    }

    pub fn parse(&mut self, input: &String) -> Result<ASTNode, String> {
        let tokens = self.lexer.parse(input)?;
        #[cfg(debug_assertions)]
        println!("{}", tokens);
        Ok(self.lr_parse(tokens)?)
    }

    pub fn lr_parse(&self, tokens: Tokens) -> Result<ASTNode, String> {
        /* parse stack initial state 0 */
        let mut parse_stack = Vec::from([0]);
        let mut ast_stack = Vec::<ASTNode>::new();
        let mut iter = tokens.0.into_iter();
        let mut token = match iter.next() {
            Some(token) => token,
            None => return Err(format!("Parse Error: Incorrect syntax")),
        };
        while parse_stack.len() != 0 {
            let state = match parse_stack.last() {
                Some(&state) => state,
                None => return Err(format!("Parse Error: stack is empty when peek")),
            };
            let action = self
                .lr_parser
                .get_action(state, &Symbol::Terminal(token.r#type))?;
            #[cfg(debug_assertions)]
            println!("{:?} -> [{}] {}", token.r#type, state, action);
            match action {
                TransitionAction::Shift(state) => {
                    parse_stack.push(*state);
                    /* push AST stack */
                    ast_stack.push(ASTNode::Token(token));
                    token = match iter.next() {
                        Some(token) => token,
                        None => return Err(format!("Parse Error: Incorrect syntax")),
                    };
                }
                TransitionAction::Reduce(rule_number) => {
                    let rule_idx = rule_number - 1;
                    let grammar = match self.lr_parser.grammar_set.grammars.get(rule_idx) {
                        Some(grammar) => grammar,
                        None => {
                            return Err(format!(
                                "Parse Error: Grammar rule {} does not exist",
                                rule_number
                            ))
                        }
                    };
                    /* pop AST stack to form AST params and the push a new AST expression */
                    if grammar.rvals.len() > parse_stack.len() {
                        return Err(format!("Parse Error: stack does not have enough items"));
                    }
                    /* Pop rvals.len() items */
                    let remains = ast_stack.len() - grammar.rvals.len();
                    let params = ast_stack.drain(remains..).collect::<Vec<_>>();
                    let ast_node = self.reducer[rule_idx - 1](params);
                    #[cfg(debug_assertions)]
                    println!("Reduce [{}. {}] -> {}", rule_number, grammar, ast_node);
                    ast_stack.push(ast_node);
                    let remains = parse_stack.len() - grammar.rvals.len();
                    parse_stack.truncate(remains);
                    /* Perform GOTO */
                    let state = match parse_stack.last() {
                        Some(&state) => state,
                        None => return Err(format!("Parse Error: stack is empty when goto")),
                    };
                    let goto_state = match self.lr_parser.get_action(state, &grammar.lval) {
                        Ok(TransitionAction::Goto(state)) => *state,
                        _ => {
                            return Err(format!(
                                "Parse Error: cannot find goto action for: {}",
                                grammar.lval
                            ))
                        }
                    };
                    parse_stack.push(goto_state);
                }
                TransitionAction::Accept => {
                    /* AST stack should have exactly 1 item left, which is the returned expression */
                    if let Some(expr) = ast_stack.pop() {
                        if ast_stack.len() != 0 {
                            return Err(format!(
                                "Parse Error: Accepted but ast stack still has items left",
                            ));
                        }
                        return Ok(expr);
                    } else {
                        return Err(format!("Parse Error: Accepted but ast stack is empty"));
                    }
                }
                TransitionAction::Goto(_) => {
                    return Err(format!("Parse Error: Unexpected goto action"))
                }
            }
        }
        Err(format!("Parse Error: Incorrect syntax"))
    }
}

pub fn multiply_reducer(args: Vec<ASTNode>) -> ASTNode {
    ASTNode::ActionExpression(
        "a * b",
        Box::new(move || -> ASTNode {
            let left = &args[0];
            let right = &args[2];
            ASTNode::Token(Token {
                value: format!("{} * {}", left, right),
                r#type: TokenType::Integer,
            })
        }),
    )
}

pub fn add_reducer(args: Vec<ASTNode>) -> ASTNode {
    ASTNode::ActionExpression(
        "a + b",
        Box::new(move || -> ASTNode {
            let left = &args[0];
            let right = &args[2];
            ASTNode::Token(Token {
                value: format!("{} * {}", left, right),
                r#type: TokenType::Integer,
            })
        }),
    )
}

pub fn value_reducer(mut args: Vec<ASTNode>) -> ASTNode {
    args.pop().unwrap().value()
}

pub fn init_math_script_parser() -> Result<ScriptParser, String> {
    let reducer: Vec<ExpressionReducer> =
        vec![multiply_reducer, add_reducer, value_reducer, value_reducer];
    let grammars = [
        "S -> E EOF",
        "E -> E * B",
        "E -> E + B",
        "E -> B",
        "B -> int",
    ];
    let terminal_symbols = [
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("+", TokenType::Plus),
        TerminalSymbolDef("int", TokenType::Integer),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];
    let grammar_set = GrammarSet::from(&grammars, &terminal_symbols)?;
    let lr_parser = LRParser::lr0(grammar_set)?;
    #[cfg(debug_assertions)]
    println!("{}", lr_parser);
    Ok(ScriptParser::from(lr_parser, reducer)?)
}
