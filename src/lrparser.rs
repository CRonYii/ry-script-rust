use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    rc::Rc,
};

use super::{
    error::{GrammarError, ParseError},
    grammar::{Grammar, GrammarSet, Symbol},
    token::ParserToken,
};

pub enum TransitionAction {
    Shift(usize),
    Reduce(usize),
    Goto(usize),
    Accept,
}

impl Display for TransitionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionAction::Shift(s) => write!(f, "shift {}", s)?,
            TransitionAction::Reduce(r) => write!(f, "reduce {}", r)?,
            TransitionAction::Goto(r) => write!(f, "goto {}", r)?,
            TransitionAction::Accept => write!(f, "accept")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ItemSet<T: ParserToken<T>> {
    items: Vec<Kernel<T>>,
}

/* TODO: Should really use a hashset instead (or maybe a BTreeSet?)
 * Issue 1. cannot add while itering
 * Issue 2. needs to impl eq and hash for kernel
 */
impl<T: ParserToken<T>> ItemSet<T> {
    fn has(&self, kernel: &Kernel<T>) -> bool {
        match self.items.iter().find(|&k| k == kernel) {
            Some(_) => true,
            None => false,
        }
    }
}

impl<T: ParserToken<T>> PartialEq for ItemSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items.iter().filter(|&k| !other.has(k)).count() == 0
    }
}

impl<T: ParserToken<T>> Display for ItemSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.items.len())?;
        for (i, kernel) in self.items.iter().enumerate() {
            write!(f, "\n\t{}. {}", i + 1, kernel)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Kernel<T: ParserToken<T>> {
    grammar: Rc<Grammar<T>>,
    rval_idx: usize,
}

impl<T: ParserToken<T>> Kernel<T> {
    fn current_symbol(&self) -> Option<Rc<Symbol<T>>> {
        match self.grammar.rvals.get(self.rval_idx) {
            None => None,
            Some(val) => Some(Rc::clone(val)),
        }
    }
}

impl<T: ParserToken<T>> Display for Kernel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> ", self.grammar.lval)?;
        for (i, rval) in self.grammar.rvals.iter().enumerate() {
            if i == self.rval_idx {
                write!(f, "• ")?;
            }
            write!(f, "{} ", rval)?
        }
        if self.rval_idx == self.grammar.rvals.len() {
            write!(f, "• ")?;
        }
        Ok(())
    }
}

pub struct LRParser<T: ParserToken<T>> {
    pub grammar_set: GrammarSet<T>,
    table: Vec<HashMap<Rc<Symbol<T>>, TransitionAction>>,
}

