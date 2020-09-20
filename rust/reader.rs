use super::Error;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::reader_macros;
use std::fmt;
use std::mem;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AstLeaf {
    Symbol(String),
    Int(i32),
    Float(f32),
    String(String),
    Function(LFunction),
}

#[derive(Clone)]
pub(crate) struct LFunction {
    f: Rc<dyn Fn(Vec<Ast>) -> Result<Ast, Error>>,
}

impl PartialEq for LFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.f, &other.f)
    }
}

impl fmt::Debug for LFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LFunction")
            .field("f", &Rc::as_ptr(&self.f))
            .finish()
    }
}

impl LFunction {
    pub(crate) fn new(f: impl Fn(Vec<Ast>) -> Result<Ast, Error> + 'static) -> Self {
        LFunction { f: Rc::new(f) }
    }
}

// impl PartialEq for AstLeaf {
//     fn eq(&self, other: &Self) -> bool {
//	match (self, other) {
//	    (AstLeaf::Symbol(a), AstLeaf::Symbol(b)) => a == b,
//	    (AstLeaf::Int(a), AstLeaf::Int(b)) => a == b,
//	    (AstLeaf::Float(a), AstLeaf::Float(b)) => a == b,
//	    (AstLeaf::String(a), AstLeaf::String(b)) => a == b,
//	    (AstLeaf::Function(a), AstLeaf::Function(b)) => Rc::ptr_eq(a, b),

//	}
//     }
// }

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum ListType {
    Parens,
    Brackets,
    Braces,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct AstList {
    pub(crate) list_type: ListType,
    pub(crate) list: Vec<Ast>,
}

#[derive(Debug, PartialEq, Clone)]
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
    pub(crate) fn function(f: impl Fn(Vec<Ast>) -> Result<Ast, Error> + 'static) -> Self {
        Ast::Leaf(AstLeaf::Function(LFunction::new(f)))
    }
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
    pub(crate) fn get_function(&self) -> Result<Rc<dyn Fn(Vec<Ast>) -> Result<Ast, Error>>, Error> {
        if let Ast::Leaf(l) = self {
            if let AstLeaf::Function(lf) = l {
                return Ok(lf.f.clone());
            }
        }
        Err(Error::EvalError("not a function".to_owned()))
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
                let mut child_list = mem::replace(&mut current_list, parent_list);
                let list_type = stack_parens.pop().unwrap();
                if !(does_terminate(l, list_type)) {
                    return Err(Error::Unbalanced);
                }
                reader_macros::apply(&mut child_list);
                current_list.push(Ast::List(AstList {
                    list_type: list_type,
                    list: child_list,
                }));
            }
        }
    }
    if stack_parens.is_empty() {
        reader_macros::apply(&mut current_list);
        Ok(current_list)
    } else {
        Err(Error::Unbalanced)
    }
}

pub(crate) fn read(s: String) -> Result<Ast, Error> {
    let lex = Lexer::new();
    let tokens = lex.tokenize(&s).map_err(|_| Error::EOF)?;
    let mut ast_top: Vec<Ast> = parse(tokens)?;
    if ast_top.is_empty() {
        Ok(Default::default())
    } else {
        Ok(ast_top.pop().unwrap())
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
