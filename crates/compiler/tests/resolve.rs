use compiler::ast::Program;
use compiler::util::bound;
use compiler::util::bound_err;
use compiler::util::expr_assoc;
use compiler::util::expr_call;
use compiler::util::expr_def;
use compiler::util::expr_enum;
use compiler::util::expr_err;
use compiler::util::expr_int;
use compiler::util::expr_struct;
use compiler::util::expr_tuple;
use compiler::util::expr_var;
use compiler::util::program;
use compiler::util::stmt_def;
use compiler::util::stmt_enum;
use compiler::util::stmt_expr;
use compiler::util::stmt_impl;
use compiler::util::stmt_struct;
use compiler::util::stmt_trait;
use compiler::util::stmt_type;
use compiler::util::stmt_var;
use compiler::util::tr_def;
use compiler::util::tr_type;
use compiler::util::ty;
use compiler::util::ty_alias;
use compiler::util::ty_assoc;
use compiler::util::ty_con;
use compiler::util::ty_err;
use compiler::util::ty_gen;
use compiler::util::ty_hole;
use compiler::util::ty_tuple;
use compiler::util::types::ty_i32;

fn diff(a: impl std::fmt::Debug, b: impl std::fmt::Debug) -> String {
    let a = format!("{:#?}", a);
    let b = format!("{:#?}", b);
    let mut output = String::new();
    let diff = similar::TextDiff::from_lines(&a, &b);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

macro_rules! ok {
    {$s:tt} => { { Program::try_resolve(indoc::indoc! { $s }).unwrap_or_else(|(_, s)| panic!("{}", s)) } }
}

macro_rules! err {
    {$s:tt} => { { Program::try_resolve(indoc::indoc! { $s }).unwrap_err() } }
}

#[test]
fn test_resolve_var0() {
    let a = ok!("var x = 0; x;");
    let b = program([
        stmt_var("x", ty_hole(), expr_int("0")),
        stmt_expr(expr_var("x")),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_var_err0() {
    let (a, msg) = err!("var x = 0; y;");
    let b = program([
        stmt_var("x", ty_hole(), expr_int("0")),
        stmt_expr(expr_err()),
    ]);
    assert!(a == b, "{a}\n\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
        Error: Name `y` not found.
           ╭─[test:1:12]
           │
         1 │ var x = 0; y;
           │            ┬
           │            ╰── Expected expression.
        ───╯"},
        "{msg}"
    )
}

#[test]
fn test_resolve_def0() {
    let a = ok!("def f(): i32 = 0; f();");
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_int("0")),
        stmt_expr(expr_call(expr_def("f", []), [])),
    ]);
    assert!(a == b, "{a:?}\n\n{b:?}");
}

#[test]
fn test_resolve_def1() {
    let a = ok!("def f(x: i32): i32 = x;");
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty_i32())],
        ty_i32(),
        [],
        expr_var("x"),
    )]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_def2() {
    let a = ok!("def f(x: i32): i32 = f(x);");
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty_i32())],
        ty_i32(),
        [],
        expr_call(expr_def("f", []), [expr_var("x")]),
    )]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_def_param1() {
    let (a, msg) = err!("def f(): i32 = x;");
    let b = program([stmt_def("f", [], [], ty_i32(), [], expr_err())]);
    assert!(a == b, "{a}\n\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
            Error: Name `x` not found.
               ╭─[test:1:16]
               │
             1 │ def f(): i32 = x;
               │                ┬
               │                ╰── Expected expression.
            ───╯"}
    )
}

