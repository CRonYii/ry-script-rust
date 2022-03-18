mod math;
mod script;

#[cfg(test)]
mod math_tests;

pub use crate::math::matrix::*;
pub use crate::script::lexer::lexer_parse;

fn main() {
    let tokens = lexer_parse("let foo");
    println!("parsed = {:?}", tokens);
}