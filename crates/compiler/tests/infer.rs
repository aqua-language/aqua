// use compiler::ast::Program;
//
// macro_rules! ok {
//     {$s:tt} => { { Program::try_infer(indoc::indoc! { $s }).unwrap_or_else(|(_, s)| panic!("{s}")) } }
// }
//
// macro_rules! ng {
//     {$s:tt} => { { Program::try_infer(indoc::indoc! { $s }).map(|v| panic!("Unexpected {v}")).unwrap_err() } }
// }
//
// macro_rules! ty {
//     {$s:tt} => { { Program::try_resolve(indoc::indoc! { $s }).unwrap_or_else(|(_, s)| panic!("{s}")) } }
// }
//
// #[test]
// fn test_infer_literal0() {
//     let a = ok!("1;");
//     let b = ty!("1:i32;");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def0() {
//     let a = ok!("
//         def f(): i32 = 0;
//         f();
//     ");
//     let b = ty!("
//         def f(): i32 = 0:i32;
//         (f:(fun():i32))():i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def1() {
//     let a = ok!("def f(x: i32): i32 = 0;");
//     let b = ty!("def f(x: i32): i32 = 0:i32;");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def2() {
//     let a = ok!("
//         def f(x: i32): i32 = x;
//         f(0);
//     ");
//     let b = ty!("
//         def f(x: i32): i32 = x:i32;
//         (f:(fun(i32):i32))(0:i32):i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def3() {
//     let (a, msg) = ng!("def f(x: i32): f32 = x;");
//     let b = ty!("def f(x: i32): f32 = x:i32;");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
//     assert_eq!(
//         msg,
//         indoc::indoc! {"
//             Error: Type mismatch
//                ╭─[test:1:1]
//                │
//              1 │ def f(x: i32): f32 = x;
//                │ ┬┬─
//                │ ╰──── Found i32
//                │  │
//                │  ╰─── Expected f32
//             ───╯"}
//     )
// }
//
// #[test]
// fn test_infer_def4() {
//     let a = ok!("def f[T](x: T): T = x;");
//     let b = ty!("def f[T](x: T): T = x:T;");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def5() {
//     let a = ok!("
//         def f[T](x: T): T = x;
//         f(0);
//     ");
//     let b = ty!("
//         def f[T](x: T): T = x:T;
//         (f[i32]:(fun(i32):i32))(0:i32):i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def6() {
//     let a = ok!("
//         def f[T](x: T): T = x;
//         f(0);
//         f(0.0);
//     ");
//     let b = ty!("
//         def f[T](x: T): T = x:T;
//         (f[i32]:(fun(i32):i32))(0:i32):i32;
//         (f[f32]:(fun(f32):f32))(0.0:f32):f32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def7() {
//     let a = ok!("
//         def f[T](x: T): T = x;
//         def g[T](x: T): T = f(x);
//         f(0);
//     ");
//     let b = ty!("
//         def f[T](x: T): T = x:T;
//         def g[T](x: T): T = (f[T]:fun(T):T)(x:T):T;
//         (f[i32]:(fun(i32):i32))(0:i32):i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_def8() {
//     let a = ok!("
//         def f[T](x: T): T = f(x);
//     ");
//     let b = ty!("
//         def f[T](x: T): T = (f[T]:fun(T):T)(x:T):T;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_struct0() {
//     let a = ok!("
//         struct Foo(x:i32);
//         Foo(x=0);
//     ");
//     let b = ty!("
//         struct Foo(x:i32);
//         Foo(x=(0:i32)):Foo;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_struct1() {
//     let a = ok!("
//         struct Foo[T](x:T);
//         Foo(x=0);
//     ");
//     let b = ty!("
//         struct Foo[T](x:T);
//         Foo[i32](x=(0:i32)):Foo[i32];
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_struct2() {
//     let a = ok!("
//         Foo(x=0);
//         struct Foo[T](x:T);
//     ");
//     let b = ty!("
//         Foo[i32](x=(0:i32)):Foo[i32];
//         struct Foo[T](x:T);
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_struct3() {
//     let a = ok!("
//         struct Foo[T](x:T);
//         var s = Foo(x=0);
//         s.x;
//     ");
//     let b = ty!("
//         struct Foo[T](x:T);
//         var s:Foo[i32] = Foo[i32](x=(0:i32)):Foo[i32];
//         ((s:Foo[i32]).x):i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_enum0() {
//     let a = ok!("
//         enum Foo { Bar(i32), Baz(f32) }
//         Foo::Bar(0);
//     ");
//     let b = ty!("
//         enum Foo { Bar(i32), Baz(f32) }
//         Foo::Bar(0:i32):Foo;
//     ");
//     assert!(a == b, "{a}\n\n{b}");
// }
//
// #[test]
// fn test_infer_enum1() {
//     let a = ok!("
//         enum Foo[T] { Bar(T), Baz(T) }
//         Foo::Bar(0);
//     ");
//     let b = ty!("
//         enum Foo[T] { Bar(T), Baz(T) }
//         Foo[i32]::Bar(0:i32):Foo[i32];
//     ");
//     assert!(a == b, "{a}\n\n{b}");
// }
//
// #[test]
// fn test_infer_impl0() {
//     let a = ok!("
//         trait Foo[T] { def f(x: T): T; }
//         impl Foo[i32] { def f(x: i32): i32 = x; }
//         f(0);
//     ");
//     let b = ty!("
//         trait Foo[T] { def f(x: T): T; }
//         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
//         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_impl1() {
//     let (_, msg) = ng!("
//         trait Foo[T] { def f(x: T): T; }
//         impl Foo[f32] { def f(x: i32): i32 = x; }
//         f(0);");
//     assert_eq!(
//         msg,
//         indoc::indoc! {"
//             Error: Unsolved goal
//                ╭─[test:1:7]
//                │
//              1 │ trait Foo[T] { def f(x: T): T; }
//                │       ─┬─
//                │        ╰─── Could not solve goal
//             ───╯"}
//     );
// }
//
// #[test]
// fn test_infer_impl2() {
//     let a = ok!("
//         trait Foo[T] {}
//         def f[T](x: T): T where Foo[T] = x;
//     ");
//     let b = ty!("
//         trait Foo[T] {}
//         def f[T](x: T): T where Foo[T] = x:T;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_impl3() {
//     let a = ok!("
//         trait Foo[T] { def f(x: T): T; }
//         def g[T](x: T): T where Foo[T] = g(x);
//     ");
//     let b = ty!("
//         trait Foo[T] { def f(x: T): T; }
//         def g[T](x: T): T where Foo[T] = (g[T]:fun(T):T)(x:T):T;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_impl4() {
//     let a = ok!("
//         trait Foo[T] { def f(x: T): T; }
//         impl[T] Foo[T] { def f(x: T): T = x; }
//     ");
//     let b = ty!("
//         trait Foo[T] { def f(x: T): T; }
//         impl[T] Foo[T] { def f(x: T): T = x:T; }
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_impl5() {
//     let a = ok!("
//         trait Foo[T] { def f(x: T): T; }
//         def g[T](x: T): T where Foo[T] = f(x);
//     ");
//     let b = ty!("
//         trait Foo[T] { def f(x: T): T; }
//         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;
//     ");
//     assert!(a == b, "{}\n\n{}", a.display_types(), b.display_types());
// }
//
// #[test]
// fn test_infer_impl6() {
//     let a = ok!("
//         trait Foo[T] { def f(x: T): T; }
//         impl Foo[i32] { def f(x: i32): i32 = x; }
//         def g[T](x: T): T where Foo[T] = Foo[T]::f(x);
//     ");
//     let b = ty!("
//         trait Foo[T] { def f(x: T): T; }
//         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
//         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;
//     ");
//     assert_eq!(a, b, "{a}\n\n{b}");
// }
//
// // #[test]
// // fn test_infer_add2() {
// //     let a = ok!("1 + 1;");
// //     let b = ty!("Add[i32](1:i32, 1:i32):i32");
// //     assert_eq!(a, b, "{a}\n\n{b}");
// // }
