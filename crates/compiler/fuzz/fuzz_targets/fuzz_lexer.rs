#![no_main]

use compiler::source::SourceId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        for _ in compiler::lexer::Lexer::new(SourceId::new("file0", s), s) {}
    }
});
