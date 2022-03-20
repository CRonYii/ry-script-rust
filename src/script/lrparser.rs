use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    rc::Rc,
};

use super::{
    grammar::{Grammar, GrammarSet, Symbol},
    token::{TokenType, Tokens},
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
}

impl PartialEq for ItemSet {
    fn eq(&self, other: &Self) -> bool {
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

pub struct LRParser {
    transition_table: TransitionTable,
    grammar_set: GrammarSet,
}

impl LRParser {
    pub fn parse(&self, tokens: Tokens) -> Result<(), String> {
        /* parse stack initial state 0 */
        let mut parse_stack = Vec::from([0]);
        let mut iter = tokens.0.into_iter();
        let mut next_char = false;
        let mut token = match iter.next() {
            Some(token) => token,
            None => return Err(format!("Parse Error: Incorrect syntax")),
        };
        while parse_stack.len() != 0 {
            if next_char {
                token = match iter.next() {
                    Some(token) => token,
                    None => return Err(format!("Parse Error: Incorrect syntax")),
                };
            }
            let state = match parse_stack.last() {
                Some(&state) => state,
                None => return Err(format!("Parse Error: stack is empty when peek")),
            };
            let action = match self.transition_table.get(state) {
                Some(row) => match row.get(&Symbol::Terminal(token.r#type)) {
                    Some(action) => action,
                    None => return Err(format!("Parse Error: Unexpected token: {}", token.value)),
                },
                None => return Err(format!("Parse Error: state {} does not exist", state)),
            };
            #[cfg(debug_assertions)]
            println!("{:?} -> [{}] {}", token.r#type, state, action);
            next_char = match action {
                TransitionAction::Shift(state) => {
                    parse_stack.push(*state);
                    /* TODO: push AST stack */
                    true
                }
                TransitionAction::Reduce(rule_number) => {
                    let grammar = match self.grammar_set.grammars.get(*rule_number - 1) {
                        Some(grammar) => grammar,
                        None => {
                            return Err(format!(
                                "Parse Error: Grammar rule {} does not exist",
                                rule_number
                            ))
                        }
                    };
                    /* TODO: pop AST stack to form AST params and the push a new AST expression */
                    if grammar.rvals.len() > parse_stack.len() {
                        return Err(format!("Parse Error: stack does not have enough items"));
                    }
                    parse_stack.truncate(parse_stack.len() - grammar.rvals.len());
                    /* Perform GOTO */
                    let state = match parse_stack.last() {
                        Some(&state) => state,
                        None => return Err(format!("Parse Error: stack is empty when goto")),
                    };
                    let goto_state = match self.transition_table.get(state) {
                        Some(row) => match row.get(&grammar.lval) {
                            Some(TransitionAction::Goto(state)) => state,
                            _ => {
                                return Err(format!(
                                    "Parse Error: cannot find goto action for: {}",
                                    grammar.lval
                                ))
                            }
                        },
                        None => return Err(format!("Parse Error: state {} does not exist", state)),
                    };
                    parse_stack.push(*goto_state);
                    false
                }
                TransitionAction::Accept => return Ok((/* TODO: Return an AST expression */)),
                TransitionAction::Goto(_) => {
                    return Err(format!("Parse Error: Unexpected goto action"))
                }
            }
        }
        Err(format!("Parse Error: Incorrect syntax"))
    }

    pub fn lr0_parser(grammar_set: GrammarSet) -> Result<LRParser, String> {
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
            transition_table,
            grammar_set,
        })
    }
}

impl Display for LRParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for terminal_symbol in self.grammar_set.terminal_symbols.values() {
            write!(f, "| {:<10} ", format!("{}", terminal_symbol))?;
        }
        for non_terminal_symbol in self.grammar_set.non_terminal_symbols.values() {
            write!(f, "| {:<10} ", format!("{}", non_terminal_symbol))?;
        }
        writeln!(f)?;
        for state in &self.transition_table {
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
