use crate::reader::{Ast, AstLeaf, ListType};
use std::fmt;

impl fmt::Display for AstLeaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstLeaf::Symbol(x) => write!(f, "{}", x),
            AstLeaf::String(x) => write!(f, "\"{}\"", x),
            AstLeaf::Int(x) => write!(f, "{}", x.to_string()),
            AstLeaf::Float(x) => write!(f, "{}", x.to_string()),
            AstLeaf::Function(lf) => write!(f, "{:?}", lf),
        }
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Leaf(x) => write!(f, "{}", x),
            Ast::List(xs) => {
                let (lp, rp) = match xs.list_type {
                    ListType::Parens => ('(', ')'),
                    ListType::Braces => ('{', '}'),
                    ListType::Brackets => ('[', ']'),
                };
                let lex = &xs.list;
                if lex.is_empty() {
                    write!(f, "{}{}", lp, rp)
                } else {
                    let mut iter = lex.iter();
                    write!(f, "{}{}", lp, iter.next().expect("What could go wrong"))?;
                    for i in iter {
                        write!(f, " {}", i)?;
                    }
                    write!(f, "{}", rp)
                }
            }
        }
    }
}
