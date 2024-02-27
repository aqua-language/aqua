#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let lexer = compiler::lexer::Lexer::new(0, s);
        let mut parser = compiler::parser::Parser::new(s, lexer);
        parser.parse();
    }
});
