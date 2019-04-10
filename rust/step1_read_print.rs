mod lexer;
use std::fmt;
use std::mem;

use lexer::Token;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Debug, PartialEq)]
pub enum Error {
    TransitionError(String),
    TokenTerminationError(String),
}

#[derive(Debug)]
enum AstLeaf {
    Symbol(String),
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Debug)]
enum Ast {
    List(Vec<Ast>),
    Leaf(AstLeaf),
}

impl fmt::Display for AstLeaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstLeaf::Symbol(x) => write!(f, "{}", x),
            AstLeaf::String(x) => write!(f, "\"{}\"", x),
            AstLeaf::Int(x) => write!(f, "{}", x.to_string()),
            AstLeaf::Float(x) => write!(f, "{}", x.to_string()),
        }
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Leaf(x) => write!(f, "{}", x),
            Ast::List(xs) => {
                if xs.is_empty() {
                    write!(f, "()")
                } else {
                    let mut iter = xs.iter();
                    write!(f, "({}", iter.next().expect("What could go wrong"))?;
                    for i in iter {
                        write!(f, " {}", i)?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
}

fn parse(lexemes: Vec<Token>) -> Result<Ast, Error> {
    let mut stack_lists: Vec<Vec<Ast>> = Vec::new();
    let mut current_list: Vec<Ast> = Vec::new();
    for l in lexemes.into_iter() {
        match l {
            Token::String(x) => current_list.push(Ast::Leaf(AstLeaf::String(x))),
            Token::Int(x) => current_list.push(Ast::Leaf(AstLeaf::Int(x))),
            Token::Float(x) => current_list.push(Ast::Leaf(AstLeaf::Float(x))),
            Token::Symbol(x) => current_list.push(Ast::Leaf(AstLeaf::Symbol(x))),
            Token::LeftParen => stack_lists.push(mem::replace(&mut current_list, Vec::new())),
            Token::RightParen => {
                let parent_list = stack_lists.pop().unwrap();
                let child_list = mem::replace(&mut current_list, parent_list);
                current_list.push(Ast::List(child_list));
            }
        }
    }
    Ok(current_list.pop().unwrap())
}

fn read(s: String) -> Ast {
    let lex = lexer::Lexer::new();
    let tokens = lex.tokenize(&s).unwrap();
    parse(tokens).unwrap()
}

fn eval(ast: Ast) -> Ast {
    ast
}

fn print(ast: Ast) -> String {
    format!("{}", ast)
}

fn repl(s: String) -> String {
    print(eval(read(s)))
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
