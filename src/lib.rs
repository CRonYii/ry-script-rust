pub mod math;
pub mod script;

#[cfg(test)]
mod math_tests;

pub use math::matrix::Matrix;

use script::ast::{ASTNode, GrammarReducer};
use script::grammar::{GrammarSet, Symbol, TerminalSymbolDef};
use script::lexer::Lexer;
use script::lrparser::{LRParser, TransitionAction};
use script::token::{TokenType, Tokens};

pub struct ScriptParser {
    lexer: Lexer,
    lr_parser: LRParser,
    reducer: GrammarReducer,
}

impl ScriptParser {
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
                    ast_stack.push(ASTNode::ValueNode(token));
                    token = match iter.next() {
                        Some(token) => token,
                        None => return Err(format!("Parse Error: Incorrect syntax")),
                    };
                }
                TransitionAction::Reduce(rule_number) => {
                    let grammar = match self.lr_parser.grammar_set.grammars.get(*rule_number - 1) {
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
                    let ast_node = ASTNode::ExpressionNode(ast_stack.drain(remains..).collect());
                    #[cfg(debug_assertions)]
                    println!("Reduce expr -> {}", ast_node);
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
                                "Parse Error: Accepted but ast stack still has items left\n{:?}",
                                ast_stack
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

pub fn init_math_script_parser() -> Result<ScriptParser, String> {
    let reducer = vec![];
    let grammars = [
        "S -> E EOF",
        "E -> E * B",
        "E -> E + B",
        "E -> B",
        "B -> id",
        "B -> num",
    ];
    let terminal_symbols = [
        TerminalSymbolDef("*", TokenType::Multiply),
        TerminalSymbolDef("+", TokenType::Plus),
        TerminalSymbolDef("id", TokenType::Identifier),
        TerminalSymbolDef("num", TokenType::Number),
        TerminalSymbolDef("EOF", TokenType::EOF),
    ];
    let grammar_set = GrammarSet::from(&grammars, &terminal_symbols)?;
    let transition_table = LRParser::lr0(grammar_set)?;
    // #[cfg(debug_assertions)]
    // println!("{}", transition_table);
    Ok(ScriptParser {
        lexer: Lexer::new(),
        lr_parser: transition_table,
        reducer,
    })
}
