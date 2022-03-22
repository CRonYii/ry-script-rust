use std::collections::{HashMap, HashSet};

use super::grammar::TerminalSymbolDef;

pub struct LexerTokenMap<T> {
    pub eof: T,
    pub identifier: T,
    pub integer: T,
    pub float: T,
    pub string: T,
}

pub trait ParserToken<T: ParserToken<T>>:
    std::fmt::Debug + std::hash::Hash + PartialEq + Eq + Clone + Copy
{
    fn entity(self, value: String) -> Token<T>;
}

#[derive(Debug)]
pub struct Token<T: ParserToken<T>> {
    pub r#type: T,
    pub value: String,
}

pub struct SpecialTokenMap<T: ParserToken<T>> {
    operator_map: HashMap<&'static str, T>,
    keyword_map: HashMap<&'static str, T>,
    signs_chars_set: HashSet<char>,
}

impl<T: ParserToken<T>> SpecialTokenMap<T> {
    pub fn new(
        operator: &[TerminalSymbolDef<T>],
        keyword: &[TerminalSymbolDef<T>],
    ) -> SpecialTokenMap<T> {
        let mut set = SpecialTokenMap {
            operator_map: HashMap::new(),
            keyword_map: HashMap::new(),
            signs_chars_set: HashSet::new(),
        };
        // operators
        operator.iter().for_each(|def| {
            set.operator_map.insert(def.0, def.1);
        });
        for (sign, _) in &set.operator_map {
            for ch in sign.chars() {
                set.signs_chars_set.insert(ch);
            }
        }
        // keyword
        keyword.iter().for_each(|def| {
            set.keyword_map.insert(def.0, def.1);
        });
        set
    }

    pub fn get_sign_type(&self, sign: &str) -> Option<T> {
        match self.operator_map.get(sign) {
            Some(token) => Some(*token),
            None => None,
        }
    }

    pub fn get_keyword_type(&self, sign: &str) -> Option<T> {
        match self.keyword_map.get(sign) {
            Some(token) => Some(*token),
            None => None,
        }
    }

    pub fn is_keyword(&self, sign: &str) -> bool {
        self.keyword_map.contains_key(sign)
    }

    pub fn is_valid_sign(&self, sign: &str) -> bool {
        self.operator_map.contains_key(sign)
    }

    pub fn is_valid_sign_character(&self, ch: char) -> bool {
        self.signs_chars_set.contains(&ch)
    }
}

pub struct Tokens<T: ParserToken<T>>(pub Vec<Token<T>>);
