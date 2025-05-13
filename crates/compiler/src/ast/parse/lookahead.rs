use crate::ast::Expr;
use crate::ast::Pat;
use crate::ast::QueryOp;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::parser::lookahead::Lookahead;

use super::token::Token;

impl Lookahead for Expr {
    type Token = Token;
    const FIRST: Token = Token::Int
        .or(Token::IntSuffix)
        .or(Token::Float)
        .or(Token::FloatSuffix)
        .or(Token::String)
        .or(Token::Name)
        .or(Token::LParen)
        .or(Token::Minus)
        .or(Token::Break)
        .or(Token::Continue)
        .or(Token::Return)
        .or(Token::LBrack)
        .or(Token::If)
        .or(Token::Match)
        .or(Token::While)
        .or(Token::True)
        .or(Token::False)
        .or(Token::For)
        .or(Token::Not)
        .or(Token::Char)
        .or(Token::LBrace)
        .or(Token::Record)
        .or(Token::From)
        .or(Token::Underscore)
        .or(Token::Star)
        .or(Token::Ampersand)
        .or(Token::Loop);
    const FOLLOW: Token = Token::Eof
        .or(Token::And)
        .or(Token::DotDot)
        .or(Token::Dot)
        .or(Token::Eq)
        .or(Token::EqEq)
        .or(Token::Ge)
        .or(Token::Gt)
        .or(Token::Le)
        .or(Token::Lt)
        .or(Token::Minus)
        .or(Token::NotEq)
        .or(Token::Or)
        .or(Token::Plus)
        .or(Token::Slash)
        .or(Token::Star)
        .or(Token::LParen)
        .or(Token::Colon)
        .or(Token::SemiColon)
        .or(Token::FatArrow);
}

impl Lookahead for Stmt {
    type Token = Token;
    const FIRST: Token = Token::Def
        .or(Token::Type)
        .or(Token::Trait)
        .or(Token::Struct)
        .or(Token::Enum)
        .or(Token::Impl)
        .or(Token::Val)
        .or(Token::Var)
        .or(Expr::FIRST);
    const FOLLOW: Token = Token::Eof;
}

impl Lookahead for Type {
    type Token = Token;
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Struct)
        .or(Token::Record)
        .or(Token::LBrack)
        .or(Token::Underscore)
        .or(Token::Not)
        .or(Token::Ampersand);
    const FOLLOW: Token = Token::Eof.or(Token::FatArrow).or(Token::Tilde);
}

impl Lookahead for Pat {
    type Token = Token;
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Underscore)
        .or(Token::Int)
        .or(Token::String)
        .or(Token::Struct)
        .or(Token::Record)
        .or(Token::True)
        .or(Token::False)
        .or(Token::Char);
    const FOLLOW: Token = Token::Eof
        .or(Token::Or)
        .or(Token::Colon)
        .or(Token::FatArrow);
}

impl Lookahead for QueryOp {
    type Token = Token;
    const FIRST: Token = Token::From
        .or(Token::Where)
        .or(Token::Over)
        .or(Token::Group)
        .or(Token::Var)
        .or(Token::Select)
        .or(Token::Join)
        .or(Token::Limit)
        .or(Token::Distinct)
        .or(Token::Limit)
        .or(Token::Skip)
        .or(Token::Compute)
        .or(Token::Order);
    const FOLLOW: Token = Expr::FOLLOW.or(QueryOp::FIRST).or(Token::Into);
}
