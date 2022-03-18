mod math;
mod script;

#[cfg(test)]
mod math_tests;

pub use crate::math::matrix::*;
pub use crate::script::lexer::lexer_parse;

fn main() {
    let tokens = lexer_parse(
        "
        # simple variable declartion
        foo = 1;
        bar = \"hello world\";
        println(bar * 3);
        ",
    );
    println!("parsed = {:?}", tokens);
}
