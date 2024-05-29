mod common;

use common::bound_err;
use common::expr_assoc;
use common::expr_block;
use common::expr_call;
use common::expr_def;
use common::expr_enum;
use common::expr_err;
use common::expr_int;
use common::expr_struct;
use common::expr_tuple;
use common::expr_unit;
use common::expr_var;
use common::program;
use common::stmt_def;
use common::stmt_enum;
use common::stmt_expr;
use common::stmt_impl;
use common::stmt_struct;
use common::stmt_trait;
use common::stmt_type;
use common::stmt_var;
use common::tr_def;
use common::tr_type;
use common::trait_bound;
use common::ty;
use common::ty_alias;
use common::ty_assoc;
use common::ty_con;
use common::ty_gen;
use common::ty_tuple;
use compiler::ast::Program;
use compiler::ast::Type;

use crate::common::expr_unresolved;
use crate::common::type_bound;

#[test]
fn test_resolve_var0() {
    let a = Program::resolve("var x = 0; x;").unwrap();
    let b = program([
        stmt_var("x", Type::hole(), expr_int("0")),
        stmt_expr(expr_var("x")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_var_err0() {
    let a = Program::resolve("var x = 0; y;").unwrap();
    let b = program([
        stmt_var("x", Type::hole(), expr_int("0")),
        stmt_expr(expr_unresolved("y", [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_def0() {
    let a = Program::resolve("def f(): i32 = 0; f();").unwrap();
    let b = program([
        stmt_def("f", [], [], Type::i32(), [], expr_int("0")),
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
        [("x", Type::i32())],
        Type::i32(),
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
        [("x", Type::i32())],
        Type::i32(),
        [],
        expr_call(expr_def("f", []), [expr_var("x")]),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def3() {
    let a = Program::resolve(
        "def f(x: i32): i32 = {
             def g(y: i32): i32 = 1;
             g(x)
         }",
    )
    .unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", Type::i32())],
        Type::i32(),
        [],
        expr_block(
            [stmt_def(
                "g",
                [],
                [("y", Type::i32())],
                Type::i32(),
                [],
                expr_int("1"),
            )],
            expr_call(expr_def("g", []), [expr_var("x")]),
        ),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def_param1() {
    let a = Program::resolve("def f(): i32 = x;").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [],
        Type::i32(),
        [],
        expr_unresolved("x", []),
    )]);
    check!(a, b);
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
        stmt_type("T", [], Type::i32()),
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
        stmt_type("T", ["U"], ty_tuple([Type::i32(), ty_gen("U")])),
        stmt_var(
            "x",
            ty_alias("T", [Type::i32()]),
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
        stmt_type("T", ["U"], ty_tuple([Type::i32(), ty_gen("U")])),
        stmt_var("x", Type::err(), expr_tuple([expr_int("0"), expr_int("0")])),
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
             def f(x:T): T;
         }",
    )
    .unwrap();
    let b = program([stmt_trait(
        "Trait",
        ["T"],
        [],
        [tr_def("f", [], [("x", ty_gen("T"))], ty_gen("T"), [])],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_trait_assoc0() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(x:T): T;
         }
         def g[T](x:T): T where Trait[T] = f(x);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [("x", ty_gen("T"))], ty_gen("T"), [])],
            [],
        ),
        stmt_def(
            "g",
            ["T"],
            [("x", ty_gen("T"))],
            ty_gen("T"),
            [trait_bound("Trait", [ty_gen("T")], [])],
            expr_call(expr_unresolved("f", []), [expr_var("x")]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_trait_impl0() {
    let a = Program::resolve(
        "trait Trait[T] {
             def f(x:T): T;
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
            [tr_def("f", [], [("x", ty_gen("T"))], ty_gen("T"), [])],
            [],
        ),
        stmt_impl(
            [],
            trait_bound("Trait", [Type::i32()], []),
            [],
            [stmt_def(
                "f",
                [],
                [("x", Type::i32())],
                Type::i32(),
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
            trait_bound("Trait", [Type::i32()], []),
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
             def f(x:T): T;
         }
         impl Trait[i32] {
             def g(x:i32): i32 = x;
         }",
    )
    .unwrap_err();
    let b = program([
        stmt_trait(
            "Trait",
            ["T"],
            [],
            [tr_def("f", [], [("x", ty_gen("T"))], ty_gen("T"), [])],
            [],
        ),
        stmt_impl(
            [],
            bound_err(),
            [],
            [stmt_def(
                "g",
                [],
                [("x", Type::i32())],
                Type::i32(),
                [],
                expr_var("x"),
            )],
            [],
        ),
    ]);
    check!(
        a,
        b,
        "Error: Wrong defs implemented for Trait. Found { g }, expected { f }
            ╭─[test:4:6]
            │
          4 │ impl Trait[i32] {
            │      ──┬──
            │        ╰──── Expected { f }.
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
    let a = Program::resolve(
        "struct S[T](x:T);
         var s: S[i32] = S[i32](x=0);",
    )
    .unwrap();
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_con("S", [Type::i32()]),
            expr_struct("S", [Type::i32()], [("x", expr_int("0"))]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_struct2() {
    let a = Program::resolve(
        "struct S(x:i32);
         S(y=0);",
    )
    .unwrap_err();
    let b = program([
        stmt_struct("S", [], [("x", Type::i32())]),
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
    let a = Program::resolve(
        "struct S;
         var s: S = S;",
    )
    .unwrap();
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
            Type::hole(),
            expr_struct("S", [Type::hole()], [("x", expr_int("0"))]),
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
    let a = Program::resolve(
        "enum E[T] { A(T) }
         var e: E[i32] = E[i32]::A(0);",
    )
    .unwrap();
    let b = program([
        stmt_enum("E", ["T"], [("A", ty_gen("T"))]),
        stmt_var(
            "e",
            ty_con("E", [Type::i32()]),
            expr_enum("E", [Type::i32()], "A", expr_int("0")),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered0() {
    let a = Program::resolve(
        "def f(): i32 = g();
         def g(): i32 = 0;",
    )
    .unwrap();
    let b = program([
        stmt_def(
            "f",
            [],
            [],
            Type::i32(),
            [],
            expr_call(expr_def("g", []), []),
        ),
        stmt_def("g", [], [], Type::i32(), [], expr_int("0")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered1() {
    let a = Program::resolve(
        "def f(): i32 = g();
         def g(): i32 = f();",
    )
    .unwrap();
    let b = program([
        stmt_def(
            "f",
            [],
            [],
            Type::i32(),
            [],
            expr_call(expr_def("g", []), []),
        ),
        stmt_def(
            "g",
            [],
            [],
            Type::i32(),
            [],
            expr_call(expr_def("f", []), []),
        ),
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
             def f(x:i32): i32;
         }
         Trait::f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", Type::i32())], Type::i32(), [])],
            [],
        ),
        stmt_expr(expr_call(
            expr_assoc(trait_bound("Trait", [], []), "f", []),
            [expr_int("0")],
        )),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc1() {
    let a = Program::resolve(
        "trait Trait {
             def f(x:i32): i32;
         }
         f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", Type::i32())], Type::i32(), [])],
            [],
        ),
        stmt_expr(expr_call(expr_unresolved("f", []), [expr_int("0")])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc2() {
    let a = Program::resolve(
        "trait Trait[A] {
             def f(x:A): A;
         }
         Trait[i32]::f(0);",
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            ["A"],
            [],
            [tr_def("f", [], [("x", ty_gen("A"))], ty_gen("A"), [])],
            [],
        ),
        stmt_expr(expr_call(
            expr_assoc(trait_bound("Trait", [Type::i32()], []), "f", []),
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
        stmt_type(
            "B",
            [],
            ty_assoc("Trait", [], [("A", Type::hole())], "A", []),
        ),
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
        stmt_type(
            "B",
            [],
            ty_assoc("Trait", [Type::i32()], [("A", Type::hole())], "A", []),
        ),
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
        stmt_type(
            "B",
            [],
            ty_assoc(
                "Trait",
                [Type::i32()],
                [("A", Type::hole())],
                "A",
                [Type::i32()],
            ),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_impl() {
    let a = Program::resolve(
        "impl i32 {
             def zero(): i32 = 0;
         }",
    )
    .unwrap();
    let b = program([stmt_impl(
        [],
        type_bound(ty("i32")),
        [],
        [stmt_def("zero", [], [], Type::i32(), [], expr_int("0"))],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_type_impl2() {
    let a = Program::resolve(
        "impl[T] Vec[T] {
             def push(v:Vec[T], x:T): () = ();
         }",
    )
    .unwrap();
    let b = program([stmt_impl(
        ["T"],
        type_bound(ty_con("Vec", [ty_gen("T")])),
        [],
        [stmt_def(
            "push",
            [],
            [("v", ty_con("Vec", [ty_gen("T")])), ("x", ty_gen("T"))],
            Type::unit(),
            [],
            expr_unit(),
        )],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_i32_abs() {
    let a = Program::resolve("1.abs();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_unresolved("abs", []),
        [expr_int("1")],
    ))]);
    check!(a, b);
}

#[test]
fn test_resolve_vec_new1() {
    let a = Program::resolve("Vec::new();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_assoc(type_bound(ty_con("Vec", [Type::Hole])), "new", []),
        [],
    ))]);
    check!(a, b);
}

#[test]
fn test_resolve_vec_new2() {
    let a = Program::resolve("Vec[i32]::new();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_assoc(type_bound(ty_con("Vec", [Type::i32()])), "new", []),
        [],
    ))]);
    check!(a, b);
}
