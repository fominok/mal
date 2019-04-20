mod formatter;
mod lexer;
mod reader;
mod reader_macros;

use crate::reader::Ast;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Debug, PartialEq)]
pub enum Error {
    TransitionError(String),
    TokenTerminationError(String),
    Unbalanced,
    EOF,
    ReaderMacroError,
}

fn read(s: String) -> Result<Ast, Error> {
    reader::read(s)
}

fn eval(ast: Ast) -> Ast {
    ast
}

fn print(ast: Ast) -> String {
    format!("{}", ast)
}

fn repl(s: String) -> String {
    if let Ok(r) = read(s) {
        print(eval(r))
    } else {
        "unbalanced".to_owned()
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                println!("{}", repl(line));
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
