use super::Error;
use crate::lexer::Lexer;
use crate::parser::{parse, Ast};

pub(crate) fn read(s: String) -> Result<Ast, Error> {
    let lex = Lexer::new();
    let tokens = lex.tokenize(&s).map_err(|_| Error::EOF)?;
    parse(tokens)
}
