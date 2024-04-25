use crate::diag::Report;

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
    Deploy,
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
    Of,
    On,
    Or,
    Order,
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

impl Token {
    pub fn expected(self) -> String {
        let mut vec = Vec::new();
        for v in [
            // Punctuations
            Token::Bar,
            Token::Colon,
            Token::ColonColon,
            Token::Comma,
            Token::Dot,
            Token::DotDot,
            Token::Eq,
            Token::EqEq,
            Token::FatArrow,
            Token::Ge,
            Token::Gt,
            Token::LBrace,
            Token::LBrack,
            Token::LParen,
            Token::Le,
            Token::Lt,
            Token::Minus,
            Token::Not,
            Token::NotEq,
            Token::Plus,
            Token::Question,
            Token::RBrace,
            Token::RBrack,
            Token::RParen,
            Token::SemiColon,
            Token::Slash,
            Token::Star,
            Token::Underscore,
            // Keywords
            Token::And,
            Token::As,
            Token::Break,
            Token::Compute,
            Token::Continue,
            Token::Def,
            Token::Deploy,
            Token::Desc,
            Token::Else,
            Token::Enum,
            Token::False,
            Token::For,
            Token::From,
            Token::Fun,
            Token::Group,
            Token::If,
            Token::Impl,
            Token::In,
            Token::Into,
            Token::Join,
            Token::Match,
            Token::Of,
            Token::On,
            Token::Or,
            Token::Order,
            Token::Over,
            Token::Return,
            Token::Select,
            Token::Struct,
            Token::Trait,
            Token::True,
            Token::Type,
            Token::Var,
            Token::Where,
            Token::While,
            Token::With,
            // Literals
            Token::Char,
            Token::Code,
            Token::Float,
            Token::FloatSuffix,
            Token::Int,
            Token::IntSuffix,
            Token::Name,
            Token::String,
            // Special
            Token::Eof,
            Token::Err,
        ] {
            if self.contains(v) {
                if vec.len() > 5 {
                    break;
                }
                vec.push(format!("`{}`", v.as_str()));
            }
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
            Token::Deploy => "deploy",
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
            Token::Order => "order",
            Token::Over => "over",
            Token::Return => "return",
            Token::Record => "record",
            Token::Select => "select",
            Token::Struct => "struct",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub s: Span,
    pub v: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, data: T) -> Spanned<T> {
        Spanned { s: span, v: data }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            s: self.s,
            v: f(self.v),
        }
    }
}

impl<T> Spanned<Option<Vec<T>>> {
    pub fn flatten(self) -> Spanned<Vec<T>> {
        self.map(|v| v.unwrap_or_default())
    }
}

impl<T> Spanned<Option<T>> {
    pub fn transpose(self) -> Option<Spanned<T>> {
        match self.v {
            Some(value) => Some(Spanned {
                s: self.s,
                v: value,
            }),
            None => None,
        }
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

#[derive(Clone, Copy, Default)]
pub enum Span {
    Source(u16, u32, u32),
    #[default]
    Generated,
}

impl Ord for Span {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Span {}
impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Span::Source(file, start, end) => write!(f, "{file}:{start}..{end}"),
            Span::Generated => write!(f, "..."),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Span::Source(file, start, end) => write!(f, "{:?}:{}-{}", file, start, end),
            Span::Generated => write!(f, "<builtin>"),
        }
    }
}

impl Span {
    pub fn new(file: u16, range: std::ops::Range<u32>) -> Span {
        Span::Source(file, range.start, range.end)
    }

    pub fn file(&self) -> &u16 {
        match self {
            Span::Source(file, _, _) => file,
            Span::Generated => &0,
        }
    }

    pub fn start(&self) -> &u32 {
        match self {
            Span::Source(_, start, _) => start,
            Span::Generated => &0,
        }
    }

    pub fn end(&self) -> &u32 {
        match self {
            Span::Source(_, _, end) => end,
            Span::Generated => &0,
        }
    }
}

impl std::ops::Add<Span> for Span {
    type Output = Span;

