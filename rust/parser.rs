use super::Error;
use crate::lexer::Token;
use std::mem;

#[derive(Debug, PartialEq)]
pub(crate) enum AstLeaf {
    Symbol(String),
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum ListType {
    Parens,
    Brackets,
    Braces,
}

#[derive(Debug, PartialEq)]
pub(crate) struct AstList {
    pub(crate) list_type: ListType,
    pub(crate) list: Vec<Ast>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Ast {
    List(AstList),
    Leaf(AstLeaf),
}

impl Default for Ast {
    fn default() -> Self {
        Ast::Leaf(AstLeaf::Symbol("".to_owned()))
    }
}

impl Ast {
    pub(crate) fn symbol(s: String) -> Self {
        Ast::Leaf(AstLeaf::Symbol(s))
    }
    pub(crate) fn int(i: i32) -> Self {
        Ast::Leaf(AstLeaf::Int(i))
    }
    pub(crate) fn float(f: f32) -> Self {
        Ast::Leaf(AstLeaf::Float(f))
    }
    pub(crate) fn string(s: String) -> Self {
        Ast::Leaf(AstLeaf::String(s))
    }
    pub(crate) fn parens(sib: Vec<Self>) -> Self {
        Ast::List(AstList {
            list_type: ListType::Parens,
            list: sib,
        })
    }
    pub(crate) fn braces(sib: Vec<Self>) -> Self {
        Ast::List(AstList {
            list_type: ListType::Braces,
            list: sib,
        })
    }
    pub(crate) fn brackets(sib: Vec<Self>) -> Self {
        Ast::List(AstList {
            list_type: ListType::Brackets,
            list: sib,
        })
    }
}

fn get_list_type(lex: Token) -> Option<ListType> {
    use ListType::*;
    use Token::*;
    match lex {
        LeftParen => Some(Parens),
        LeftBrace => Some(Braces),
        LeftBracket => Some(Brackets),
        _ => None,
    }
}

fn does_terminate(lex: Token, list_type: ListType) -> bool {
    use ListType::*;
    use Token::*;
    match lex {
        RightParen => list_type == Parens,
        RightBrace => list_type == Braces,
        RightBracket => list_type == Brackets,
        _ => false,
    }
}

pub(crate) fn parse(lexemes: Vec<Token>) -> Result<Vec<Ast>, Error> {
    let mut stack_parens: Vec<ListType> = Vec::new();
    let mut stack_lists: Vec<Vec<Ast>> = Vec::new();
    let mut current_list: Vec<Ast> = Vec::new();
    for l in lexemes.into_iter() {
        match l {
            Token::String(x) => current_list.push(Ast::string(x)),
            Token::Int(x) => current_list.push(Ast::int(x)),
            Token::Float(x) => current_list.push(Ast::float(x)),
            Token::Symbol(x) => current_list.push(Ast::symbol(x)),
            Token::LeftParen | Token::LeftBrace | Token::LeftBracket => {
                stack_parens.push(get_list_type(l).expect("Trust me"));
                stack_lists.push(mem::replace(&mut current_list, Vec::new()));
            }
            Token::RightParen | Token::RightBrace | Token::RightBracket => {
                let parent_list = stack_lists.pop().unwrap();
                let child_list = mem::replace(&mut current_list, parent_list);
                let list_type = stack_parens.pop().unwrap();
                if !(does_terminate(l, list_type)) {
                    return Err(Error::Unbalanced);
                }
                current_list.push(Ast::List(AstList {
                    list_type: list_type,
                    list: child_list,
                }));
            }
        }
    }
    if stack_parens.is_empty() {
        Ok(
            current_list, // .pop()
                          // .unwrap_or(Ast::Leaf(AstLeaf::Symbol("".to_owned())))
        )
    } else {
        Err(Error::Unbalanced)
    }
}
