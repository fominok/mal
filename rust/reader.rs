use crate::parser::{parse, Ast};
use crate::lexer::Lexer;
use super::Error;

pub(crate) fn read(s: String) -> Result<Ast, Error> {
    let lex = Lexer::new();
    let tokens = lex.tokenize(&s).map_err(|_| Error::EOF)?;
    parse(tokens)
}
