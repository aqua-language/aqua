#![no_main]

use compiler::ast::Program;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Program::parse);
});