impl<T: ParserToken<T>> LRParser<T> {
    pub fn lr0(grammar_set: GrammarSet<T>) -> Result<LRParser<T>, GrammarError> {
        let mut transition_table = Vec::new();
        let starter_grammar = match grammar_set.grammars.get(0) {
            None => {
                return Err(GrammarError::Error(
                    "Grammar set does not have a starter grammar",
                ))
            }
            Some(rule) => rule,
        };
        let starter_item_set = ItemSet {
            items: vec![Kernel {
                grammar: Rc::clone(starter_grammar),
                rval_idx: 0,
            }],
        };
        let mut item_sets = Vec::from([starter_item_set]);
        let mut item_set_idx = 0;
        while let Some(item_set) = item_sets.get_mut(item_set_idx) {
            // populate a transition row
            let mut transition_row = HashMap::<Rc<Symbol<T>>, TransitionAction>::new();
            let mut new_itemsets: HashMap<Rc<Symbol<T>>, ItemSet<T>> = HashMap::new();
            let mut i = 0;
            while let Some(kernel) = item_set.items.get(i) {
                match kernel.current_symbol() {
                    Some(symbol) => {
                        match *symbol {
                            Symbol::Terminal(t) if t == grammar_set.eof => {
                                // EOF - Accept
                                transition_row.insert(symbol, TransitionAction::Accept);
                            }
                            Symbol::NonTerminal(_) => {
                                // expand item sets
                                let mut kernels: Vec<Kernel<T>> = grammar_set
                                    .find_grammars(symbol)
                                    .into_iter()
                                    .map(|grammar| Kernel {
                                        grammar,
                                        rval_idx: 0,
                                    })
                                    .filter(|k| !item_set.has(k))
                                    .collect();
                                item_set.items.append(&mut kernels);
                            }
                            _ => (/* do nothing for terminal symbols */),
                        }
                    }
                    _ => grammar_set.terminal_symbols.iter().for_each(|(_, symbol)| {
                        transition_row.insert(
                            Rc::clone(symbol),
                            TransitionAction::Reduce(kernel.grammar.rule_number),
                        );
                    }),
                }
                i += 1;
            }
            // populate new itemsets
            for kernel in &item_set.items {
                match kernel.current_symbol() {
                    Some(symbol) => match *symbol {
                        Symbol::Terminal(t) if t == grammar_set.eof => (/* do nothing */),
                        _ => {
                            let new_kernel = Kernel {
                                grammar: Rc::clone(&kernel.grammar),
                                rval_idx: kernel.rval_idx + 1,
                            };
                            let new_item_set = match new_itemsets.entry(symbol) {
                                Entry::Occupied(o) => o.into_mut(),
                                Entry::Vacant(v) => v.insert(ItemSet { items: vec![] }),
                            };
                            new_item_set.items.push(new_kernel);
                        }
                    },
                    None => (/* do nothing */),
                }
            }
            #[cfg(feature = "debug_lrparser")]
            println!("Parsed Itemset {}: {}\n", transition_table.len(), item_set);
            // add new itemsets to vector
            new_itemsets.into_iter().for_each(|(symbol, itemset)| {
                let state = match item_sets.iter().position(|set| itemset == *set) {
                    None => -1,
                    Some(idx) => idx as i32,
                };
                let action_value = match state == -1 {
                    true => item_sets.len(),
                    false => state as usize,
                };
                // shift/goto
                match *symbol {
                    Symbol::Terminal(_) => {
                        transition_row.insert(symbol, TransitionAction::Shift(action_value))
                    }
                    Symbol::NonTerminal(_) => {
                        transition_row.insert(symbol, TransitionAction::Goto(action_value))
                    }
                };

                // add brand new itemset
                if state == -1 {
                    item_sets.push(itemset);
                }
            });
            transition_table.push(transition_row);
            item_set_idx += 1;
        }
        Ok(LRParser {
            grammar_set,
            table: transition_table,
        })
    }

    pub fn get_action(
        &self,
        state: usize,
        symbol: &Symbol<T>,
    ) -> Result<&TransitionAction, ParseError> {
        match self.table.get(state) {
            Some(row) => match row.get(symbol) {
                Some(action) => Ok(action),
                None => return Err(ParseError::UnexpectedSymbol(format!("{}", symbol))),
            },
            None => return Err(ParseError::StateDoesNotExist(state)),
        }
    }
}

impl<T: ParserToken<T>> Display for LRParser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for terminal_symbol in self.grammar_set.terminal_symbols.values() {
            write!(f, "| {:<10} ", format!("{}", terminal_symbol))?;
        }
        for non_terminal_symbol in self.grammar_set.non_terminal_symbols.values() {
            write!(f, "| {:<10} ", format!("{}", non_terminal_symbol))?;
        }
        writeln!(f)?;
        for state in &self.table {
            for terminal_symbol in self.grammar_set.terminal_symbols.values() {
                if let Some(action) = state.get(terminal_symbol) {
                    write!(f, "| {:<10} ", format!("{}", action))?;
                } else {
                    write!(f, "| {:<10} ", "")?;
                }
            }
            for non_terminal_symbol in self.grammar_set.non_terminal_symbols.values() {
                if let Some(action) = state.get(non_terminal_symbol) {
                    write!(f, "| {:<10} ", format!("{}", action))?;
                } else {
                    write!(f, "| {:<10} ", "")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
