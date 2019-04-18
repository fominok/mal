use super::Error;
use crate::lexer::Lexer;
use crate::parser::{parse, Ast, AstLeaf, AstList, ListType};
use std::mem;

pub(crate) trait ReaderMacro {
    const WINDOW: usize;
    fn process_ast_slice(ast: &mut [Ast]) -> bool;

    fn process_ast(ast: &mut Vec<Ast>) {
        let mut i = 0;

        while ast.len() >= Self::WINDOW && i <= ast.len() - Self::WINDOW {
            let changed = Self::process_ast_slice(&mut ast[i..i + Self::WINDOW]);
            if changed {
                for _ in 0..Self::WINDOW - 1 {
                    ast.remove(i + 1);
                }
                i = 0;
            } else {
                i += 1;
            }
        }
    }
}

pub(crate) struct WithMeta;
pub(crate) struct Quote;
pub(crate) struct QuasiQuote;
pub(crate) struct Deref;
pub(crate) struct Unquote;
pub(crate) struct SpliceUnquote;

impl ReaderMacro for WithMeta {
    const WINDOW: usize = 3;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        let meta_symbol = &ast[0];
        let meta_info = &ast[1];

        match (meta_symbol, meta_info) {
            (
                Ast::Leaf(AstLeaf::Symbol(ref meta_char)),
                Ast::List(AstList {
                    list_type: ListType::Braces,
                    list: _,
                }),
            ) if meta_char == "^" => {
                let replace = Ast::List(AstList {
                    list_type: ListType::Parens,
                    list: vec![
                        Ast::symbol("with-meta".to_owned()),
                        mem::replace(&mut ast[2], Default::default()),
                        mem::replace(&mut ast[1], Default::default()),
                    ],
                });
                mem::replace(&mut ast[0], replace);
                true
            }
            _ => false,
        }
    }
}

fn simple_sub(ast: &mut [Ast], matcher: &str, replacement: &str) -> bool {
    let reader_symbol = &ast[0];

    match reader_symbol {
        Ast::Leaf(AstLeaf::Symbol(ref reader_str)) if reader_str == matcher => {
            let replace = Ast::List(AstList {
                list_type: ListType::Parens,
                list: vec![
                    Ast::symbol(replacement.to_owned()),
                    mem::replace(&mut ast[1], Default::default()),
                ],
            });
            mem::replace(&mut ast[0], replace);
            true
        }
        _ => false,
    }
}

impl ReaderMacro for Quote {
    const WINDOW: usize = 2;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        simple_sub(ast, "'", "quote")
    }
}

impl ReaderMacro for QuasiQuote {
    const WINDOW: usize = 2;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        simple_sub(ast, "`", "quasiquote")
    }
}

impl ReaderMacro for Deref {
    const WINDOW: usize = 2;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        simple_sub(ast, "@", "deref")
    }
}

impl ReaderMacro for Unquote {
    const WINDOW: usize = 2;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        simple_sub(ast, "~", "unquote")
    }
}
impl ReaderMacro for SpliceUnquote {
    const WINDOW: usize = 2;

    fn process_ast_slice(ast: &mut [Ast]) -> bool {
        simple_sub(ast, "~@", "splice-unquote")
    }
}

fn reader_macros(mut ast_top: Vec<Ast>) -> Result<Ast, Error> {
    Ok(ast_top.pop().unwrap())
}

pub(crate) fn read(s: String) -> Result<Ast, Error> {
    let lex = Lexer::new();
    let tokens = lex.tokenize(&s).map_err(|_| Error::EOF)?;
    let ast_top: Vec<Ast> = parse(tokens)?;
    if ast_top.is_empty() {
        Ok(Default::default())
    } else {
        reader_macros(ast_top)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn macro_with_meta() {
        let lex = Lexer::new();
        let tokens = lex
            .tokenize("^{yolo swag} (+ '1 3 ^{300 bucks} [420  ^{top kek} (+ 1 322)] 3 7) ")
            .unwrap();
        let mut ast_top: Vec<Ast> = parse(tokens).unwrap();
        WithMeta::process_ast(&mut ast_top);
        assert_eq!(
            ast_top,
            vec![Ast::parens(vec![
                Ast::symbol("with-meta".to_owned()),
                Ast::parens(vec![
                    Ast::symbol("+".to_owned()),
                    Ast::parens(vec![Ast::symbol("quote".to_owned()), Ast::int(1),]),
                    Ast::int(3),
                    Ast::parens(vec![
                        Ast::symbol("with-meta".to_owned()),
                        Ast::brackets(vec![
                            Ast::int(420),
                            Ast::parens(vec![
                                Ast::symbol("with-meta".to_owned()),
                                Ast::parens(vec![
                                    Ast::symbol("+".to_owned()),
                                    Ast::int(1),
                                    Ast::int(322)
                                ]),
                                Ast::braces(vec![
                                    Ast::symbol("top".to_owned()),
                                    Ast::symbol("kek".to_owned())
                                ]),
                            ]),
                        ]),
                        Ast::braces(vec![Ast::int(300), Ast::symbol("bucks".to_owned())]),
                    ]),
                    Ast::int(3),
                    Ast::int(7),
                ]),
                Ast::braces(vec![
                    Ast::symbol("yolo".to_owned()),
                    Ast::symbol("swag".to_owned())
                ]),
            ])]
        );
    }
}
