mod formatter;
mod lexer;
mod reader;
mod reader_macros;

use crate::reader::{Ast, AstLeaf, ListType};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
    #[error("{0}")]
    EvalError(String),
}

struct Env {
    env: HashMap<String, Ast>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub(crate) fn search(&self, symbol: &str) -> Result<Ast, Error> {
        let err = Error::EvalError(format!("'{}' not found", symbol));

        if let Some(value) = self.env.get(symbol) {
            Ok(value.clone())
        } else {
            self.parent
                .clone()
                .map(|p| p.borrow().search(symbol))
                .ok_or(err)?
        }
    }
}

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

fn function_call(
    mut args: Vec<Ast>,
    env: Rc<RefCell<Env>>,
    f: Rc<dyn Fn(Vec<Ast>) -> Result<Ast, Error>>,
) -> Result<Ast, Error> {
    for l in args.iter_mut() {
        eval(l, env.clone())?;
    }
    f(args)
}

fn eval(ast: &mut Ast, env: Rc<RefCell<Env>>) -> Result<(), Error> {
    match ast {
        Ast::Leaf(leaf) => {
            // resolve symbols
            match leaf {
                AstLeaf::Symbol(sym) => {
                    *ast = env.borrow().search(sym)?;
                }
                _ => {}
            };
            Ok(())
        }
        Ast::List(list) => {
            if list.list_type == ListType::Parens && !list.list.is_empty() {
                let mut args: Vec<Ast> = list.list.drain(1..).collect();
                match list.list[0].get_leaf()? {
                    AstLeaf::Symbol(s) => {
                        if s == "let*" {
                            let inner_env = Rc::new(RefCell::new(Env {
                                env: HashMap::new(),
                                parent: Some(env),
                            }));
                            for bind in args[0].get_any_list_mut()?.chunks_mut(2) {
                                eval(&mut bind[1], inner_env.clone())?;
                                inner_env
                                    .borrow_mut()
                                    .env
                                    .insert(bind[0].get_symbol()?, bind[1].clone());
                            }

                            let mut inner_args: Ast =
                                args.drain(1..).collect::<Vec<Ast>>()[0].clone();
                            *ast = inner_args;
                            eval(ast, inner_env)?;
                        } else if s == "def!" {
                            eval(&mut args[1], env.clone())?;
                            env.borrow_mut()
                                .env
                                .insert(args[0].get_symbol()?, args[1].clone());
                            *ast = args[1].clone();
                        } else {
                            let f_ast = env.borrow().search(s)?;
                            let f = f_ast.get_function()?;
                            *ast = function_call(args, env, f.clone())?;
                        }
                    }
                    AstLeaf::Function(f) => {
                        *ast = function_call(args, env, f.f.clone())?;
                    }
                    _ => return Err(Error::EvalError("not a function".to_owned())),
                };
            // let f = list.list[0].get_function()?;
            // *ast = f(args)?;
            } else {
                for l in list.list.iter_mut() {
                    eval(l, env.clone())?;
                }
            }
            Ok(())
        }
    }
}

fn print(ast: Ast) -> String {
    format!("{}", ast)
}

fn repl(s: String, env: Rc<RefCell<Env>>) -> String {
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
    let mut hm: HashMap<String, Ast> = HashMap::new();
    hm.insert(
        "+".to_owned(),
        Ast::function(|args| {
            Ok(Ast::Leaf(AstLeaf::Float(
                ast_to_f32(args[0].clone())? + ast_to_f32(args[1].clone())?,
            )))
        }),
    );
    hm.insert(
        "-".to_owned(),
        Ast::function(|args| {
            Ok(Ast::Leaf(AstLeaf::Float(
                ast_to_f32(args[0].clone())? - ast_to_f32(args[1].clone())?,
            )))
        }),
    );
    hm.insert(
        "*".to_owned(),
        Ast::function(|args| {
            Ok(Ast::Leaf(AstLeaf::Float(
                ast_to_f32(args[0].clone())? * ast_to_f32(args[1].clone())?,
            )))
        }),
    );
    hm.insert(
        "/".to_owned(),
        Ast::function(|args| {
            Ok(Ast::Leaf(AstLeaf::Float(
                ast_to_f32(args[0].clone())? / ast_to_f32(args[1].clone())?,
            )))
        }),
    );

    // hm.insert("-".to_owned(), Box::new(|args| args[0] - args[1]));
    // hm.insert("*".to_owned(), Box::new(|args| args[0] * args[1]));
    // hm.insert("/".to_owned(), Box::new(|args| args[0] / args[1]));

    let env = Rc::new(RefCell::new(Env {
        env: hm,
        parent: None,
    }));
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("{}", repl(line, env.clone()));
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
