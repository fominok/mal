mod formatter;
mod lexer;
mod reader;
mod reader_macros;

use crate::reader::{Ast, AstLeaf, ListType};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("transition error `{0}`")]
    TransitionError(String),
    #[error("token termination error `{0}`")]
    TokenTerminationError(String),
    #[error("unbalanced parens")]
    Unbalanced,
    #[error("eof while parsing a string")]
    EOF,
    #[error("reader macro error")]
    ReaderMacroError,
    #[error("eval error `{0}`")]
    EvalError(String),
}

struct Env(HashMap<String, Box<dyn Fn(Vec<f32>) -> f32>>);

fn read(s: String) -> Result<Ast, Error> {
    reader::read(s)
}

fn ast_to_f32(ast: Ast) -> Result<f32, Error> {
    if let Ast::Leaf(leaf) = ast {
        return match leaf {
            AstLeaf::Float(f) => Ok(f),
            AstLeaf::Int(i) => Ok(i as f32),
            _ => Err(Error::EvalError("cannot convert to f32".to_string())),
        };
    }
    Err(Error::EvalError("not a leaf".to_string()))
}

fn eval(ast: &mut Ast, env: &Env) -> Result<(), Error> {
    match ast {
        Ast::Leaf(leaf) => Ok(()),
        Ast::List(list) => {
            for l in list.list.iter_mut() {
                eval(l, env)?;
            }
            if list.list_type == ListType::Parens && !list.list.is_empty() {
                let args = list
                    .list
                    .drain(1..)
                    .map(ast_to_f32)
                    .collect::<Result<_, Error>>()?;
                let first = if let Ast::Leaf(leaf) = &list.list[0] {
                    if let AstLeaf::Symbol(sym) = leaf {
                        sym
                    } else {
                        todo!()
                    }
                } else {
                    todo!()
                };
                if let Some(f) = env.0.get(first) {
                    *ast = Ast::Leaf(AstLeaf::Float(f(args)))
                } else {
                    *ast = Ast::Leaf(AstLeaf::String(format!(r#""{}" not found"#, first)));
                };
            }
            Ok(())
        }
    }
}

fn print(ast: Ast) -> String {
    format!("{}", ast)
}

fn repl(s: String, env: &Env) -> String {
    if let Ok(mut r) = read(s) {
        match eval(&mut r, env) {
            Ok(_) => print(r),
            Err(e) => e.to_string(),
        }
    } else {
        "unbalanced".to_owned()
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let mut hm: HashMap<String, Box<Fn(Vec<f32>) -> f32>> = HashMap::new();
    hm.insert("+".to_owned(), Box::new(|args| args[0] + args[1]));
    hm.insert("-".to_owned(), Box::new(|args| args[0] - args[1]));
    hm.insert("*".to_owned(), Box::new(|args| args[0] * args[1]));
    hm.insert("/".to_owned(), Box::new(|args| args[0] / args[1]));

    let env = Env(hm);
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("{}", repl(line, &env));
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
