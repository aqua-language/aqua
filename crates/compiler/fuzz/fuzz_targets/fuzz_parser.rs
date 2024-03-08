#![no_main]

use compiler::ast::Program;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        Program::parse_ok(s);
    }
});
