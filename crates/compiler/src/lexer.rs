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
    Select,
    Struct,
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Not => write!(f, "!"),
            Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Ge => write!(f, ">="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::SemiColon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBrack => write!(f, "["),
            Token::RBrack => write!(f, "]"),
            Token::Underscore => write!(f, "_"),
            Token::Question => write!(f, "?"),
            Token::FatArrow => write!(f, "=>"),
            Token::Bar => write!(f, "|"),
            // Keywords
            Token::And => write!(f, "and"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Def => write!(f, "def"),
            Token::Deploy => write!(f, "deploy"),
            Token::Desc => write!(f, "desc"),
            Token::Else => write!(f, "else"),
            Token::Enum => write!(f, "enum"),
            Token::False => write!(f, "false"),
            Token::For => write!(f, "for"),
            Token::From => write!(f, "from"),
            Token::Fun => write!(f, "fun"),
            Token::Group => write!(f, "group"),
            Token::If => write!(f, "if"),
            Token::Impl => write!(f, "impl"),
            Token::In => write!(f, "in"),
            Token::Into => write!(f, "into"),
            Token::Join => write!(f, "join"),
            Token::Match => write!(f, "match"),
            Token::On => write!(f, "on"),
            Token::Or => write!(f, "or"),
            Token::Order => write!(f, "order"),
            Token::Over => write!(f, "over"),
            Token::Return => write!(f, "return"),
            Token::Select => write!(f, "select"),
            Token::Struct => write!(f, "struct"),
            Token::True => write!(f, "true"),
            Token::Type => write!(f, "type"),
            Token::Var => write!(f, "var"),
            Token::Where => write!(f, "where"),
            Token::While => write!(f, "while"),
            Token::With => write!(f, "with"),
            // Literals
            Token::Code => write!(f, "<code>"),
            Token::Name => write!(f, "<name>"),
            Token::Int => write!(f, "<int>"),
            Token::IntSuffix => write!(f, "<int-suffix>"),
            Token::Float => write!(f, "<float>"),
            Token::FloatSuffix => write!(f, "<float-suffix>"),
            Token::String => write!(f, "<string>"),
            Token::Char => write!(f, "<char>"),
            Token::Err => write!(f, "<err>"),
            Token::Eof => write!(f, "<eof>"),
            Token::Of => write!(f, "of"),
            Token::As => write!(f, "as"),
            Token::Compute => write!(f, "compute"),
            Token::Trait => write!(f, "trait"),
            _ => write!(f, "<unknown>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Spanned<T> {
        Spanned { span, value }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            span: self.span,
            value: f(self.value),
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
        match self.value {
            Some(value) => Some(Spanned {
                span: self.span,
                value,
            }),
            None => None,
        }
    }
}

impl Spanned<Token> {
    pub fn text(self, input: &str) -> &str {
        let span = self.span;
        match self.value {
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
                                } else {
                                    break Token::Float;
                                }
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
                            } else {
                                break Token::Int;
                            }
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
