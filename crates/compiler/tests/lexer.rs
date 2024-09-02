use compiler::lexer::Lexer;
use compiler::sources::Sources;
use compiler::token::Token;

fn next<'a>(lexer: &mut Lexer<'a>) -> Option<(std::ops::Range<u32>, Token, &'a str)> {
    lexer
        .next()
        .map(|t| ((*t.s.start()..*t.s.end()), t.v, lexer.text(t)))
}

#[test]
fn test_lexer_int0() {
    let lexer = &mut Lexer::new(0, "123");
    assert_eq!(next(lexer), Some((0..3, Token::Int, "123")));
    assert_eq!(next(lexer), Some((3..3, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_int_suffix0() {
    let lexer = &mut Lexer::new(0, "60s");
    assert_eq!(next(lexer), Some((0..3, Token::IntSuffix, "60s")));
    assert_eq!(next(lexer), Some((3..3, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_int_suffix1() {
    let lexer = &mut Lexer::new(0, "60_foo");
    assert_eq!(next(lexer), Some((0..6, Token::IntSuffix, "60_foo")));
    assert_eq!(next(lexer), Some((6..6, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_float0() {
    let lexer = &mut Lexer::new(0, "123.456");
    assert_eq!(next(lexer), Some((0..7, Token::Float, "123.456")));
    assert_eq!(next(lexer), Some((7..7, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_float1() {
    let lexer = &mut Lexer::new(0, "123.");
    assert_eq!(next(lexer), Some((0..3, Token::Int, "123")));
    assert_eq!(next(lexer), Some((3..4, Token::Dot, ".")));
    assert_eq!(next(lexer), Some((4..4, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_float_suffix0() {
    let lexer = &mut Lexer::new(0, "60.0ms");
    assert_eq!(next(lexer), Some((0..6, Token::FloatSuffix, "60.0ms")));
    assert_eq!(next(lexer), Some((6..6, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_float_suffix1() {
    let lexer = &mut Lexer::new(0, "60.0_foo");
    assert_eq!(next(lexer), Some((0..8, Token::FloatSuffix, "60.0_foo")));
    assert_eq!(next(lexer), Some((8..8, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_name0() {
    let lexer = &mut Lexer::new(0, "abc");
    assert_eq!(next(lexer), Some((0..3, Token::Name, "abc")));
    assert_eq!(next(lexer), Some((3..3, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_name1() {
    let lexer = &mut Lexer::new(0, "_ _1 _a a_ a_1");
    assert_eq!(next(lexer), Some((0..1, Token::Underscore, "_")));
    assert_eq!(next(lexer), Some((2..4, Token::Name, "_1")));
    assert_eq!(next(lexer), Some((5..7, Token::Name, "_a")));
    assert_eq!(next(lexer), Some((8..10, Token::Name, "a_")));
    assert_eq!(next(lexer), Some((11..14, Token::Name, "a_1")));
    assert_eq!(next(lexer), Some((14..14, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_keywords0() {
    let lexer = &mut Lexer::new(0, "def impl struct enum var type");
    assert_eq!(next(lexer), Some((0..3, Token::Def, "def")));
    assert_eq!(next(lexer), Some((4..8, Token::Impl, "impl")));
    assert_eq!(next(lexer), Some((9..15, Token::Struct, "struct")));
    assert_eq!(next(lexer), Some((16..20, Token::Enum, "enum")));
    assert_eq!(next(lexer), Some((21..24, Token::Var, "var")));
    assert_eq!(next(lexer), Some((25..29, Token::Type, "type")));
    assert_eq!(next(lexer), Some((29..29, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_keywords1() {
    let lexer = &mut Lexer::new(
        0,
        "true false or and if else while for break continue return match fun",
    );
    assert_eq!(next(lexer), Some((0..4, Token::True, "true")));
    assert_eq!(next(lexer), Some((5..10, Token::False, "false")));
    assert_eq!(next(lexer), Some((11..13, Token::Or, "or")));
    assert_eq!(next(lexer), Some((14..17, Token::And, "and")));
    assert_eq!(next(lexer), Some((18..20, Token::If, "if")));
    assert_eq!(next(lexer), Some((21..25, Token::Else, "else")));
    assert_eq!(next(lexer), Some((26..31, Token::While, "while")));
    assert_eq!(next(lexer), Some((32..35, Token::For, "for")));
    assert_eq!(next(lexer), Some((36..41, Token::Break, "break")));
    assert_eq!(next(lexer), Some((42..50, Token::Continue, "continue")));
    assert_eq!(next(lexer), Some((51..57, Token::Return, "return")));
    assert_eq!(next(lexer), Some((58..63, Token::Match, "match")));
    assert_eq!(next(lexer), Some((64..67, Token::Fun, "fun")));
    assert_eq!(next(lexer), Some((67..67, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_keywords2() {
    let lexer = &mut Lexer::new(0, "from into where select group with over join on");
    assert_eq!(next(lexer), Some((0..4, Token::From, "from")));
    assert_eq!(next(lexer), Some((5..9, Token::Into, "into")));
    assert_eq!(next(lexer), Some((10..15, Token::Where, "where")));
    assert_eq!(next(lexer), Some((16..22, Token::Select, "select")));
    assert_eq!(next(lexer), Some((23..28, Token::Group, "group")));
    assert_eq!(next(lexer), Some((29..33, Token::With, "with")));
    assert_eq!(next(lexer), Some((34..38, Token::Over, "over")));
    assert_eq!(next(lexer), Some((39..43, Token::Join, "join")));
    assert_eq!(next(lexer), Some((44..46, Token::On, "on")));
    assert_eq!(next(lexer), Some((46..46, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_punct0() {
    let lexer = &mut Lexer::new(0, "= == != < <= > >= + - * / . .. , : ; ? _ => ::");
    assert_eq!(next(lexer), Some((0..1, Token::Eq, "=")));
    assert_eq!(next(lexer), Some((2..4, Token::EqEq, "==")));
    assert_eq!(next(lexer), Some((5..7, Token::NotEq, "!=")));
    assert_eq!(next(lexer), Some((8..9, Token::Lt, "<")));
    assert_eq!(next(lexer), Some((10..12, Token::Le, "<=")));
    assert_eq!(next(lexer), Some((13..14, Token::Gt, ">")));
    assert_eq!(next(lexer), Some((15..17, Token::Ge, ">=")));
    assert_eq!(next(lexer), Some((18..19, Token::Plus, "+")));
    assert_eq!(next(lexer), Some((20..21, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((22..23, Token::Star, "*")));
    assert_eq!(next(lexer), Some((24..25, Token::Slash, "/")));
    assert_eq!(next(lexer), Some((26..27, Token::Dot, ".")));
    assert_eq!(next(lexer), Some((28..30, Token::DotDot, "..")));
    assert_eq!(next(lexer), Some((31..32, Token::Comma, ",")));
    assert_eq!(next(lexer), Some((33..34, Token::Colon, ":")));
    assert_eq!(next(lexer), Some((35..36, Token::SemiColon, ";")));
    assert_eq!(next(lexer), Some((37..38, Token::Question, "?")));
    assert_eq!(next(lexer), Some((39..40, Token::Underscore, "_")));
    assert_eq!(next(lexer), Some((41..43, Token::FatArrow, "=>")));
    assert_eq!(next(lexer), Some((44..46, Token::ColonColon, "::")));
    assert_eq!(next(lexer), Some((46..46, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_string0() {
    let lexer = &mut Lexer::new(0, r#""abc""#);
    assert_eq!(next(lexer), Some((0..5, Token::String, "abc")));
    assert_eq!(next(lexer), Some((5..5, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_string1() {
    let lexer = &mut Lexer::new(0, r#""abc\"def""#);
    assert_eq!(next(lexer), Some((0..10, Token::String, r#"abc\"def"#)));
    assert_eq!(next(lexer), Some((10..10, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_char0() {
    let lexer = &mut Lexer::new(0, "'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j'");
    assert_eq!(next(lexer), Some((0..3, Token::Char, "a")));
    assert_eq!(next(lexer), Some((4..7, Token::Char, "b")));
    assert_eq!(next(lexer), Some((8..11, Token::Char, "c")));
    assert_eq!(next(lexer), Some((12..15, Token::Char, "d")));
    assert_eq!(next(lexer), Some((16..19, Token::Char, "e")));
    assert_eq!(next(lexer), Some((20..23, Token::Char, "f")));
    assert_eq!(next(lexer), Some((24..27, Token::Char, "g")));
    assert_eq!(next(lexer), Some((28..31, Token::Char, "h")));
    assert_eq!(next(lexer), Some((32..35, Token::Char, "i")));
    assert_eq!(next(lexer), Some((36..39, Token::Char, "j")));
    assert_eq!(next(lexer), Some((39..39, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_char1() {
    let lexer = &mut Lexer::new(0, r"'\n' '\t' '\r' '\0' '\\' '\''");
    assert_eq!(next(lexer), Some((0..4, Token::Char, r"\n")));
    assert_eq!(next(lexer), Some((5..9, Token::Char, r"\t")));
    assert_eq!(next(lexer), Some((10..14, Token::Char, r"\r")));
    assert_eq!(next(lexer), Some((15..19, Token::Char, r"\0")));
    assert_eq!(next(lexer), Some((20..24, Token::Char, r"\\")));
    assert_eq!(next(lexer), Some((25..29, Token::Char, r"\'")));
    assert_eq!(next(lexer), Some((29..29, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_separators0() {
    let lexer = &mut Lexer::new(0, "[[]] (()) {{}}");
    assert_eq!(next(lexer), Some((0..1, Token::LBrack, "[")));
    assert_eq!(next(lexer), Some((1..2, Token::LBrack, "[")));
    assert_eq!(next(lexer), Some((2..3, Token::RBrack, "]")));
    assert_eq!(next(lexer), Some((3..4, Token::RBrack, "]")));
    assert_eq!(next(lexer), Some((5..6, Token::LParen, "(")));
    assert_eq!(next(lexer), Some((6..7, Token::LParen, "(")));
    assert_eq!(next(lexer), Some((7..8, Token::RParen, ")")));
    assert_eq!(next(lexer), Some((8..9, Token::RParen, ")")));
    assert_eq!(next(lexer), Some((10..11, Token::LBrace, "{")));
    assert_eq!(next(lexer), Some((11..12, Token::LBrace, "{")));
    assert_eq!(next(lexer), Some((12..13, Token::RBrace, "}")));
    assert_eq!(next(lexer), Some((13..14, Token::RBrace, "}")));
    assert_eq!(next(lexer), Some((14..14, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_code0() {
    let lexer = &mut Lexer::new(0, "--- fn main() {} ---");
    assert_eq!(next(lexer), Some((0..20, Token::Code, " fn main() {} ")));
    assert_eq!(next(lexer), Some((20..20, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_code1() {
    let lexer = &mut Lexer::new(0, "1 + 2; --- fn main() {} --- 3 + 4;");
    assert_eq!(next(lexer), Some((0..1, Token::Int, "1")));
    assert_eq!(next(lexer), Some((2..3, Token::Plus, "+")));
    assert_eq!(next(lexer), Some((4..5, Token::Int, "2")));
    assert_eq!(next(lexer), Some((5..6, Token::SemiColon, ";")));
    assert_eq!(next(lexer), Some((7..27, Token::Code, " fn main() {} ")));
    assert_eq!(next(lexer), Some((28..29, Token::Int, "3")));
    assert_eq!(next(lexer), Some((30..31, Token::Plus, "+")));
    assert_eq!(next(lexer), Some((32..33, Token::Int, "4")));
    assert_eq!(next(lexer), Some((33..34, Token::SemiColon, ";")));
    assert_eq!(next(lexer), Some((34..34, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_code2() {
    let lexer = &mut Lexer::new(0, "-- -");
    assert_eq!(next(lexer), Some((0..1, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((1..2, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((3..4, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((4..4, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_code3() {
    let lexer = &mut Lexer::new(0, "--- -- - ---");
    assert_eq!(next(lexer), Some((0..12, Token::Code, " -- - ")));
    assert_eq!(next(lexer), Some((12..12, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_comments0() {
    let lexer = &mut Lexer::new(0, "1 # + 2");
    assert_eq!(next(lexer), Some((0..1, Token::Int, "1")));
    assert_eq!(next(lexer), Some((7..7, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_comments1() {
    let lexer = &mut Lexer::new(0, "1 # + 2\n3");
    assert_eq!(next(lexer), Some((0..1, Token::Int, "1")));
    assert_eq!(next(lexer), Some((8..9, Token::Int, "3")));
    assert_eq!(next(lexer), Some((9..9, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_err0() {
    let lexer = &mut Lexer::new(0, "\\ % ^ & ~ ` $");
    assert_eq!(next(lexer), Some((0..1, Token::Err, "\\")));
    assert_eq!(next(lexer), Some((2..3, Token::Err, "%")));
    assert_eq!(next(lexer), Some((4..5, Token::Err, "^")));
    assert_eq!(next(lexer), Some((6..7, Token::Err, "&")));
    assert_eq!(next(lexer), Some((8..9, Token::Err, "~")));
    assert_eq!(next(lexer), Some((10..11, Token::Err, "`")));
    assert_eq!(next(lexer), Some((12..13, Token::Err, "$")));
    assert_eq!(next(lexer), Some((13..13, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(!lexer.report.is_empty());
}

#[test]
fn test_lexer_err1() {
    let mut sources = Sources::new();
    let source = "%";
    let file = sources.add("file", source);
    let lexer = &mut Lexer::new(file, source);
    assert_eq!(next(lexer), Some((0..1, Token::Err, "%")));
    assert_eq!(next(lexer), Some((1..1, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert_eq!(
        lexer.report.string(&mut sources).unwrap(),
        indoc::indoc! {"
        Error: Unexpected character
           ╭─[file:1:1]
           │
         1 │ %
           │ ┬  
           │ ╰── Unexpected character '%'
        ───╯

        "}
    );
}

#[test]
fn test_lexer_unused0() {
    let lexer = &mut Lexer::new(0, "-> <- ..=");
    assert_eq!(next(lexer), Some((0..1, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((1..2, Token::Gt, ">")));
    assert_eq!(next(lexer), Some((3..4, Token::Lt, "<")));
    assert_eq!(next(lexer), Some((4..5, Token::Minus, "-")));
    assert_eq!(next(lexer), Some((6..8, Token::DotDot, "..")));
    assert_eq!(next(lexer), Some((8..9, Token::Eq, "=")));
    assert_eq!(next(lexer), Some((9..9, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_eof0() {
    let lexer = &mut Lexer::new(0, "");
    assert_eq!(next(lexer), Some((0..0, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_eof1() {
    let lexer = &mut Lexer::new(0, " ");
    assert_eq!(next(lexer), Some((1..1, Token::Eof, "")));
    assert_eq!(next(lexer), None);
    assert!(lexer.report.is_empty());
}

#[test]
fn test_lexer_range0() {
    let lexer = &mut Lexer::new(0, ".. 1.. ..2 1..2");
    assert_eq!(next(lexer), Some(((0..2), Token::DotDot, "..")));
    assert_eq!(next(lexer), Some(((3..4), Token::Int, "1")));
    assert_eq!(next(lexer), Some(((4..6), Token::DotDot, "..")));
    assert_eq!(next(lexer), Some(((7..9), Token::DotDot, "..")));
    assert_eq!(next(lexer), Some(((9..10), Token::Int, "2")));
    assert_eq!(next(lexer), Some(((11..12), Token::Int, "1")));
    assert_eq!(next(lexer), Some(((12..14), Token::DotDot, "..")));
    assert_eq!(next(lexer), Some(((14..15), Token::Int, "2")));
}
