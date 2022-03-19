use super::grammar::GrammarSet;

enum TransitionAction {
    Shift,
    Reduce,
    Goto,
    Accept,
}

pub fn lr0_parse(grammar_set: &GrammarSet) {
    let n_symbols = grammar_set.n_terminal_symbols + grammar_set.n_non_terminal_symbols;
    let transition_table: Vec<Vec<TransitionAction>> = Vec::new();
    let kernels = Vec::from([1]);
}