#[test]
fn test_resolve_def_generic() {
    let a = ok!("def f[T](x: T): T = x;");
    let b = program([stmt_def(
        "f",
        ["T"],
        [("x", ty_gen("T"))],
        ty_gen("T"),
        [],
        expr_var("x"),
    )]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_resolve_type0() {
    let a = ok!("type T = i32;
                 var x: T = 0;");
    let b = program([
        stmt_type("T", [], ty_i32()),
        stmt_var("x", ty_alias("T", []), expr_int("0")),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_type1() {
    let a = ok!("type T[U] = (i32, U);
                 var x: T[i32] = (0, 0);");
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty_i32(), ty_gen("U")])),
        stmt_var(
            "x",
            ty_alias("T", [ty_i32()]),
            expr_tuple([expr_int("0"), expr_int("0")]),
        ),
    ]);
    assert!(a == b, "{}", diff(&a, &b));
}

#[test]
fn test_resolve_type2() {
    let (a, msg) = err!(
        "type T[U] = (i32, U);
         var x: T[i32, i32] = (0, 0);"
    );
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty_i32(), ty_gen("U")])),
        stmt_var("x", ty_err(), expr_tuple([expr_int("0"), expr_int("0")])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
            Error: Wrong number of type arguments. Found 2, expected 1
               ╭─[test:2:8]
               │
             2 │ var x: T[i32, i32] = (0, 0);
               │        ┬
               │        ╰── Expected 1 arguments.
            ───╯"}
    );
}

#[test]
fn test_resolve_trait0() {
    let a = ok!("trait Trait[T] {
                     def f(x:T): T;
                 }");
    let b = program([stmt_trait(
        "Trait",
        ["T"],
        [],
        [tr_def("f", [], [("x", ty_gen("T"))], [], ty_gen("T"))],
        [],
    )]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_trait_assoc0() {
    let a = ok!("trait Trait[T] {
                     def f(x:T): T;
                 }
                 def g[T](x:T): T where Trait[T] = f(x);");
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [("x", ty_gen("T"))], [], ty_gen("T"))],
            [],
        ),
        stmt_def(
            "g",
            ["T"],
            [("x", ty_gen("T"))],
            ty_gen("T"),
            [bound("Trait", [ty_gen("T")])],
            expr_call(expr_assoc("Trait", [ty_hole()], "f", []), [expr_var("x")]),
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_trait_impl0() {
    let a = ok!("trait Trait[T] {
                     def f(x:T): T;
                 }
                 impl Trait[i32] {
                     def f(x:i32): i32 = x;
                 }");
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [("x", ty_gen("T"))], [], ty_gen("T"))],
            [],
        ),
        stmt_impl(
            [],
            bound("Trait", [ty_i32()]),
            [],
            [stmt_def(
                "f",
                [],
                [("x", ty_i32())],
                ty_i32(),
                [],
                expr_var("x"),
            )],
            [],
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[ignore]
#[test]
fn test_resolve_trait_impl1() {
    let a = ok!("trait Trait[T] {
                     type A[U];
                 }
                 impl Trait[i32] {
                     type A[U] = U;
                 }");
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("f", [])]),
        stmt_impl(
            [],
            bound("Trait", [ty_i32()]),
            [],
            [],
            [stmt_type("A", ["U"], ty_gen("U"))],
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_trait_impl2() {
    let (a, msg) = err!(
        "trait Trait[T] {
             def f(x: T): T;
         }
         impl Trait[i32] {
             def g(x: i32): i32 = x;
         }"
    );
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [("x", ty_gen("T"))], [], ty_gen("T"))],
            [],
        ),
        stmt_impl(
            [],
            bound_err(),
            [],
            [stmt_def(
                "g",
                [],
                [("x", ty_i32())],
                ty_i32(),
                [],
                expr_var("x"),
            )],
            [],
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
        Error: Wrong defs implemented for Trait. Found { `g` }, expected { `f` }
           ╭─[test:4:6]
           │
         4 │ impl Trait[i32] {
           │      ──┬──
           │        ╰──── Expected { `f` }.
        ───╯"}
    );
}

