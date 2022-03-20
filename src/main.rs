use std::io;

use ry_mathr::init_math_script_parser;

fn main() {
    use std::io::Write;
    let mut lr_parser = init_math_script_parser().unwrap();
    let mut line = String::new();
    loop {
        print!("> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(error) => eprintln!("stdout error: {}", error),
        }
        let _ = io::stdin().read_line(&mut line).unwrap();
        if line.starts_with(".exit") {
            break;
        }
        match lr_parser.parse(&line) {
            Ok(node) => println!("> {}", node),
            Err(err) => eprintln!("{}", err),
        }
        line.clear();
    }
}
