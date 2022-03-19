use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::token::TokenType;

#[derive(Debug)]
enum Symbol {
    NonTerminal(&'static str),
    Termial(TokenType),
}

pub struct Grammar {
    lval: Symbol,
    rvals: Vec<Symbol>,
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Grammar: {:?} -> ", self.lval)?;
        for rval in &self.rvals {
            write!(f, "{:?} ", rval)?
        }
        Ok(())
    }
}

pub struct GrammarSet {
    pub start_grammar: Grammar,
    pub grammars: Vec<Grammar>,
    pub n_non_terminal_symbols: usize,
    pub n_terminal_symbols: usize,
    pub end_token: TokenType,
}

impl GrammarSet {
    pub fn from(
        start_grammar_text: &'static str,
        grammars: Vec<&'static str>,
        terminal_symbols: HashMap<&'static str, TokenType>,
        end_token: TokenType,
    ) -> Result<GrammarSet, String> {
        // non-terminal symbols
        let mut non_terminal_symbols = HashSet::new();
        for text in &grammars {
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
                Some(Symbol::Termial(*terminal_symbol))
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
            println!("{}", grammar);
            Ok(grammar)
        };
        let grammar_rules = grammars
            .iter()
            .map(|text| parse_grammar(*text))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(GrammarSet {
            start_grammar: parse_grammar(start_grammar_text)?,
            grammars: grammar_rules,
            n_non_terminal_symbols: non_terminal_symbols.len(),
            n_terminal_symbols: terminal_symbols.len(),
            end_token,
        })
    }
}
