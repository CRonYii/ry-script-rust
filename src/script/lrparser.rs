use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    rc::Rc,
};

use super::{
    grammar::{Grammar, GrammarSet, Symbol},
    token::TokenType,
};

pub enum TransitionAction {
    Shift(usize),
    Reduce(usize),
    Goto(usize),
    Accept,
}

#[derive(Debug)]
struct ItemSet {
    items: Vec<Kernel>,
}

/* TODO: Should really use a hashset instead (or maybe a BTreeSet?)
 * Issue 1. cannot add while itering
 * Issue 2. needs to impl eq and hash for kernel
 */
impl ItemSet {
    fn has(&self, kernel: &Kernel) -> bool {
        match self.items.iter().find(|&k| k == kernel) {
            Some(_) => true,
            None => false,
        }
    }

    fn equal(&self, other: &ItemSet) -> bool {
        self.items.iter().filter(|&k| !other.has(k)).count() == 0
    }
}

impl Display for ItemSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.items.len())?;
        for (i, kernel) in self.items.iter().enumerate() {
            write!(f, "\n\t{}. {}", i + 1, kernel)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Kernel {
    grammar: Rc<Grammar>,
    rval_idx: usize,
}

impl Kernel {
    fn current_symbol(&self) -> Option<Rc<Symbol>> {
        match self.grammar.rvals.get(self.rval_idx) {
            None => None,
            Some(val) => Some(Rc::clone(val)),
        }
    }
}

impl Display for Kernel {
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

type TransitionTable = Vec<HashMap<Rc<Symbol>, TransitionAction>>;

pub fn lr0_parse(grammar_set: &GrammarSet) -> Result<TransitionTable, String> {
    let mut transition_table: TransitionTable = Vec::new();
    let starter_grammar = match grammar_set.grammars.get(0) {
        None => return Err(format!("Grammar set does not have a starter grammar")),
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
        let mut transition_row = HashMap::<Rc<Symbol>, TransitionAction>::new();
        let mut new_itemsets: HashMap<Rc<Symbol>, ItemSet> = HashMap::new();
        let mut i = 0;
        while let Some(kernel) = item_set.items.get(i) {
            match kernel.current_symbol() {
                Some(symbol) => {
                    match *symbol {
                        Symbol::Terminal(TokenType::EOF) => {
                            // EOF - Accept
                            transition_row.insert(symbol, TransitionAction::Accept);
                        }
                        Symbol::NonTerminal(_) => {
                            // expand item sets
                            let mut kernels: Vec<Kernel> = grammar_set
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
                _ => (/* TODO reduce */),
            }
            i += 1;
        }
        // populate new itemsets
        for kernel in &item_set.items {
            match kernel.current_symbol() {
                Some(symbol) => match *symbol {
                    Symbol::Terminal(TokenType::EOF) => (/* do nothing */),
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
        #[cfg(debug_assertions)]
        println!("Parsed Itemset {}: {}\n", transition_table.len(), item_set);
        transition_table.push(transition_row);
        // TODO: shift/goto action
        // add new itemsets to vector
        let mut newsets: Vec<ItemSet> = new_itemsets
            .into_iter()
            .map(|(_, value)| value)
            .filter(|x| match item_sets.iter().find(|&y| x.equal(y)) {
                None => true,
                _ => false,
            })
            .collect();
        item_sets.append(&mut newsets);
        item_set_idx += 1;
    }
    Ok(transition_table)
}
