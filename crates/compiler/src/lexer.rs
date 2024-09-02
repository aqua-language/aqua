use crate::diag::Report;
use crate::span::Span;
use crate::spanned::Spanned;
use crate::token::Token;

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
                    while let Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_')) = chars.next() {
                        self.pos += c.len_utf8();
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
                            "let" => Token::Let,
                            "of" => Token::Of,
                            "on" => Token::On,
                            "or" => Token::Or,
                            "over" => Token::Over,
                            "return" => Token::Return,
                            "select" => Token::Select,
                            "struct" => Token::Struct,
                            "record" => Token::Record,
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
                                break Token::Int;
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
