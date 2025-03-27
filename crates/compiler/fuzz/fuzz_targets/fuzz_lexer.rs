#![no_main]

use std::rc::Rc;

use compiler::ast::parse::lexer::Lexer;
use compiler::report::source::Cache;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut cache = Cache::new();
        let s: Rc<str> = s.into();
        let id = cache.add("test", s.clone());
        for _ in Lexer::new(id, &s) {}
    }
});
