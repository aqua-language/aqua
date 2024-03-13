#![no_main]

use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::Type;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Program::parse);
});

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Expr::parse);
});

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Type::parse);
});

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Stmt::parse);
});

fuzz_target!(|data: &[u8]| {
    std::str::from_utf8(data).ok().map(Pat::parse);
});
