use super::token::Token;

impl Token {
    pub fn expr_infix_bp(self) -> Option<(u8, u8)> {
        let bp = match self {
            Token::Eq => (1, 2),
            Token::DotDot => (2, 3),
            Token::And | Token::Or => (3, 4),
            Token::EqEq | Token::NotEq | Token::Lt | Token::Gt | Token::Le | Token::Ge => (4, 5),
            Token::Plus | Token::Minus => (5, 6),
            Token::Star | Token::Slash => (6, 7),
            _ => return None,
        };
        Some(bp)
    }

    pub fn expr_prefix_bp(self) -> Option<((), u8)> {
        let bp = match self {
            Token::Not | Token::Minus | Token::Star | Token::Ampersand => ((), 8),
            _ => return None,
        };
        Some(bp)
    }

    pub fn expr_postfix_bp(self) -> Option<(u8, ())> {
        let bp = match self {
            Token::Question | Token::LParen | Token::Dot | Token::Colon | Token::FatArrow => {
                (9, ())
            }
            _ => return None,
        };
        Some(bp)
    }

    pub fn pat_infix_bp(self) -> Option<(u8, u8)> {
        let bp = match self {
            Token::Eq => (1, 2),
            Token::Or => (2, 3),
            _ => return None,
        };
        Some(bp)
    }

    pub fn pat_postfix_bp(self) -> Option<(u8, ())> {
        let bp = match self {
            Token::Colon => (9, ()),
            _ => return None,
        };
        Some(bp)
    }

    pub fn ty_postfix_bp(self) -> Option<(u8, ())> {
        let bp = match self {
            Token::FatArrow => (9, ()),
            _ => return None,
        };
        Some(bp)
    }
}