#[test]
fn test_resolve_struct0() {
    let a = ok!("struct S[T](x:T);");
    let b = program([stmt_struct("S", ["T"], [("x", ty_gen("T"))])]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_struct1() {
    let a = ok!("struct S[T](x:T);
                 var s: S[i32] = S[i32](x=0);");
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_con("S", [ty_i32()]),
            expr_struct("S", [ty_i32()], [("x", expr_int("0"))]),
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_struct2() {
    let (a, msg) = err!(
        "struct S(x:i32);
         S(y=0);"
    );
    let b = program([
        stmt_struct("S", [], [("x", ty_i32())]),
        stmt_expr(expr_err()),
    ]);
    assert!(a == b, "{a}\n\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
            Error: Wrong fields provided. Found S(y), expected S(x)
               ╭─[test:2:1]
               │
             2 │ S(y=0);
               │ ┬
               │ ╰── Expected S(x) fields.
            ───╯"}
    );
}

#[test]
fn test_resolve_struct3() {
    let a = ok!("struct S;
                 var s: S = S;");
    let b = program([
        stmt_struct("S", [], []),
        stmt_var("s", ty("S"), expr_struct("S", [], [])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_struct4() {
    let a = ok!("struct S[T](x:T);
                 var s = S(x=0);");
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_hole(),
            expr_struct("S", [ty_hole()], [("x", expr_int("0"))]),
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_enum0() {
    let a = ok!("enum E[T] { A(T) }");
    let b = program([stmt_enum("E", ["T"], [("A", ty_gen("T"))])]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_enum1() {
    let a = ok!("enum E[T] { A(T) }
                 var e: E[i32] = E[i32]::A(0);");
    let b = program([
        stmt_enum("E", ["T"], [("A", ty_gen("T"))]),
        stmt_var(
            "e",
            ty_con("E", [ty_i32()]),
            expr_enum("E", [ty_i32()], "A", expr_int("0")),
        ),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_unordered0() {
    let a = ok!("def f(): i32 = g();
                 def g(): i32 = 0;");
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty_i32(), [], expr_int("0")),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_unordered1() {
    let a = ok!("def f(): i32 = g();
                 def g(): i32 = f();");
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty_i32(), [], expr_call(expr_def("f", []), [])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_unordered2() {
    let a = ok!("type A = B;
                 type B = A;");
    let b = program([
        stmt_type("A", [], ty_alias("B", [])),
        stmt_type("B", [], ty_alias("A", [])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_expr_assoc0() {
    let a = ok!("trait Trait {
                     def f(x: i32): i32;
                 }
                 Trait::f(0);");
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", ty_i32())], [], ty_i32())],
            [],
        ),
        stmt_expr(expr_call(expr_assoc("Trait", [], "f", []), [expr_int("0")])),
    ]);
    assert!(a == b, "{}", diff(&a, &b));
}

#[test]
fn test_resolve_expr_assoc1() {
    let a = ok!("trait Trait {
                     def f(x: i32): i32;
                 }
                 f(0);");
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", ty_i32())], [], ty_i32())],
            [],
        ),
        stmt_expr(expr_call(expr_assoc("Trait", [], "f", []), [expr_int("0")])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_expr_assoc2() {
    let a = ok!("trait Trait[A] {
                     def f(x: A): A;
                 }
                 Trait[i32]::f(0);");
    let b = program([
        stmt_trait(
            "Trait",
            ["A"],
            [],
            [tr_def("f", [], [("x", ty_gen("A"))], [], ty_gen("A"))],
            [],
        ),
        stmt_expr(expr_call(
            expr_assoc("Trait", [ty_i32()], "f", []),
            [expr_int("0")],
        )),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_type_assoc0() {
    let a = ok!("trait Trait {
                     type A;
                 }
                 type B = Trait::A;");
    let b = program([
        stmt_trait("Trait", [], [], [], [tr_type("A", [])]),
        stmt_type("B", [], ty_assoc("Trait", [], "A", [])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_type_assoc1() {
    let a = ok!("trait Trait[T] {
                     type A;
                 }
                 type B = Trait[i32]::A;");
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", [])]),
        stmt_type("B", [], ty_assoc("Trait", [ty_i32()], "A", [])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_resolve_type_assoc2() {
    let a = ok!("trait Trait[T] {
                     type A[U];
                 }
                 type B = Trait[i32]::A[i32];");
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", ["U"])]),
        stmt_type("B", [], ty_assoc("Trait", [ty_i32()], "A", [ty_i32()])),
    ]);
    assert!(a == b, "{a}\n\n{b}");
}
