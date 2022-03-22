use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use super::error::{GrammarError, RuntimeError};
use super::token::ParserToken;
use super::{ast::RuntimeValue, runner::GrammarRule};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Symbol<T: ParserToken<T>> {
    NonTerminal(&'static str),
    Terminal(T),
}

impl<T: ParserToken<T>> Display for Symbol<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::NonTerminal(s) => write!(f, "{}", s)?,
            Symbol::Terminal(t) => write!(f, "{:?}", t)?,
        }
        Ok(())
    }
}

#[derive(Debug, Hash, Eq)]
pub struct Grammar<T: ParserToken<T>> {
    pub rule_number: usize,
    pub lval: Rc<Symbol<T>>,
    pub rvals: Vec<Rc<Symbol<T>>>,
}

impl<T: ParserToken<T>> PartialEq for Grammar<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rule_number == other.rule_number
    }
}

impl<T: ParserToken<T>> Display for Grammar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> ", self.lval)?;
        for rval in &self.rvals {
            write!(f, "{} ", rval)?
        }
        Ok(())
    }
}

type SymbolMap<T> = HashMap<&'static str, Rc<Symbol<T>>>;

#[derive(Clone, Copy)]
pub struct TerminalSymbolDef<T: ParserToken<T>>(pub &'static str, pub T);

pub struct GrammarSet<T: ParserToken<T>> {
    pub grammars: Vec<Rc<Grammar<T>>>,
    pub eof: T,
    pub terminal_symbols: SymbolMap<T>,
    pub non_terminal_symbols: SymbolMap<T>,
}

impl<T: ParserToken<T>> GrammarSet<T> {
    /* Pre-condition: The first grammar is expected to be the starter grammar */
    pub fn new<ENV, R: RuntimeValue<T>, E: RuntimeError>(
        grammars: &Vec<GrammarRule<ENV, T, R, E>>,
        terminals: &Vec<TerminalSymbolDef<T>>,
        eof: T,
    ) -> Result<GrammarSet<T>, GrammarError> {
        // terminal symbols
        let mut terminal_symbols = HashMap::new();
        terminals.iter().for_each(|def| {
            terminal_symbols.insert(def.0, Rc::new(Symbol::Terminal(def.1)));
            ()
        });
        // non-terminal symbols
        let mut non_terminal_symbols = HashMap::new();
        for text in grammars {
            let mut tokens = text.0.split(" ");
            let lval = match tokens.next() {
                Some(token) => token,
                None => return Err(GrammarError::InvalidGrammarText(text.0)),
            };
            non_terminal_symbols.insert(lval, Rc::new(Symbol::NonTerminal(lval)));
        }
        let mut grammar = GrammarSet {
            grammars: vec![],
            terminal_symbols,
            non_terminal_symbols,
            eof,
        };
        for text in grammars {
            grammar.parse_grammar(text.0)?;
        }
        Ok(grammar)
    }

    fn get_symbol(&self, symbol: &'static str) -> Option<Rc<Symbol<T>>> {
        if let Some(terminal_symbol) = self.terminal_symbols.get(symbol) {
            Some(Rc::clone(terminal_symbol))
        } else if let Some(non_terminal_symbol) = self.non_terminal_symbols.get(symbol) {
            Some(Rc::clone(non_terminal_symbol))
        } else {
            #[cfg(feature = "debug_grammar")]
            println!("cannot find symbol {}", symbol);
            None
        }
    }

    fn parse_grammar(&mut self, text: &'static str) -> Result<(), GrammarError> {
        let mut tokens = text.split(" ");
        let lval = match tokens.next() {
            Some(token) => match self.get_symbol(token) {
                Some(symbol) => match *symbol {
                    Symbol::NonTerminal(_) => symbol,
                    _ => return Err(GrammarError::InvalidSymbol(token)),
                },
                None => return Err(GrammarError::InvalidSymbol(token)),
            },
            None => return Err(GrammarError::InvalidGrammarText(text)),
        };
        match tokens.next() {
            Some(token) if token == "->" => (),
            _ => return Err(GrammarError::InvalidGrammarText(text)),
        };
        let rvals = match tokens
            .map(|token| self.get_symbol(token))
            .collect::<Option<Vec<_>>>()
        {
            Some(rvals) => rvals,
            None => return Err(GrammarError::InvalidGrammarText(text)),
        };
        let grammar = Rc::new(Grammar {
            rule_number: self.grammars.len() + 1,
            lval,
            rvals,
        });
        #[cfg(feature = "debug_grammar")]
        println!("Parsed grammar: {}", grammar);
        self.grammars.push(grammar);
        Ok(())
    }

    pub fn find_grammars(&self, lval: Rc<Symbol<T>>) -> Vec<Rc<Grammar<T>>> {
        self.grammars
            .iter()
            .filter(|&g| lval == g.lval)
            .map(|g| Rc::clone(g))
            .collect()
    }
}
