use std::path::Path;

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
    todo!()
}

fn run_from_file(path: &Path) {
    todo!()
}

fn run(source: String) {
    Parser::new(&source);
}
