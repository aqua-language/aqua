use crate::diag::Report;
use crate::syntax::source::SourceId;
use crate::syntax::span::Span;
use crate::syntax::spanned::Spanned;
use crate::syntax::token::Token;

pub struct Lexer<'a> {
    pub input: &'a str,
    pos: usize,
    eof: bool,
    pub file: SourceId,
    pub report: Report,
}

impl<'a> Lexer<'a> {
    pub fn new(file: SourceId, input: &'a str) -> Lexer<'a> {
        Lexer {
            file,
            input,
            eof: false,
            pos: 0,
            report: Report::new(),
        }
    }

    pub fn new_from(file: SourceId, input: &'a str, pos: usize) -> Lexer<'a> {
        Lexer {
            file,
            input,
            eof: false,
            pos,
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

    #[inline(always)]
    fn lex(&mut self) -> Option<Spanned<Token>> {
        loop {
            let start = self.pos;
            let mut chars = self.input[self.pos..].chars();
            let c = chars.next()?;
            let token = match c {
                ' ' | '\n' | '\t' => {
                    self.pos += 1;
                    continue;
                }
                ';' => {
                    self.pos += 1;
                    Token::SemiColon
                }
                '0'..='9' => {
                    self.pos += 1;
                    loop {
                        match chars.next() {
                            Some('0'..='9') => self.pos += 1,
                            Some('.') => match chars.next() {
                                Some('0'..='9') => {
                                    self.pos += 2;
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
                            Some('a'..='z' | 'A'..='Z' | '_') => {
                                self.pos += 1;
                                while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') =
                                    chars.next()
                                {
                                    self.pos += 1;
                                }
                                break Token::IntSuffix;
                            }
                            Some(_) | None => {
                                break Token::Int;
                            }
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.pos += 1;
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
                            "else" => Token::Else,
                            "enum" => Token::Enum,
                            "false" => Token::False,
                            "for" => Token::For,
                            "from" => Token::From,
                            "group" => Token::Group,
                            "if" => Token::If,
                            "impl" => Token::Impl,
                            "in" => Token::In,
                            "into" => Token::Into,
                            "join" => Token::Join,
                            "match" => Token::Match,
                            "mut" => Token::Mut,
                            "let" => Token::Let,
                            "of" => Token::Of,
                            "on" => Token::On,
                            "or" => Token::Or,
                            "over" => Token::Over,
                            "return" => Token::Return,
                            "select" => Token::Select,
                            "limit" => Token::Limit,
                            "struct" => Token::Struct,
                            "record" => Token::Record,
                            "trait" => Token::Trait,
                            "true" => Token::True,
                            "type" => Token::Type,
                            "var" => Token::Var,
                            "where" => Token::Where,
                            "while" => Token::While,
                            "with" => Token::With,
                            "drop" => Token::Drop,
                            _ => Token::Name,
                        }
                    }
                }
                '"' => {
                    self.pos += 1;
                    loop {
                        let c = chars.next()?;
                        match c {
                            '\\' => {
                                self.pos += 1;
                                let c = chars.next()?;
                                self.pos += c.len_utf8();
                            }
                            '"' => {
                                self.pos += 1;
                                break Token::String;
                            }
                            _ => {
                                self.pos += c.len_utf8();
                            }
                        }
                    }
                }
                '\'' => {
                    self.pos += 1;
                    let c = chars.next()?;
                    if c == '\\' {
                        self.pos += 1;
                        let c = chars.next()?;
                        self.pos += c.len_utf8();
                    } else {
                        self.pos += c.len_utf8();
                    }
                    let c = chars.next()?;
                    if c == '\'' {
                        self.pos += 1;
                        Token::Char
                    } else {
                        self.pos += c.len_utf8();
                        self.unexpected_char(c);
                        continue;
                    }
                }
                '(' => {
                    self.pos += 1;
                    Token::LParen
                }
                ')' => {
                    self.pos += 1;
                    Token::RParen
                }
                '{' => {
                    self.pos += 1;
                    Token::LBrace
                }
                '}' => {
                    self.pos += 1;
                    Token::RBrace
                }
                '[' => {
                    self.pos += 1;
                    Token::LBrack
                }
                ']' => {
                    self.pos += 1;
                    Token::RBrack
                }
                '=' => {
                    self.pos += 1;
                    match chars.next() {
                        Some('=') => {
                            self.pos += 1;
                            Token::EqEq
                        }
                        Some('>') => {
                            self.pos += 1;
                            Token::FatArrow
                        }
                        _ => Token::Eq,
                    }
                }
                ':' => {
                    self.pos += 1;
                    if let Some(':') = chars.next() {
                        self.pos += 1;
                        Token::ColonColon
                    } else {
                        Token::Colon
                    }
                }
                '!' => {
                    self.pos += 1;
                    if let Some('=') = chars.next() {
                        self.pos += 1;
                        Token::NotEq
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    self.pos += 1;
                    if let Some('=') = chars.next() {
                        self.pos += 1;
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.pos += 1;
                    if let Some('=') = chars.next() {
                        self.pos += 1;
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                '.' => {
                    self.pos += 1;
                    if let Some('.') = chars.next() {
                        self.pos += 1;
                        Token::DotDot
                    } else {
                        Token::Dot
                    }
                }
                ',' => {
                    self.pos += 1;
                    Token::Comma
                }
                '+' => {
                    self.pos += 1;
                    Token::Plus
                }
                '|' => {
                    self.pos += 1;
                    Token::Bar
                }
                '-' => {
                    self.pos += 1;
                    if let (Some('-'), Some('-')) = (chars.next(), chars.next()) {
                        self.pos += 2;
                        loop {
                            let c = chars.next()?;
                            if c == '-' {
                                self.pos += 1;
                                let c = chars.next()?;
                                if c == '-' {
                                    self.pos += 1;
                                    let c = chars.next()?;
                                    if c == '-' {
                                        self.pos += 1;
                                        break;
                                    } else {
                                        self.pos += c.len_utf8();
                                    }
                                } else {
                                    self.pos += c.len_utf8();
                                }
                            } else {
                                self.pos += c.len_utf8();
                            }
                        }
                        Token::Code
                    } else {
                        Token::Minus
                    }
                }
                '*' => {
                    self.pos += 1;
                    Token::Star
                }
                '/' => {
                    self.pos += 1;
                    Token::Slash
                }
                '?' => {
                    self.pos += 1;
                    Token::Question
                }
                '#' => {
                    self.pos += 1;
                    for c in chars.by_ref() {
                        if c == '\n' {
                            break;
                        }
                        self.pos += c.len_utf8();
                    }
                    continue;
                }
                '&' => {
                    self.pos += 1;
                    Token::Ampersand
                }
                c => {
                    self.pos += c.len_utf8();
                    self.unexpected_char(c);
                    Token::Err
                }
            };
            let span = Span::new(self.file, start as u32..self.pos as u32);
            return Some(Spanned::new(span, token));
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Spanned<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.lex() {
            Some(token)
        } else {
            if !self.eof {
                self.eof = true;
                let span = Span::new(self.file, self.pos as u32..self.pos as u32);
                Some(Spanned::new(span, Token::Eof))
            } else {
                None
            }
        }
    }
}
