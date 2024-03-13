use compiler::ast::Program;
use compiler::check;
use compiler::dsl::bound;
use compiler::dsl::bound_err;
use compiler::dsl::expr_assoc;
use compiler::dsl::expr_call;
use compiler::dsl::expr_def;
use compiler::dsl::expr_enum;
use compiler::dsl::expr_err;
use compiler::dsl::expr_int;
use compiler::dsl::expr_struct;
use compiler::dsl::expr_tuple;
use compiler::dsl::expr_var;
use compiler::dsl::program;
use compiler::dsl::stmt_def;
use compiler::dsl::stmt_enum;
use compiler::dsl::stmt_expr;
use compiler::dsl::stmt_impl;
use compiler::dsl::stmt_struct;
use compiler::dsl::stmt_trait;
use compiler::dsl::stmt_type;
use compiler::dsl::stmt_var;
use compiler::dsl::tr_def;
use compiler::dsl::tr_type;
use compiler::dsl::ty;
use compiler::dsl::ty_alias;
use compiler::dsl::ty_assoc;
use compiler::dsl::ty_con;
use compiler::dsl::ty_err;
use compiler::dsl::ty_gen;
use compiler::dsl::ty_hole;
use compiler::dsl::ty_tuple;
use compiler::dsl::types::ty_i32;

#[test]
fn test_resolve_var0() {
    let a = Program::resolve("var x = 0; x;").unwrap();
    let b = program([
        stmt_var("x", ty_hole(), expr_int("0")),
        stmt_expr(expr_var("x")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_var_err0() {
    let a = Program::resolve("var x = 0; y;").unwrap_err();
    let b = program([
        stmt_var("x", ty_hole(), expr_int("0")),
        stmt_expr(expr_err()),
    ]);
    check!(
        a,
        b,
        "Error: Name `y` not found.
            ╭─[test:1:12]
            │
          1 │ var x = 0; y;
            │            ┬
            │            ╰── Expected expression.
         ───╯"
    );
}

#[test]
fn test_resolve_def0() {
    let a = Program::resolve("def f(): i32 = 0; f();").unwrap();
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_int("0")),
        stmt_expr(expr_call(expr_def("f", []), [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_def1() {
    let a = Program::resolve("def f(x: i32): i32 = x;").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty_i32())],
        ty_i32(),
        [],
        expr_var("x"),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def2() {
    let a = Program::resolve("def f(x: i32): i32 = f(x);").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty_i32())],
        ty_i32(),
        [],
        expr_call(expr_def("f", []), [expr_var("x")]),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def_param1() {
    let a = Program::resolve("def f(): i32 = x;").unwrap_err();
    let b = program([stmt_def("f", [], [], ty_i32(), [], expr_err())]);
    check!(
        a,
        b,
        "Error: Name `x` not found.
            ╭─[test:1:16]
            │
          1 │ def f(): i32 = x;
            │                ┬
            │                ╰── Expected expression.
         ───╯"
    );
}

#[test]
fn test_resolve_def_generic() {
    let a = Program::resolve("def f[T](x: T): T = x;").unwrap();
    let b = program([stmt_def(
        "f",
        ["T"],
        [("x", ty_gen("T"))],
        ty_gen("T"),
        [],
        expr_var("x"),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_type0() {
    let a = Program::resolve(
        "type T = i32;
         var x: T = 0;",
    )
    .unwrap();
    let b = program([
        stmt_type("T", [], ty_i32()),
        stmt_var("x", ty_alias("T", []), expr_int("0")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type1() {
    let a = Program::resolve(
        "type T[U] = (i32, U);
         var x: T[i32] = (0, 0);",
    )
    .unwrap();
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty_i32(), ty_gen("U")])),
        stmt_var(
            "x",
            ty_alias("T", [ty_i32()]),
            expr_tuple([expr_int("0"), expr_int("0")]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type2() {
    let a = Program::resolve(
        "type T[U] = (i32, U);
         var x: T[i32, i32] = (0, 0);",
    )
    .unwrap_err();
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty_i32(), ty_gen("U")])),
        stmt_var("x", ty_err(), expr_tuple([expr_int("0"), expr_int("0")])),
    ]);
    check!(
        a,
        b,
        "Error: Wrong number of type arguments. Found 2, expected 1
            ╭─[test:2:8]
            │
          2 │ var x: T[i32, i32] = (0, 0);
            │        ┬
            │        ╰── Expected 1 arguments.
         ───╯"
    );
}

#[test]
fn test_resolve_trait0() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(T): T;
         }",
    )
    .unwrap();
    let b = program([stmt_trait(
        "Trait",
        ["T"],
        [],
        [tr_def("f", [], [ty_gen("T")], [], ty_gen("T"))],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_trait_assoc0() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(T): T;
         }
         def g[T](x:T): T where Trait[T] = f(x);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [ty_gen("T")], [], ty_gen("T"))],
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
    check!(a, b);
}

#[test]
fn test_resolve_trait_impl0() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(T): T;
         }
         impl Trait[i32] {
             def f(x:i32): i32 = x;
         }",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [ty_gen("T")], [], ty_gen("T"))],
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
    check!(a, b);
}

#[ignore]
#[test]
fn test_resolve_trait_impl1() {
    let a = Program::resolve(
        "trait Trait[T] {
             type A[U];
         }
         impl Trait[i32] {
             type A[U] = U;
         }",
    )
    .unwrap();
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
    check!(a, b);
}

#[test]
fn test_resolve_trait_impl2() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(T): T;
         }
         impl Trait[i32] {
             def g(x: i32): i32 = x;
         }",
    )
    .unwrap_err();
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [ty_gen("T")], [], ty_gen("T"))],
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
    check!(
        a,
        b,
        "Error: Wrong defs implemented for Trait. Found { `g` }, expected { `f` }
            ╭─[test:4:6]
            │
          4 │ impl Trait[i32] {
            │      ──┬──
            │        ╰──── Expected { `f` }.
         ───╯"
    );
}

