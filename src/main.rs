use std::io::BufRead;

use unnamed_language::parser::scanner::Scanner;

fn main() {
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut scanner = Scanner::new(&line);

        loop {
            match scanner.scan() {
                Ok(tok) => {
                    let span = tok.span();
                    let kind = format!("{:?}", tok.kind());
                    let max = (line.len().max(1) as f32).log10() as usize + 1;
                    println!(
                        "[{:>max$}..{:<max$}] {:<12} | \"{}\"",
                        span.start(),
                        span.end(),
                        kind,
                        span.slice(),
                        max = max
                    );

                    if tok.is_eof() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("lexical error: \"{}\"", err.message);
                    let line_number = err.span.line_number();
                    let column_number = err.span.column_number();
                    let line = &line.lines().nth(line_number - 1).unwrap();

                    eprintln!("[{line_number}:{column_number}] {line}");
                }
            }
        }
    }
}
