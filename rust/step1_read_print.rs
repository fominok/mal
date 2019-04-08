mod tokenizer;

use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::mem;
#[macro_use]
extern crate lazy_static;

// (println (* (+ 1 2) 5))

#[derive(Debug)]
enum Error {
    ParseError(String),
}


struct Parser {
    buffer: String,
    ast: Ast,
    in_string: bool,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            buffer: "".to_owned(),
            ast: Ast::List(vec![]),
            in_string: false,
        }
    }

    // fn tokenize(&mut self, s: String) -> Result<Vec<Token>, Error> {
    //     unimplemented!()
    // }

    fn parse(mut self, s: String) -> Result<Ast, Error> {
        for c in s.chars() {
            // if c.is_whitespace() {
            //     self.token_end()?;
            // } else {
            //     self.buffer.push(c);
            // }
        }
        self.token_end()?;
        //Ok(self.ast)
        unimplemented!()
    }

    fn try_bool(s: &str) -> Option<TokenType> {
        match s {
            "true" => Some(TokenType::Bool(true)),
            "false" => Some(TokenType::Bool(false)),
            _ => None,
        }
    }

    fn try_int(s: &str) -> Option<TokenType> {
        s.parse().ok().map(|x| TokenType::Number(x))
    }

    fn try_string(s: &str) -> Option<TokenType> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"^"([^"]|\\")*"$"#).expect("Error in predefined string regexp");
        }
        unimplemented!()
    }

    fn token_end(&mut self) -> Result<(), Error> {
        if !self.buffer.is_empty() {
            let b = mem::replace(&mut self.buffer, "".to_owned());
            unimplemented!()
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Ast {
    List(Vec<Ast>),
    Token(TokenType),
}

#[derive(Debug)]
enum TokenType {
    Symbol(String),
    Number(i32),
    Bool(bool),
    String(String),
}

fn read(s: String) -> Ast {
    let p = Parser::new();
    p.parse(s).expect("shit")
}

fn eval(ast: Ast) -> Ast {
    ast
}

fn print(ast: Ast) -> String {
    format!("{:?}", ast)
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