#[test]
fn test_resolve_struct0() {
    let a = Program::resolve("struct S[T](x:T);").unwrap();
    let b = program([stmt_struct("S", ["T"], [("x", ty_gen("T"))])]);
    check!(a, b);
}

#[test]
fn test_resolve_struct1() {
    let a = Program::resolve("struct S[T](x:T); var s: S[i32] = S[i32](x=0);").unwrap();
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_con("S", [ty_i32()]),
            expr_struct("S", [ty_i32()], [("x", expr_int("0"))]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_struct2() {
    let a = Program::resolve("struct S(x:i32); S(y=0);").unwrap_err();
    let b = program([
        stmt_struct("S", [], [("x", ty_i32())]),
        stmt_expr(expr_err()),
    ]);
    check!(
        a,
        b,
        "Error: Wrong fields provided. Found S(y), expected S(x)
            ╭─[test:2:1]
            │
          2 │ S(y=0);
            │ ┬
            │ ╰── Expected S(x) fields.
         ───╯"
    );
}

#[test]
fn test_resolve_struct3() {
    let a = Program::resolve("struct S; var s: S = S;").unwrap();
    let b = program([
        stmt_struct("S", [], []),
        stmt_var("s", ty("S"), expr_struct("S", [], [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_struct4() {
    let a = Program::resolve(
        "struct S[T](x:T);
         var s = S(x=0);",
    )
    .unwrap();
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_hole(),
            expr_struct("S", [ty_hole()], [("x", expr_int("0"))]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_enum0() {
    let a = Program::resolve("enum E[T] { A(T) }").unwrap();
    let b = program([stmt_enum("E", ["T"], [("A", ty_gen("T"))])]);
    check!(a, b);
}

#[test]
fn test_resolve_enum1() {
    let a = Program::resolve("enum E[T] { A(T) } var e: E[i32] = E[i32]::A(0);").unwrap();
    let b = program([
        stmt_enum("E", ["T"], [("A", ty_gen("T"))]),
        stmt_var(
            "e",
            ty_con("E", [ty_i32()]),
            expr_enum("E", [ty_i32()], "A", expr_int("0")),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered0() {
    let a = Program::resolve("def f(): i32 = g(); def g(): i32 = 0;").unwrap();
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty_i32(), [], expr_int("0")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered1() {
    let a = Program::resolve("def f(): i32 = g(); def g(): i32 = f();").unwrap();
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty_i32(), [], expr_call(expr_def("f", []), [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered2() {
    let a = Program::resolve(
        "type A = B;
         type B = A;",
    )
    .unwrap();
    let b = program([
        stmt_type("A", [], ty_alias("B", [])),
        stmt_type("B", [], ty_alias("A", [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc0() {
    let a = Program::resolve(
        "trait Trait {
             def f(i32): i32;
         }
         Trait::f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [ty_i32()], [], ty_i32())],
            [],
        ),
        stmt_expr(expr_call(expr_assoc("Trait", [], "f", []), [expr_int("0")])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc1() {
    let a = Program::resolve(
        "trait Trait {
             def f(i32): i32;
         }
         f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [ty_i32()], [], ty_i32())],
            [],
        ),
        stmt_expr(expr_call(expr_assoc("Trait", [], "f", []), [expr_int("0")])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc2() {
    let a = Program::resolve(
        "trait Trait[A] {
             def f(A): A;
         }
         Trait[i32]::f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            ["A"],
            [],
            [tr_def("f", [], [ty_gen("A")], [], ty_gen("A"))],
            [],
        ),
        stmt_expr(expr_call(
            expr_assoc("Trait", [ty_i32()], "f", []),
            [expr_int("0")],
        )),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc0() {
    let a = Program::resolve(
        "trait Trait {
             type A;
         }
         type B = Trait::A;",
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", [], [], [], [tr_type("A", [])]),
        stmt_type("B", [], ty_assoc("Trait", [], "A", [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc1() {
    let a = Program::resolve(
        "trait Trait[T] {
             type A;
         }
         type B = Trait[i32]::A;",
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", [])]),
        stmt_type("B", [], ty_assoc("Trait", [ty_i32()], "A", [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc2() {
    let a = Program::resolve(
        "trait Trait[T] {
             type A[U];
         }
         type B = Trait[i32]::A[i32];",
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", ["U"])]),
        stmt_type("B", [], ty_assoc("Trait", [ty_i32()], "A", [ty_i32()])),
    ]);
    check!(a, b);
}
