use macros::enumset;

use crate::report::spanned::Spanned;

#[enumset]
pub enum Token {
    // Punctuations
    Ampersand,
    Bar,
    Colon,
    ColonColon,
    Comma,
    Dot,
    DotDot,
    Eq,
    EqEq,
    FatArrow,
    Ge,
    Gt,
    LBrace,
    LBrack,
    LParen,
    Label,
    Le,
    Lt,
    Minus,
    Not,
    NotEq,
    Plus,
    Question,
    RBrace,
    RBrack,
    RParen,
    SemiColon,
    Slash,
    Star,
    Tilde,
    Underscore,
    // Keywords
    And,
    As,
    Break,
    Compute,
    Continue,
    Def,
    Distinct,
    Drop,
    Else,
    Enum,
    False,
    For,
    From,
    Order,
    Cross,
    Group,
    If,
    Impl,
    In,
    Into,
    Join,
    Limit,
    Skip,
    Loop,
    Match,
    Mut,
    Of,
    On,
    Or,
    Over,
    Record,
    Return,
    Select,
    Struct,
    Trait,
    True,
    Type,
    Union,
    Val,
    Var,
    Where,
    While,
    With,
    // Literals
    Char,
    Code,
    Float,
    FloatSuffix,
    Int,
    IntSuffix,
    Name,
    String,
    // Special
    Eof,
    Err,
}

#[test]
fn test_token() {
    let t = Token::Bar.or(Token::Colon).or(Token::ColonColon);
    let mut iter = t.into_iter();
    assert_eq!(iter.next(), Some(Token::Bar));
    assert_eq!(iter.next(), Some(Token::Colon));
    assert_eq!(iter.next(), Some(Token::ColonColon));
}

impl IntoIterator for Token {
    type Item = Token;

    type IntoIter = TokenIterator;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator { bits: self.bits() }
    }
}

pub struct TokenIterator {
    bits: u128,
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Token::from(bit))
        }
    }
}

impl Token {
    pub fn expected(self) -> String {
        let mut vec = Vec::new();
        for token in self {
            if vec.len() > 5 {
                break;
            }
            vec.push(format!("`{}`", token.as_str()));
        }
        if vec.len() == 1 {
            format!("Expected {}", vec.pop().unwrap())
        } else if vec.len() > 5 {
            format!("Expected one of {}, ...", vec.join(", "))
        } else {
            format!("Expected one of {}", vec.join(", "))
        }
    }
}

impl Token {
    fn as_str(self) -> &'static str {
        match self {
            Token::Ampersand => "&",
            Token::Eq => "=",
            Token::EqEq => "==",
            Token::Not => "!",
            Token::NotEq => "!=",
            Token::Lt => "<",
            Token::Le => "<=",
            Token::Gt => ">",
            Token::Ge => ">=",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Dot => ".",
            Token::DotDot => "..",
            Token::Colon => ":",
            Token::ColonColon => "::",
            Token::SemiColon => ";",
            Token::Comma => ",",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBrace => "{",
            Token::RBrace => "}",
            Token::LBrack => "[",
            Token::RBrack => "]",
            Token::Underscore => "_",
            Token::Question => "?",
            Token::FatArrow => "=>",
            Token::Bar => "|",
            // Keywords
            Token::And => "and",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Def => "def",
            Token::Distinct => "distinct",
            Token::Else => "else",
            Token::Enum => "enum",
            Token::False => "false",
            Token::For => "for",
            Token::Loop => "loop",
            Token::From => "from",
            Token::Cross => "cross",
            Token::Group => "group",
            Token::If => "if",
            Token::Impl => "impl",
            Token::In => "in",
            Token::Into => "into",
            Token::Join => "join",
            Token::Match => "match",
            Token::Mut => "mut",
            Token::On => "on",
            Token::Or => "or",
            Token::Over => "over",
            Token::Return => "return",
            Token::Record => "record",
            Token::Select => "select",
            Token::Struct => "struct",
            Token::True => "true",
            Token::Type => "type",
            Token::Val => "val",
            Token::Var => "var",
            Token::Where => "where",
            Token::While => "while",
            Token::With => "with",
            Token::Union => "union",
            Token::Limit => "limit",
            Token::Skip => "skip",
            Token::Of => "of",
            Token::As => "as",
            Token::Compute => "compute",
            Token::Trait => "trait",
            // Literals
            Token::Label => "<label>",
            Token::Code => "<code>",
            Token::Name => "<name>",
            Token::Int => "<int>",
            Token::IntSuffix => "<int-suffix>",
            Token::Float => "<float>",
            Token::FloatSuffix => "<float-suffix>",
            Token::String => "<string>",
            Token::Char => "<char>",
            Token::Err => "<err>",
            Token::Eof => "<eof>",
            _ => "<unknown>",
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Spanned<Token> {
    // Extracts the text of a token from the input string
    pub fn text(self, input: &str) -> &str {
        let start = self.s.start().expect("Should not be generated") as usize;
        let end = self.s.end().expect("Should not be generated") as usize;
        match self.v {
            Token::Code => &input[start + 3..end - 3],
            Token::String => &input[start + 1..end - 1],
            Token::Char => &input[start + 1..end - 1],
            Token::Label => &input[start + 1..end],
            _ => &input[start..end],
        }
    }
}

impl Token {
    pub fn opens(self, other: Token) -> bool {
        match (self, other) {
            (Token::LParen, Token::RParen) => true,
            (Token::LBrace, Token::RBrace) => true,
            (Token::LBrack, Token::RBrack) => true,
            _ => false,
        }
    }
}
