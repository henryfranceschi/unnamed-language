use std::{
    fs::File,
    io::{Write, Read},
    path::Path,
};

use unnamed_language::parser::Parser;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("usage: {} [filename]", env!("CARGO_BIN_NAME"));
        return;
    }

    if let Some(path) = args.get(1).map(Path::new) {
        run_from_file(path);
    } else {
        repl();
    }
}

fn repl() {
    loop {
        print!("> ");
        if let Err(error) = std::io::stdout().flush() {
            eprintln!("error: {error}");
            return;
        }

        let mut buf = String::new();
        match std::io::stdin().read_line(&mut buf) {
            Ok(0) => {
                println!("exiting...");
                break;
            }
            Ok(_) => {
                run(buf);
            }
            Err(error) => eprintln!("error: {error}"),
        }
    }
}

fn run_from_file(path: &Path) {
    if !path.is_file() {
        eprintln!("error: file {:?} not found", path);
        return;
    }

    let Ok(mut file) = File::open(path) else {
        eprintln!("error: file {:?} could not be opened", path);
        return;
    };

    let mut source = String::new();
    if file.read_to_string(&mut source).is_err() {
        eprintln!("error: file {:?} could not be read", path);
        return;
    }

    run(source);
}

fn run(source: String) {
    let mut parser = Parser::new(&source);
    match parser.expr() {
        Ok(expr) => {
            dbg!(expr);
        }
        Err(error) => {
            eprintln!("parsing error: {}", error.message());
        }
    }
}
