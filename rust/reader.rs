use super::Error;
use crate::lexer::Lexer;
use crate::parser::{parse, Ast, AstLeaf, AstList, ListType};
use std::mem;

pub(crate) trait ReaderMacro {
    fn process_ast(ast: &mut Vec<Ast>);
}

pub(crate) struct WithMeta;

impl ReaderMacro for WithMeta {
    fn process_ast(ast: &mut Vec<Ast>) {
        let mut i = 0;

        while ast.len() > 2 && i < ast.len() - 2 {
            let meta_symbol = &ast[i];
            let meta_info = &ast[i + 1];

            match (meta_symbol, meta_info) {
                (
                    Ast::Leaf(AstLeaf::Symbol(ref meta_char)),
                    Ast::List(AstList {
                        list_type: ListType::Braces,
                        list: _,
                    }),
                ) if meta_char == "^" => (),
                _ => {
                    i += 1;
                    continue;
                }
            }

            let replace = Ast::List(AstList {
                list_type: ListType::Parens,
                list: vec![
                    Ast::symbol("with-meta".to_owned()),
                    mem::replace(&mut ast[i + 2], Default::default()),
                    mem::replace(&mut ast[i + 1], Default::default()),
                ],
            });

            mem::replace(&mut ast[i], replace);
            ast.remove(i + 1);
            ast.remove(i + 1);
            i = 0;
        }
    }
}

fn reader_macros(mut ast_top: Vec<Ast>) -> Result<Ast, Error> {
    Ok(ast_top.pop().unwrap())
}

pub(crate) fn read(s: String) -> Result<Ast, Error> {
    let lex = Lexer::new();
    let tokens = lex.tokenize(&s).map_err(|_| Error::EOF)?;
    println!("{:?}", tokens);
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
            .tokenize("^{yolo swag} (+ 1 3 ^{300 bucks} [420  ^{top kek} (+ 1 322)] 3 7) ")
            .unwrap();
        let mut ast_top: Vec<Ast> = parse(tokens).unwrap();
        WithMeta::process_ast(&mut ast_top);
        assert_eq!(
            ast_top,
            vec![Ast::parens(vec![
                Ast::symbol("with-meta".to_owned()),
                Ast::parens(vec![
                    Ast::symbol("+".to_owned()),
                    Ast::int(1),
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
