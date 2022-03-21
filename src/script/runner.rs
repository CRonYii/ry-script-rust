use crate::script::grammar::GrammarSet;

use super::ast::{ASTNode, ExpressionReducer};
use super::grammar::{Symbol, TerminalSymbolDef};
use super::lexer::Lexer;
use super::lrparser::{LRParser, TransitionAction};
use super::token::Tokens;

pub struct ScriptRunner {
    lexer: Lexer,
    lr_parser: LRParser,
    reducer: Vec<ExpressionReducer>,
}

pub struct GrammarRule(pub &'static str, pub ExpressionReducer);

impl ScriptRunner {
    pub fn from(
        grammars: Vec<GrammarRule>,
        terminal_symbols: &[TerminalSymbolDef],
    ) -> Result<ScriptRunner, String> {
        let grammar_set = GrammarSet::from(&grammars, &terminal_symbols)?;
        let lr_parser = LRParser::lr0(grammar_set)?;
        #[cfg(feature = "debug_lrparser")]
        println!("{}", lr_parser);
        Ok(ScriptRunner {
            lexer: Lexer::new(),
            lr_parser,
            reducer: grammars.into_iter().map(|g| g.1).collect(),
        })
    }

    pub fn run(&mut self, input: &String) -> Result<ASTNode, String> {
        let tokens = self.lexer.parse(input)?;
        #[cfg(feature = "debug_lexer")]
        println!("{}", tokens);
        let execution_result = self.lr_parse(tokens)?.evaluate()?;
        Ok(execution_result)
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
            #[cfg(feature = "debug_lrparser")]
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
                    let params = ast_stack.drain(remains..).rev().collect();
                    let args = ReducerArg::from(params);
                    let ast_node = self.reducer[rule_idx](args);
                    #[cfg(feature = "debug_lrparser")]
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

pub struct ReducerArg {
    args: Vec<ASTNode>,
}

impl ReducerArg {
    fn from(args: Vec<ASTNode>) -> Self {
        Self { args }
    }

    pub fn eval(&mut self) -> Result<ASTNode, String> {
        self.args.pop().unwrap().evaluate()
    }

    pub fn skip(&mut self) {
        self.args.pop();
    }
}
