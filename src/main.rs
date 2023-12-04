use std::io::BufRead;

use thing::parser::scanner::Scanner;

fn main() {
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let scanner = Scanner::new(&line);

        for token in scanner {
            match token {
                Ok(tok) => {
                    println!("token: {tok:?}");
                    if tok.is_eof() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("lexical error: \"{}\"", err.message);
                }
            }
        }
    }
}
