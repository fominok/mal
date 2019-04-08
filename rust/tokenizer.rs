use std::mem;

#[derive(Debug)]
enum Error {
    TransitionError(String),
    TokenTerminationError(String),
}

macro_rules! trans_err {
    ($c:expr, $buff:expr, $state:expr) => {
        Err(Error::TransitionError(format!(
            "Unexpected character: {}, buffer: {}, state: {:?}",
            $c, $buff, $state
        )))
    };
}

macro_rules! match_terminated {
   ($tokenizer:expr, $obj:expr, $($matcher:pat $(if $pred:expr)* => $result:expr),*) => {
       match $obj {
           ' ' => Ok($tokenizer.end_token()?),
           '(' => Ok($tokenizer.end_token_push(Token::LeftParen)?),
           ')' => Ok($tokenizer.end_token_push(Token::RightParen)?),
           $($matcher $(if $pred)* => $result),*
       }
   }
}

#[derive(Debug)]
enum State {
    Init,
    Minus,
    Num,
    Dot,
    Float,
    StringStart,
    Escape,
    StringClose,
    Symbol,
}

struct Tokenizer {
    state: State,
    tokens: Vec<Token>,
    buffer: String,
}

#[derive(Debug, PartialEq)]
enum TokenNum {
    Int(i32),
    Float(f32),
}

#[derive(Debug, PartialEq)]
enum Token {
    LeftParen,
    RightParen,
    String(String),
    Num(TokenNum),
    Symbol(String),
}

impl Tokenizer {
    fn new() -> Self {
        Tokenizer {
            state: State::Init,
            tokens: vec![],
            buffer: "".to_owned(),
        }
    }

    fn trans(&mut self, c: char, state: State) {
        self.state = state;
        self.buffer.push(c);
    }

    fn push_buffer(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn end_token(&mut self) -> Result<(), Error> {
        let b = mem::replace(&mut self.buffer, "".to_owned());
        let token = match &self.state {
            State::Minus => Ok(Token::Symbol(b)),
            State::Num => Ok(Token::Num(TokenNum::Int(
                b.parse().expect("Programming error in tokenizer"),
            ))),
            State::Float => Ok(Token::Num(TokenNum::Float(
                b.parse().expect("Programming error in tokenizer"),
            ))),
            State::StringClose => Ok(Token::String(b)),
            State::Symbol => Ok(Token::Symbol(b)),
            _ => Err(Error::TokenTerminationError(format!(
                "Non terminating state: {:?}",
                self.state
            ))),
        }?;
        self.push_token(token);
        self.state = State::Init;
        Ok(())
    }

    fn end_token_push(&mut self, t: Token) -> Result<(), Error> {
        self.end_token()?;
        self.tokens.push(t);
        Ok(())
    }

    fn push_token(&mut self, t: Token) {
        self.tokens.push(t);
    }

    fn trans_init(&mut self, c: char) -> Result<(), Error> {
        match c {
            ' ' => Ok(()),
            '(' => Ok(self.push_token(Token::LeftParen)),
            ')' => Ok(self.push_token(Token::RightParen)),
            '-' => Ok(self.trans(c, State::Minus)),
            '"' => Ok(self.trans(c, State::StringStart)),
            c if c.is_digit(10) => Ok(self.trans(c, State::Num)),
            c if !c.is_digit(10) && c != '"' && c != '-' => Ok(self.trans(c, State::Symbol)),
            _ => trans_err!(c, self.buffer, State::Init),
        }
    }

    fn trans_num(&mut self, c: char) -> Result<(), Error> {
        match_terminated! {self, c,
            c if c.is_digit(10) => Ok(self.push_buffer(c)),
            '.' => Ok(self.trans(c, State::Dot)),
            _ => trans_err!(c, self.buffer, State::Num)
        }
    }

    fn trans_dot(&mut self, c: char) -> Result<(), Error> {
        match c {
            c if c.is_digit(10) => Ok(self.trans(c, State::Float)),
            _ => trans_err!(c, self.buffer, State::Dot),
        }
    }

    fn trans_float(&mut self, c: char) -> Result<(), Error> {
        match_terminated! {self, c,
            c if c.is_digit(10) => Ok(self.push_buffer(c)),
            _ => trans_err!(c, self.buffer, State::Float)
        }
    }

    fn trans_string_start(&mut self, c: char) -> Result<(), Error> {
        match c {
            '"' => Ok(self.trans(c, State::StringClose)),
            '\\' => Ok(self.trans(c, State::Escape)),
            c if c != '"' => Ok(self.push_buffer(c)),
            _ => trans_err!(c, self.buffer, State::StringStart),
        }
    }

    fn trans_escape(&mut self, c: char) -> Result<(), Error> {
        match c {
            _ => Ok(self.trans(c, State::StringStart)),
        }
    }

    fn trans_string_close(&mut self, c: char) -> Result<(), Error> {
        match_terminated! {self, c,
            _ => trans_err!(c, self.buffer, State::StringClose)
        }
    }

    fn trans_minus(&mut self, c: char) -> Result<(), Error> {
        match_terminated! {self, c,
            c if c.is_digit(10) => Ok(self.trans(c, State::Num)),
            c if c != '"' => Ok(self.trans(c, State::Symbol)),
            _ => trans_err!(c, self.buffer, State::Minus)
        }
    }

    fn trans_symbol(&mut self, c: char) -> Result<(), Error> {
        match_terminated! {self, c,
            c if c != '"' => Ok(self.push_buffer(c)),
            _ => trans_err!(c, self.buffer, State::Symbol)
        }
    }

    fn process_char(&mut self, c: char) -> Result<(), Error> {
        match self.state {
            State::Init => self.trans_init(c),
            State::Minus => self.trans_minus(c),
            State::Num => self.trans_num(c),
            State::Dot => self.trans_dot(c),
            State::Float => self.trans_float(c),
            State::StringStart => self.trans_string_start(c),
            State::Escape => self.trans_escape(c),
            State::StringClose => self.trans_string_close(c),
            State::Symbol => self.trans_symbol(c),
        }
    }

    fn tokenize(mut self, input: &str) -> Result<Vec<Token>, Error> {
        for c in input.chars() {
            self.process_char(c)?;
        }
        self.end_token()?;
        Ok(self.tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize_int() {
        let t = Tokenizer::new();
        let tokens = t.tokenize("1337 420 -322").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Num(TokenNum::Int(1337)),
                Token::Num(TokenNum::Int(420)),
                Token::Num(TokenNum::Int(-322))
            ]
        );
    }
}
