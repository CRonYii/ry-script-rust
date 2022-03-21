use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use super::{runner::GrammarRule, token::TokenType, ast::RuntimeValue};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Symbol {
    NonTerminal(&'static str),
    Terminal(TokenType),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::NonTerminal(s) => write!(f, "{}", s)?,
            Symbol::Terminal(t) => write!(f, "{:?}", t)?,
        }
        Ok(())
    }
}

#[derive(Debug, Hash, Eq)]
pub struct Grammar {
    pub rule_number: usize,
    pub lval: Rc<Symbol>,
    pub rvals: Vec<Rc<Symbol>>,
}

impl PartialEq for Grammar {
    fn eq(&self, other: &Self) -> bool {
        self.rule_number == other.rule_number
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> ", self.lval)?;
        for rval in &self.rvals {
            write!(f, "{} ", rval)?
        }
        Ok(())
    }
}

type SymbolMap = HashMap<&'static str, Rc<Symbol>>;

pub struct TerminalSymbolDef(pub &'static str, pub TokenType);

pub struct GrammarSet {
    pub grammars: Vec<Rc<Grammar>>,
    pub terminal_symbols: SymbolMap,
    pub non_terminal_symbols: SymbolMap,
}

impl GrammarSet {
    /* Pre-condition: The first grammar is expected to be the starter grammar */
    pub fn from<T: RuntimeValue>(
        grammars: &Vec<GrammarRule<T>>,
        terminals: &[TerminalSymbolDef],
    ) -> Result<GrammarSet, String> {
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
                None => return Err(format!("Grammar has no lval")),
            };
            non_terminal_symbols.insert(lval, Rc::new(Symbol::NonTerminal(lval)));
        }
        let mut grammar = GrammarSet {
            grammars: vec![],
            terminal_symbols,
            non_terminal_symbols,
        };
        for text in grammars {
            grammar.parse_grammar(text.0)?;
        }
        Ok(grammar)
    }

    fn get_symbol(&self, symbol: &'static str) -> Option<Rc<Symbol>> {
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

    fn parse_grammar(&mut self, text: &'static str) -> Result<(), String> {
        let mut tokens = text.split(" ");
        let lval = match tokens.next() {
            Some(token) => match self.get_symbol(token) {
                Some(symbol) => match *symbol {
                    Symbol::NonTerminal(_) => symbol,
                    _ => return Err(format!("lval {} is not a non-terminal symbol", token)),
                },
                None => return Err(format!("lval {} is not a valid non-terminal symbol", token)),
            },
            None => return Err(format!("Grammar has no lval")),
        };
        match tokens.next() {
            Some(token) if token == "->" => (),
            _ => {
                return Err(format!(
                    "{}, Expected lval -> rvals, but -> is not presented",
                    text
                ))
            }
        };
        let rvals = match tokens
            .map(|token| self.get_symbol(token))
            .collect::<Option<Vec<_>>>()
        {
            Some(rvals) => rvals,
            None => return Err(format!("{} rvals contains invalid symbol", text)),
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

    pub fn find_grammars(&self, lval: Rc<Symbol>) -> Vec<Rc<Grammar>> {
        self.grammars
            .iter()
            .filter(|&g| lval == g.lval)
            .map(|g| Rc::clone(g))
            .collect()
    }
}
