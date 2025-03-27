#![no_main]

use std::rc::Rc;

use compiler::ast::parse::lexer::Lexer;
use compiler::ast::parse::parser::Parser;
use compiler::report::source::Cache;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut cache = Cache::new();
        let s: Rc<str> = s.into();
        let id = cache.add("<fuzz>", s.clone());
        let lexer = Lexer::new(id, &s);
        let mut parser = Parser::new(&s, lexer);
        parser.parse(Parser::program).unwrap();
    }
});
