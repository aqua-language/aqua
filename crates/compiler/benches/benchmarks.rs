use std::hint::black_box;
use std::rc::Rc;

use compiler::analysis;
use compiler::aqua;
use compiler::pass;
use compiler::pass::Pass as _;
use compiler::syntax::lexer::Lexer;
use compiler::syntax::parser::Parser;
use compiler::syntax::source::Cache;
use compiler::Compiler;
use divan::Bencher;

#[derive(Copy, Clone)]
struct Input {
    name: &'static str,
    code: &'static str,
}

#[derive(Copy, Clone)]
struct Benchmark {
    input: Input,
    pass: Pass,
}

#[derive(Copy, Clone)]
enum Pass {
    Lexer,
    Parser,
    Desugar,
    QueryComp,
    Resolve,
    Infer,
    Check,
    // Monomorphise,
}

const PASSES: &[Pass] = &[
    Pass::Lexer,
    Pass::Parser,
    Pass::Desugar,
    Pass::QueryComp,
    Pass::Resolve,
    Pass::Infer,
    Pass::Check,
    // Pass::Monomorphise,
];

fn benchmarks() -> impl Iterator<Item = Benchmark> {
    INPUTS
        .iter()
        .flat_map(|&input| PASSES.iter().map(move |&pass| Benchmark { input, pass }))
}

fn main() {
    divan::main();
}

const INPUTS: &[Input] = &[
    Input {
        name: "empty",
        code: aqua!(""),
    },
    Input {
        name: "num1",
        code: aqua!("0;"),
    },
    Input {
        name: "num2",
        code: aqua!("0; 0;"),
    },
    Input {
        name: "num4",
        code: aqua!("0; 0; 0; 0;"),
    },
    Input {
        name: "num8",
        code: aqua!("0; 0; 0; 0; 0; 0; 0; 0;"),
    },
    Input {
        name: "num256",
        code: aqua!(
            "0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;
             0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0;"
        ),
    },
    Input {
        name: "function-fib",
        code: aqua!(
            r#"# The Fibonacci sequence function.
               def fib(n: i32): i32 = {
                   if n <= 1 {
                       n
                   } else {
                       fib(n - 1) + fib(n - 2)
                   }
               }"#
        ),
    },
    Input {
        name: "struct-point",
        code: aqua!("struct Point[T](x:T, y:T, z:T);"),
    },
    Input {
        name: "add2",
        code: aqua!("1 + 1;"),
    },
    Input {
        name: "add4",
        code: aqua!("1 + 1 + 1 + 1;"),
    },
    Input {
        name: "add8",
        code: aqua!("1 + 1 + 1 + 1 + 1 + 1 + 1 + 1;"),
    },
    // Input {
    //     name: "add256",
    //     code: aqua!(
    //         "1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+
    //          1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1;"
    //     ),
    // },
    Input {
        name: "brace1",
        code: aqua!("{}"),
    },
    Input {
        name: "brace2",
        code: aqua!("{{}}"),
    },
    Input {
        name: "brace4",
        code: aqua!("{{{{}}}}"),
    },
    Input {
        name: "brace8",
        code: aqua!("{{{{{{{{}}}}}}}}"),
    },
    Input {
        name: "brace16",
        code: aqua!("{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}"),
    },
];

impl std::fmt::Display for Pass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass::Lexer => write!(f, "(1) lexer"),
            Pass::Parser => write!(f, "(2) parser"),
            Pass::Desugar => write!(f, "(3) desugar"),
            Pass::QueryComp => write!(f, "(4) querycomp"),
            Pass::Resolve => write!(f, "(5) resolve"),
            Pass::Infer => write!(f, "(6) infer"),
            Pass::Check => write!(f, "(7) check"),
            // Pass::Monomorphise => write!(f, "(8) monomorphise"),
        }
    }
}

impl std::fmt::Display for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.input, self.pass)
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[divan::bench(args = benchmarks())]
fn benchmark(bencher: Bencher, arg: Benchmark) {
    match arg.pass {
        Pass::Lexer => lexer(bencher, arg),
        Pass::Parser => parser(bencher, arg.input),
        Pass::Desugar => desugar(bencher, arg.input),
        Pass::QueryComp => querycomp(bencher, arg.input),
        Pass::Resolve => resolve(bencher, arg.input),
        Pass::Infer => infer(bencher, arg.input),
        Pass::Check => check(bencher, arg.input),
        // Pass::Monomorphise => {} //,monomorphise(bencher, arg.input),
    }
}

fn lexer(bencher: Bencher, arg: Benchmark) {
    bencher
        .with_inputs(|| {
            let mut cache = Cache::new();
            let source: Rc<str> = arg.input.code.into();
            let id = cache.add(arg.input.name, source.clone());
            (id, source)
        })
        .bench_local_values(|(id, code)| {
            let lexer = Lexer::new(id, &code);
            for token in lexer {
                black_box(token);
            }
        });
}

fn parser(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut cache = Cache::new();
            let code: Rc<str> = arg.code.into();
            let id = cache.add(arg.name, code.clone());
            (id, code)
        })
        .bench_local_values(|(id, code)| {
            let lexer = Lexer::new(id, &code);
            let mut parser = Parser::new(&code, lexer);
            parser.parse(Parser::program).unwrap()
        });
}

fn desugar(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut compiler = Compiler::default();
            let program = compiler.parse(arg.name, arg.code);
            let ctx = compiler::pass::desugar::Context::new();
            (ctx, program)
        })
        .bench_local_values(|(mut ctx, program)| {
            ctx.run(&program);
            program
        });
}

fn querycomp(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut compiler = Compiler::default();
            let program = compiler.parse(arg.name, arg.code);
            let program = compiler.query_desugar(&program);
            let ctx = pass::query_desugar::Context::new();
            (ctx, program)
        })
        .bench_local_values(|(mut ctx, program)| ctx.run(&program));
}

fn resolve(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut compiler = Compiler::default();
            compiler.init();
            let program = compiler.parse(arg.name, arg.code);
            let program = compiler.resolve(&program);
            let ctx = pass::query_desugar::Context::new();
            (ctx, program)
        })
        .bench_local_values(|(mut ctx, program)| ctx.run(&program));
}

fn infer(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut compiler = Compiler::default();
            compiler.init();
            let program = compiler.parse(arg.name, arg.code);
            let program = compiler.infer(&program);
            let ctx = pass::query_desugar::Context::new();
            (ctx, program)
        })
        .bench_local_values(|(mut ctx, program)| ctx.run(&program));
}

fn check(bencher: Bencher, arg: Input) {
    bencher
        .with_inputs(|| {
            let mut compiler = Compiler::default();
            compiler.init();
            let program = compiler.parse(arg.name, arg.code);
            let program = compiler.compile(&program);
            program
        })
        .bench_local_values(|program| analysis::check::check(&program))
}
