#[macro_use]
mod common;

use common::dsl::bound_err;
use common::dsl::expr_assoc;
use common::dsl::expr_block;
use common::dsl::expr_call;
use common::dsl::expr_def;
use common::dsl::expr_enum;
use common::dsl::expr_err;
use common::dsl::expr_int;
use common::dsl::expr_struct;
use common::dsl::expr_tuple;
use common::dsl::expr_unit;
use common::dsl::expr_unresolved;
use common::dsl::expr_var;
use common::dsl::program;
use common::dsl::stmt_def;
use common::dsl::stmt_enum;
use common::dsl::stmt_expr;
use common::dsl::stmt_impl;
use common::dsl::stmt_struct;
use common::dsl::stmt_trait;
use common::dsl::stmt_type;
use common::dsl::stmt_var;
use common::dsl::tr_def;
use common::dsl::tr_type;
use common::dsl::trait_bound;
use common::dsl::ty;
use common::dsl::ty_alias;
use common::dsl::ty_assoc;
use common::dsl::ty_con;
use common::dsl::ty_gen;
use common::dsl::ty_tuple;
use common::dsl::type_bound;

use compiler::ast::Type;

#[test]
fn test_resolve_var0() {
    let a = resolve_program!("var x = 0; x;").unwrap();
    let b = program([
        stmt_var("x", Type::Unknown, expr_int("0")),
        stmt_expr(expr_var("x")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_var_err0() {
    let a = resolve_program!("var x = 0; y;").unwrap();
    let b = program([
        stmt_var("x", Type::Unknown, expr_int("0")),
        stmt_expr(expr_unresolved("y", [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_def0() {
    let a = resolve_program!("def f(): i32 = 0; f();").unwrap();
    let b = program([
        stmt_def("f", [], [], ty("i32"), [], expr_int("0")),
        stmt_expr(expr_call(expr_def("f", []), [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_def1() {
    let a = resolve_program!("def f(x: i32): i32 = x;").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def2() {
    let a = resolve_program!("def f(x: i32): i32 = f(x);").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_call(expr_def("f", []), [expr_var("x")]),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def3() {
    let a = resolve_program!(
        "def f(x: i32): i32 = {
             def g(y: i32): i32 = 1;
             g(x)
         }"
    )
    .unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_block(
            [stmt_def(
                "g",
                [],
                [("y", ty("i32"))],
                ty("i32"),
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
    let a = resolve_program!("def f(): i32 = x;").unwrap();
    let b = program([stmt_def(
        "f",
        [],
        [],
        ty("i32"),
        [],
        expr_unresolved("x", []),
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_def_generic() {
    let a = resolve_program!("def f[T](x: T): T = x;").unwrap();
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
    let a = resolve_program!(
        "type T = i32;
         var x: T = 0;"
    )
    .unwrap();
    let b = program([
        stmt_type("T", [], ty("i32")),
        stmt_var("x", ty_alias("T", []), expr_int("0")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type1() {
    let a = resolve_program!(
        "type T[U] = (i32, U);
         var x: T[i32] = (0, 0);"
    )
    .unwrap();
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty("i32"), ty_gen("U")])),
        stmt_var(
            "x",
            ty_alias("T", [ty("i32")]),
            expr_tuple([expr_int("0"), expr_int("0")]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type2() {
    let a = resolve_program!(
        "type T[U] = (i32, U);
         var x: T[i32, i32] = (0, 0);"
    )
    .unwrap_err();
    let b = program([
        stmt_type("T", ["U"], ty_tuple([ty("i32"), ty_gen("U")])),
        stmt_var("x", Type::Err, expr_tuple([expr_int("0"), expr_int("0")])),
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
    let a = resolve_program!(
        "trait Trait[T] {
             def f(x:T): T;
         }"
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
    let a = resolve_program!(
        "trait Trait[T] {
             def f(x:T): T;
         }
         def g[T](x:T): T where Trait[T] = f(x);"
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
    let a = resolve_program!(
        "trait Trait[T] {
             def f(x:T): T;
         }
         impl Trait[i32] {
             def f(x:i32): i32 = x;
         }"
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
            trait_bound("Trait", [ty("i32")], []),
            [],
            [stmt_def(
                "f",
                [],
                [("x", ty("i32"))],
                ty("i32"),
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
    let a = resolve_program!(
        "trait Trait[T] {
             type A[U];
         }
         impl Trait[i32] {
             type A[U] = U;
         }"
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("f", [])]),
        stmt_impl(
            [],
            trait_bound("Trait", [ty("i32")], []),
            [],
            [],
            [stmt_type("A", ["U"], ty_gen("U"))],
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_trait_impl2() {
    let a = resolve_program!(
        "trait Trait[T] {
             def f(x:T): T;
         }
         impl Trait[i32] {
             def g(x:i32): i32 = x;
         }"
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
                [("x", ty("i32"))],
                ty("i32"),
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
    let a = resolve_program!("struct S[T](x:T);").unwrap();
    let b = program([stmt_struct("S", ["T"], [("x", ty_gen("T"))])]);
    check!(a, b);
}

#[test]
fn test_resolve_struct1() {
    let a = resolve_program!(
        "struct S[T](x:T);
         var s: S[i32] = S[i32](x=0);"
    )
    .unwrap();
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            ty_con("S", [ty("i32")]),
            expr_struct("S", [ty("i32")], [("x", expr_int("0"))]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_struct2() {
    let a = resolve_program!(
        "struct S(x:i32);
         S(y=0);"
    )
    .unwrap_err();
    let b = program([
        stmt_struct("S", [], [("x", ty("i32"))]),
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
    let a = resolve_program!(
        "struct S;
         var s: S = S;"
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
    let a = resolve_program!(
        "struct S[T](x:T);
         var s = S(x=0);"
    )
    .unwrap();
    let b = program([
        stmt_struct("S", ["T"], [("x", ty_gen("T"))]),
        stmt_var(
            "s",
            Type::Unknown,
            expr_struct("S", [Type::Unknown], [("x", expr_int("0"))]),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_enum0() {
    let a = resolve_program!("enum E[T] { A(T) }").unwrap();
    let b = program([stmt_enum("E", ["T"], [("A", ty_gen("T"))])]);
    check!(a, b);
}

#[test]
fn test_resolve_enum1() {
    let a = resolve_program!(
        "enum E[T] { A(T) }
         var e: E[i32] = E[i32]::A(0);"
    )
    .unwrap();
    let b = program([
        stmt_enum("E", ["T"], [("A", ty_gen("T"))]),
        stmt_var(
            "e",
            ty_con("E", [ty("i32")]),
            expr_enum("E", [ty("i32")], "A", expr_int("0")),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered0() {
    let a = resolve_program!(
        "def f(): i32 = g();
         def g(): i32 = 0;"
    )
    .unwrap();
    let b = program([
        stmt_def("f", [], [], ty("i32"), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty("i32"), [], expr_int("0")),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered1() {
    let a = resolve_program!(
        "def f(): i32 = g();
         def g(): i32 = f();"
    )
    .unwrap();
    let b = program([
        stmt_def("f", [], [], ty("i32"), [], expr_call(expr_def("g", []), [])),
        stmt_def("g", [], [], ty("i32"), [], expr_call(expr_def("f", []), [])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_unordered2() {
    let a = resolve_program!(
        "type A = B;
         type B = A;"
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
    let a = resolve_program!(
        "trait Trait {
             def f(x:i32): i32;
         }
         Trait::f(0);"
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", ty("i32"))], ty("i32"), [])],
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
    let a = resolve_program!(
        "trait Trait {
             def f(x:i32): i32;
         }
         f(0);"
    )
    .unwrap();
    let b = program([
        stmt_trait(
            "Trait",
            [],
            [],
            [tr_def("f", [], [("x", ty("i32"))], ty("i32"), [])],
            [],
        ),
        stmt_expr(expr_call(expr_unresolved("f", []), [expr_int("0")])),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_expr_assoc2() {
    let a = resolve_program!(
        "trait Trait[A] {
             def f(x:A): A;
         }
         Trait[i32]::f(0);"
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
            expr_assoc(trait_bound("Trait", [ty("i32")], []), "f", []),
            [expr_int("0")],
        )),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc0() {
    let a = resolve_program!(
        "trait Trait {
             type A;
         }
         type B = Trait::A;"
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", [], [], [], [tr_type("A", [])]),
        stmt_type(
            "B",
            [],
            ty_assoc("Trait", [], [("A", Type::Unknown)], "A", []),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc1() {
    let a = resolve_program!(
        "trait Trait[T] {
             type A;
         }
         type B = Trait[i32]::A;"
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", [])]),
        stmt_type(
            "B",
            [],
            ty_assoc("Trait", [ty("i32")], [("A", Type::Unknown)], "A", []),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_assoc2() {
    let a = resolve_program!(
        "trait Trait[T] {
             type A[U];
         }
         type B = Trait[i32]::A[i32];"
    )
    .unwrap();
    let b = program([
        stmt_trait("Trait", ["T"], [], [], [tr_type("A", ["U"])]),
        stmt_type(
            "B",
            [],
            ty_assoc(
                "Trait",
                [ty("i32")],
                [("A", Type::Unknown)],
                "A",
                [ty("i32")],
            ),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_resolve_type_impl() {
    let a = resolve_program!(
        "impl i32 {
             def zero(): i32 = 0;
         }"
    )
    .unwrap();
    let b = program([stmt_impl(
        [],
        type_bound(ty("i32")),
        [],
        [stmt_def("zero", [], [], ty("i32"), [], expr_int("0"))],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_type_impl2() {
    let a = resolve_program!(
        "impl[T] Vec[T] {
             def push(v:Vec[T], x:T): () = ();
         }"
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
            Type::Tuple(vec![]),
            [],
            expr_unit(),
        )],
        [],
    )]);
    check!(a, b);
}

#[test]
fn test_resolve_i32_abs() {
    let a = resolve_program!("1.abs();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_unresolved("abs", []),
        [expr_int("1")],
    ))]);
    check!(a, b);
}

#[test]
fn test_resolve_vec_new1() {
    let a = resolve_program!("Vec::new();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_assoc(type_bound(ty_con("Vec", [Type::Unknown])), "new", []),
        [],
    ))]);
    check!(a, b);
}

#[test]
fn test_resolve_vec_new2() {
    let a = resolve_program!("Vec[i32]::new();").unwrap();
    let b = program([stmt_expr(expr_call(
        expr_assoc(type_bound(ty_con("Vec", [ty("i32")])), "new", []),
        [],
    ))]);
    check!(a, b);
}
