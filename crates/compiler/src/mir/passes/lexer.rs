use crate::token::Spanned;
use crate::token::Token;

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Spanned<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.input[self.pos..].chars();
        let mut start = self.pos;
        let token = loop {
            let c = chars.next()?;
            match c {
                '\'' => {
                    self.advance();
                    let c = chars.next()?;
                    if !c.is_alphanumeric() {
                        panic!("Expected alphnumeric");
                    }
                    self.advance();
                    while let Some(c) = chars.next() {
                        if c.is_alphanumeric() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Token::Label
                }
                '/' => {
                    self.advance();
                    match chars.next()? {
                        '/' => {
                            self.advance();
                            while let Some(c) = chars.next() {
                                if c == '\n' {
                                    break;
                                }
                                self.advance();
                            }
                            start = self.pos;
                            continue;
                        }
                        '*' => {
                            self.advance();
                            loop {
                                let c = chars.next().expect("unexpected end of input");
                                self.advance();
                                if c == '*' {
                                    let c = chars.next().expect("unexpected end of input");
                                    if c == '/' {
                                        self.advance();
                                        break;
                                    } else {
                                        chars = self.input[self.pos..].chars();
                                    }
                                }
                            }
                            start = self.pos;
                            continue;
                        }
                        _ => break Token::Slash,
                    }
                }
                '\n' | ' ' | '\t' => {
                    self.advance();
                    start = self.pos;
                    continue;
                }
                '=' => {
                    self.advance();
                    break Token::Equal;
                }
                '0'..='9' => {
                    self.advance();
                    while let Some(c) = chars.next() {
                        if c.is_ascii_digit() {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                    break Token::Number;
                }
                '"' => {
                    self.advance();
                    while let Some(c) = chars.next() {
                        self.advance();
                        if c == '"' {
                            break;
                        }
                    }
                    break Token::String;
                }
                ':' => {
                    self.advance();
                    break Token::Colon;
                }
                ';' => {
                    self.advance();
                    break Token::SemiColon;
                }
                '{' => {
                    self.advance();
                    Token::LeftBrace
                }
                '}' => {
                    self.advance();
                    Token::RightBrace
                }
                '&' => {
                    self.advance();
                    Token::Ampersand
                }
                '.' => {
                    self.advance();
                    break Token::Dot;
                }
                '+' => {
                    self.advance();
                    break Token::Plus;
                }
                '-' => {
                    self.advance();
                    break Token::Minus;
                }
                '*' => {
                    self.advance();
                    break Token::Star;
                }
                '(' => {
                    self.advance();
                    break Token::LeftParen;
                }
                ')' => {
                    self.advance();
                    break Token::RightParen;
                }
                'a'..='z' | 'A'..='Z' => {
                    self.advance();
                    while let Some(c) = chars.next() {
                        match c {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                self.advance();
                                continue;
                            }
                            _ => break,
                        }
                    }
                    match &self.input[start..self.pos] {
                        "fn" => break Token::Ident,
                        "let" => break Token::Ident,
                        "if" => break Token::Ident,
                        "else" => break Token::Ident,
                        "while" => break Token::Ident,
                        "return" => break Token::Ident,
                        "mut" => break Token::Ident,
                        _ => break Token::Ident,
                    }
                }
                _ => {
                    self.advance();
                    Token::Err
                }
            };
        };
        Some(Spanned::new(token, start..self.pos))
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { input, pos: 0 }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
