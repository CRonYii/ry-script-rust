use std::io::*;

mod math;
mod script;

#[cfg(test)]
mod math_tests;

pub use crate::math::matrix::*;
pub use crate::script::lexer::Lexer;

fn main() {
    let mut line = String::new();
    let mut lexer = Lexer::new();
    loop {
        print!("> ");
        match stdout().flush() {
            Ok(_) => (),
            Err(error) => println!("{}", error),
        }
        let _ = stdin().read_line(&mut line).unwrap();
        if line.starts_with(".exit") {
            break;
        }
        match lexer.parse(&line) {
            Ok(tokens) => {
                dbg!(tokens);
                ()
            }
            Err(err) => eprintln!("{}", err),
        }
        line.clear();
    }
}
