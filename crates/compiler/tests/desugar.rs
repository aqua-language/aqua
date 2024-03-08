

// #[test]
// fn test_parser_expr_if1() {
//     let a = Expr::parse2("if true { 1 } if false { 2 }");
//     let b = program([
//         stmt_expr(expr_match(
//             expr_bool(true),
//             [(pat_bool(true), expr_int("1")), (pat_wild(), expr_unit())],
//         )),
//         stmt_expr(expr_match(
//             expr_bool(false),
//             [(pat_bool(true), expr_int("2")), (pat_wild(), expr_unit())],
//         )),
//     ]);
//     check!(a, b);
// }

// #[test]
// fn test_parser_expr_if_else1() {
//     let a = Expr::parse2("if true { 1; 2 } else { 3; 4 }");
//     let b = expr_if_else(
//         expr_bool(true),
//         [
//             (
//                 pat_bool(true),
//                 expr_block([stmt_expr(expr_int("1"))], expr_int("2")),
//             ),
//             (
//                 pat_wild(),
//                 expr_block([stmt_expr(expr_int("3"))], expr_int("4")),
//             ),
//         ],
//     );
//     check!(a, b);
// }

// #[test]
// fn test_parser_expr_if_else1() {
//     let a = Expr::parse2("if true { 1; 2 } else { 3; 4 }");
//     let b = expr_if_else(
//         expr_bool(true),
//         [
//             (
//                 pat_bool(true),
//                 expr_block([stmt_expr(expr_int("1"))], expr_int("2")),
//             ),
//             (
//                 pat_wild(),
//                 expr_block([stmt_expr(expr_int("3"))], expr_int("4")),
//             ),
//         ],
//     );
//     check!(a, b);
// }
