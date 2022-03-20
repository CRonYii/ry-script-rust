use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub struct Grammar {
    pub lval: Symbol,
    pub rvals: Vec<Symbol>,
}

impl Grammar {
    pub fn equal(&self, other: &Grammar) -> bool {
        self.lval == other.lval &&
        self.rvals == other.rvals
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

pub struct GrammarSet {
    pub grammars: Vec<Grammar>,
}

impl GrammarSet {
    /* Pre-condition: The first grammar is expected to be the starter grammar */
    pub fn from(
        grammars_text: Vec<&'static str>,
        terminal_symbols: HashMap<&'static str, TokenType>,
    ) -> Result<GrammarSet, String> {
        // non-terminal symbols
        let mut non_terminal_symbols = HashSet::new();
        for text in &grammars_text {
            let mut tokens = text.split(" ");
            let lval = match tokens.next() {
                Some(token) => token,
                None => return Err(format!("Grammar has no lval")),
            };
            non_terminal_symbols.insert(lval);
        }
        // symbol getter closure
        let get_symbol = |symbol: &'_ str| -> Option<Symbol> {
            if let Some(terminal_symbol) = terminal_symbols.get(symbol) {
                Some(Symbol::Terminal(*terminal_symbol))
            } else if let Some(non_terminal_symbol) = non_terminal_symbols.get(symbol) {
                Some(Symbol::NonTerminal(*non_terminal_symbol))
            } else {
                #[cfg(debug_assertions)]
                println!("cannot find symbol {}", symbol);
                None
            }
        };

        // Parse a single string "lval -> [rval [..rval]]"
        let parse_grammar = |text: &str| -> Result<Grammar, String> {
            let mut tokens = text.split(" ");
            let lval = match tokens.next() {
                Some(token) => match get_symbol(token) {
                    Some(symbol) => match symbol {
                        Symbol::NonTerminal(_) => symbol,
                        _ => return Err(format!("lval {} is not a non-terminal symbol", token)),
                    },
                    None => {
                        return Err(format!("lval {} is not a valid non-terminal symbol", token))
                    }
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
                .map(|token| get_symbol(token))
                .collect::<Option<Vec<Symbol>>>()
            {
                Some(rvals) => rvals,
                None => return Err(format!("{} rvals contains invalid symbol", text)),
            };
            let grammar = Grammar { lval, rvals };
            #[cfg(debug_assertions)]
            println!("Parsed grammar: {}", grammar);
            Ok(grammar)
        };
        let grammars = grammars_text
            .iter()
            .map(|text| parse_grammar(*text))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(GrammarSet {
            grammars,
        })
    }

    pub fn find_grammars(&self, lval: Symbol) -> Vec<&Grammar> {
        self.grammars.iter().filter(|&g| g.lval == lval).collect()
    }
}
