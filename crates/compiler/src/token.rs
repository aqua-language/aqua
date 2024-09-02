use crate::spanned::Spanned;

#[bitmask_enum::bitmask(u128)]
#[bitmask_config(vec_debug)]
pub enum Token {
    // Punctuations
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
    Underscore,
    // Keywords
    And,
    As,
    Break,
    Compute,
    Continue,
    Def,
    Desc,
    Else,
    Enum,
    False,
    For,
    From,
    Fun,
    Group,
    If,
    Impl,
    In,
    Into,
    Join,
    Match,
    Let,
    Of,
    On,
    Or,
    Over,
    Return,
    Struct,
    Select,
    Record,
    Trait,
    True,
    Type,
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
    let t = Token::Bar | Token::Colon | Token::ColonColon;
    let mut iter = t.into_iter();
    assert_eq!(iter.next(), Some(Token::Bar));
    assert_eq!(iter.next(), Some(Token::Colon));
    assert_eq!(iter.next(), Some(Token::ColonColon));
}

impl IntoIterator for Token {
    type Item = Token;

    type IntoIter = TokenIterator;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator { n: self.bits() }
    }
}

pub struct TokenIterator {
    n: u128,
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            None
        } else {
            let trailing_zeros = self.n.trailing_zeros();
            let bit = 1 << trailing_zeros;
            self.n &= !bit;
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
            Token::Desc => "desc",
            Token::Else => "else",
            Token::Enum => "enum",
            Token::False => "false",
            Token::For => "for",
            Token::From => "from",
            Token::Fun => "fun",
            Token::Group => "group",
            Token::If => "if",
            Token::Impl => "impl",
            Token::In => "in",
            Token::Into => "into",
            Token::Join => "join",
            Token::Match => "match",
            Token::On => "on",
            Token::Or => "or",
            Token::Over => "over",
            Token::Return => "return",
            Token::Record => "record",
            Token::Select => "select",
            Token::Struct => "struct",
            Token::Let => "let",
            Token::True => "true",
            Token::Type => "type",
            Token::Var => "var",
            Token::Where => "where",
            Token::While => "while",
            Token::With => "with",
            // Literals
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
            Token::Of => "of",
            Token::As => "as",
            Token::Compute => "compute",
            Token::Trait => "trait",
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
    pub fn text(self, input: &str) -> &str {
        let span = self.s;
        match self.v {
            Token::Code => &input[(span.start() + 3) as usize..(span.end() - 3) as usize],
            Token::String => &input[(span.start() + 1) as usize..(span.end() - 1) as usize],
            Token::Char => &input[(span.start() + 1) as usize..(span.end() - 1) as usize],
            _ => &input[*span.start() as usize..*span.end() as usize],
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
