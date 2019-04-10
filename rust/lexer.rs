use std::mem;
use super::Error;

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
           x if x.is_whitespace() => Ok($tokenizer.end_token()?),
           ',' => Ok($tokenizer.end_token()?),
           '(' => Ok($tokenizer.end_token_push(Token::LeftParen)?),
           ')' => Ok($tokenizer.end_token_push(Token::RightParen)?),
           $($matcher $(if $pred)* => $result),*
       }
   }
}

#[derive(PartialEq, Debug)]
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

#[derive(Debug)]
pub struct Lexer {
    state: State,
    tokens: Vec<Token>,
    buffer: String,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    String(String),
    Int(i32),
    Float(f32),
    Symbol(String),
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            state: State::Init,
            tokens: vec![],
            buffer: "".to_owned(),
        }
    }

    fn trans_ignore(&mut self, state: State) {
        self.state = state;
    }

    fn trans(&mut self, c: char, state: State) {
        self.state = state;
        self.buffer.push(c);
    }

    fn push_buffer(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn end_token(&mut self) -> Result<(), Error> {
        static EXPECT_NUM: &'static str = "Programming error in tokenizer";
        let b = mem::replace(&mut self.buffer, "".to_owned());
        let token = match &self.state {
            State::Minus => Ok(Token::Symbol(b)),
            State::Num => Ok(Token::Int(b.parse().expect(EXPECT_NUM))),
            State::Float => Ok(Token::Float(b.parse().expect(EXPECT_NUM))),
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

    fn try_end_token(&mut self) -> Result<(), Error> {
        if !self.buffer.is_empty() {
            self.end_token()
        } else {
            Ok(())
        }
    }

    fn end_token_push(&mut self, t: Token) -> Result<(), Error> {
        self.end_token()?;
        self.push_token(t);
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
            '"' => Ok(self.trans_ignore(State::StringStart)),
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
            '"' => Ok(self.trans_ignore(State::StringClose)),
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

    pub fn tokenize(mut self, input: &str) -> Result<Vec<Token>, Error> {
        for c in input.chars() {
            self.process_char(c)?;
        }
        self.try_end_token()?;
        Ok(self.tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize_int() {
        let t = Lexer::new();
        let tokens = t.tokenize("1337 420 -322").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Int(1337),
                Token::Int(420),
                Token::Int(-322)
            ]
        );

        let t_bad = Lexer::new();
        let err = t_bad.tokenize("1337s").err().unwrap();
        assert_eq!(
            err,
            Error::TransitionError("Unexpected character: s, buffer: 1337, state: Num".to_owned())
        );
    }

    #[test]
    fn tokenize_float() {
        let t = Lexer::new();
        let tokens = t.tokenize("1337.44 420.33 -322.0").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Float(1337.44),
                Token::Float(420.33),
                Token::Float(-322.0)
            ]
        );

        let t_bad = Lexer::new();
        let err = t_bad.tokenize("1337.").err().unwrap();
        assert_eq!(
            err,
            Error::TokenTerminationError("Non terminating state: Dot".to_owned())
        );

        let t_bad2 = Lexer::new();
        let err = t_bad2.tokenize("1337.a").err().unwrap();
        assert_eq!(
            err,
            Error::TransitionError("Unexpected character: a, buffer: 1337., state: Dot".to_owned())
        );
    }

    #[test]
    fn tokenize_symbol() {
        let t = Lexer::new();
        let tokens = t.tokenize("true false - ----").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("true".to_owned()),
                Token::Symbol("false".to_owned()),
                Token::Symbol("-".to_owned()),
                Token::Symbol("----".to_owned()),
            ]
        );

        let t_bad = Lexer::new();
        let err = t_bad.tokenize("lol\"kek").err().unwrap();
        assert_eq!(
            err,
            Error::TransitionError(
                "Unexpected character: \", buffer: lol, state: Symbol".to_owned()
            )
        );
    }

    #[test]
    fn tokenize_string() {
        let t = Lexer::new();
        let tokens = t
            .tokenize("\"\" \" \" \"yolo swag()() hihihe \\\" win\" \"nailed it\"")
            .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::String("".to_owned()),
                Token::String(" ".to_owned()),
                Token::String("yolo swag()() hihihe \\\" win".to_owned()),
                Token::String("nailed it".to_owned())
            ]
        );

        let t_bad = Lexer::new();
        let err = t_bad.tokenize("\"lol\"kek\"").err().unwrap();
        assert_eq!(
            err,
            Error::TransitionError(
                "Unexpected character: k, buffer: lol, state: StringClose".to_owned()
            )
        );
    }

    #[test]
    fn tokenize_sexp() {
        let t = Lexer::new();
        let tokens = t
            .tokenize("(println \"hey lisp\" (* (+ 1 2.02 3) 420))")
            .unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("println".to_owned()),
                Token::String("hey lisp".to_owned()),
                Token::LeftParen,
                Token::Symbol("*".to_owned()),
                Token::LeftParen,
                Token::Symbol("+".to_owned()),
                Token::Int(1),
                Token::Float(2.02),
                Token::Int(3),
                Token::RightParen,
                Token::Int(420),
                Token::RightParen,
                Token::RightParen,
            ]
        );
    }
}
