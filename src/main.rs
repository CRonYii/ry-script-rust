use std::io;

use ry_mathr::init_math_script_parser;

fn main() {
    use std::io::Write;
    let mut script_runner = match init_math_script_parser() {
        Ok(lr_parser) => lr_parser,
        Err(err) => {
            eprintln!("Failed during initialization: {}", err);
            std::process::exit(1);
        }
    };
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
        match script_runner.run(&line) {
            Ok(node) => println!("> {}", node),
            Err(err) => eprintln!("{}", err),
        }
        line.clear();
    }
}