    fn add(self, other: Span) -> Self::Output {
        match (self, other) {
            (Span::Generated, Span::Generated) => Span::Generated,
            (Span::Generated, Span::Source(file, start, end)) => Span::new(file, start..end),
            (Span::Source(file, start, end), Span::Generated) => Span::new(file, start..end),
            (Span::Source(file, start, _), Span::Source(_, _, end)) => Span::new(file, start..end),
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    eof: bool,
    pub file: u16,
    pub report: Report,
}

impl<'a> Lexer<'a> {
    pub fn new(file: u16, input: &'a str) -> Lexer<'a> {
        Lexer {
            file,
            input,
            eof: false,
            pos: 0,
            report: Report::new(),
        }
    }

    fn unexpected_char(&mut self, c: char) {
        self.report.err(
            Span::new(self.file, (self.pos - 1) as u32..self.pos as u32),
            "Unexpected character",
            format!("Unexpected character '{c}'"),
        );
    }

    pub fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    pub fn lex(&mut self) -> Option<Spanned<Token>> {
        loop {
            let start = self.pos;
            let mut chars = self.input[self.pos..].chars();
            let c = chars.next()?;
            self.pos += c.len_utf8();

            let token = match c {
                '#' => {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            break;
                        }
                        self.pos += c.len_utf8();
                    }
                    continue;
                }
                ' ' | '\n' | '\t' => continue,
                'a'..='z' | 'A'..='Z' | '_' => {
                    while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = chars.next() {
                        self.pos += 1;
                    }
                    if self.pos - start == 1 && c == '_' {
                        Token::Underscore
                    } else {
                        match &self.input[start..self.pos] {
                            "and" => Token::And,
                            "as" => Token::As,
                            "break" => Token::Break,
                            "compute" => Token::Compute,
                            "continue" => Token::Continue,
                            "def" => Token::Def,
                            "deploy" => Token::Deploy,
                            "else" => Token::Else,
                            "enum" => Token::Enum,
                            "false" => Token::False,
                            "for" => Token::For,
                            "from" => Token::From,
                            "fun" => Token::Fun,
                            "group" => Token::Group,
                            "if" => Token::If,
                            "impl" => Token::Impl,
                            "in" => Token::In,
                            "into" => Token::Into,
                            "join" => Token::Join,
                            "match" => Token::Match,
                            "of" => Token::Of,
                            "on" => Token::On,
                            "or" => Token::Or,
                            "order" => Token::Order,
                            "over" => Token::Over,
                            "return" => Token::Return,
                            "select" => Token::Select,
                            "struct" => Token::Struct,
                            "record" => Token::Struct,
                            "trait" => Token::Trait,
                            "true" => Token::True,
                            "type" => Token::Type,
                            "var" => Token::Var,
                            "where" => Token::Where,
                            "while" => Token::While,
                            "with" => Token::With,
                            _ => Token::Name,
                        }
                    }
                }
                '0'..='9' => loop {
                    match chars.next() {
                        Some('0'..='9') => self.pos += 1,
                        Some('.') => match chars.next() {
                            Some('a'..='z' | 'A'..='Z' | '_') => {
                                break Token::Int;
                            }
                            Some('0'..='9') => {
                                self.pos += 1;
                                self.pos += 1;
                                let mut c = chars.next();
                                while let Some('0'..='9') = c {
                                    self.pos += 1;
                                    c = chars.next();
                                }
                                if let Some('a'..='z' | 'A'..='Z' | '_') = c {
                                    self.pos += 1;
                                    while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') =
                                        chars.next()
                                    {
                                        self.pos += 1;
                                    }
                                    break Token::FloatSuffix;
                                }
                                break Token::Float;
                            }
                            _ => {
                                self.pos += 1;
                                break Token::Float;
                            }
                        },
                        Some(c) => {
                            if let 'a'..='z' | 'A'..='Z' | '_' = c {
                                self.pos += 1;
                                while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') =
                                    chars.next()
                                {
                                    self.pos += 1;
                                }
                                break Token::IntSuffix;
                            }
                            break Token::Int;
                        }
                        None => {
                            break Token::Int;
                        }
                    }
                },
                '"' => loop {
                    let c = chars.next()?;
                    self.pos += c.len_utf8();
                    match c {
                        '\\' => {
                            let c = chars.next()?;
                            self.pos += c.len_utf8();
                        }
                        '"' => {
                            break Token::String;
                        }
                        _ => {}
                    }
                },
                '\'' => {
                    let c = chars.next()?;
                    self.pos += c.len_utf8();
                    if c == '\\' {
                        let c = chars.next()?;
                        self.pos += c.len_utf8();
                    }
                    let c = chars.next()?;
                    self.pos += c.len_utf8();
                    if c == '\'' {
                        Token::Char
                    } else {
                        self.unexpected_char(c);
                        continue;
                    }
                }
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBrack,
                ']' => Token::RBrack,
                '=' => match chars.next() {
                    Some('=') => {
                        self.pos += '='.len_utf8();
                        Token::EqEq
                    }
                    Some('>') => {
                        self.pos += '>'.len_utf8();
                        Token::FatArrow
                    }
                    _ => Token::Eq,
                },
                ':' => {
                    if let Some(':') = chars.next() {
                        self.pos += ':'.len_utf8();
                        Token::ColonColon
                    } else {
                        Token::Colon
                    }
                }
                '!' => {
                    if let Some('=') = chars.next() {
                        self.pos += '='.len_utf8();
                        Token::NotEq
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    if let Some('=') = chars.next() {
                        self.pos += '='.len_utf8();
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    if let Some('=') = chars.next() {
                        self.pos += '='.len_utf8();
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                '.' => {
                    if let Some('.') = chars.next() {
                        self.pos += '.'.len_utf8();
                        Token::DotDot
                    } else {
                        Token::Dot
                    }
                }
                ';' => Token::SemiColon,
                ',' => Token::Comma,
                '+' => Token::Plus,
                '|' => Token::Bar,
                '-' => {
                    if let (Some('-'), Some('-')) = (chars.next(), chars.next()) {
                        self.pos += '-'.len_utf8() * 2;
                        loop {
                            let c = chars.next()?;
                            self.pos += c.len_utf8();
                            if c == '-' {
                                let c = chars.next()?;
                                self.pos += c.len_utf8();
                                if c == '-' {
                                    let c = chars.next()?;
                                    self.pos += c.len_utf8();
                                    if c == '-' {
                                        break;
                                    }
                                }
                            }
                        }
                        Token::Code
                    } else {
                        Token::Minus
                    }
                }
                '*' => Token::Star,
                '/' => Token::Slash,
                '?' => Token::Question,
                t => {
                    self.unexpected_char(t);
                    Token::Err
                }
            };
            return Some(Spanned::new(
                Span::new(self.file, start as u32..self.pos as u32),
                token,
            ));
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Spanned<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lex().or_else(|| {
            if self.eof {
                None
            } else {
                self.eof = true;
                let span = Span::new(self.file, self.pos as u32..self.pos as u32);
                Some(Spanned::new(span, Token::Eof))
            }
        })
    }
}
