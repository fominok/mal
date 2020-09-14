mod formatter;
mod lexer;
mod reader;
mod reader_macros;

use crate::reader::{Ast, AstLeaf, ListType};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Error {
    TransitionError(String),
    TokenTerminationError(String),
    Unbalanced,
    EOF,
    ReaderMacroError,
}

struct Env(HashMap<String, Box<dyn Fn(Vec<f32>) -> f32>>);

fn read(s: String) -> Result<Ast, Error> {
    reader::read(s)
}

fn ast_to_f32(ast: Ast) -> f32 {
    if let Ast::Leaf(leaf) = ast {
        return match leaf {
            AstLeaf::Float(f) => f,
            AstLeaf::Int(i) => i as f32,
            _ => panic!("shi"),
        };
    }
    panic!("shi");
}

fn eval(ast: &mut Ast, env: &Env) {
    match ast {
        Ast::Leaf(leaf) => {}
        Ast::List(list) => {
            for l in list.list.iter_mut() {
                eval(l, env);
            }
            if list.list_type == ListType::Parens {
                let args = list.list.drain(1..).map(ast_to_f32).collect();
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
        }
    }
}

fn print(ast: Ast) -> String {
    format!("{}", ast)
}

fn repl(s: String, env: &Env) -> String {
    if let Ok(mut r) = read(s) {
        eval(&mut r, env);
        print(r)
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
