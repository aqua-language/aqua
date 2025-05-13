use compiler::aqua;

use crate::common::passes::querycomp;

#[macro_use]
mod common;

#[test]
fn test_desugar_query_from0() {
    let a = querycomp(aqua!("from x in e;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x));")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_where1() {
    let a = querycomp(aqua!("from x in e where x;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x)).filter(r => r.x);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_from0() {
    let a = querycomp(aqua!("from x0 in e0 from x1 in e1;")).unwrap();
    let b = querycomp(aqua!(
        "e0.map[_](x0 => record(x0))
           .flatMap[_](r => e1.map[_](x1 => record(x1 = x1, x0 = r.x0)));"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_select() {
    let a = querycomp(aqua!("from x in e select y=f(x);")).unwrap();
    let b = querycomp(aqua!(
        "e.map[_](x => record(x = x))
          .map[_](r => record(y = f(r.x)));"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_group() {
    let a = querycomp(aqua!(
        "from x in e0
         group k=x
            over Window::tumbling(1min)
            compute a = sum of x, b = count of x;"
    ))
    .unwrap();
    let b = querycomp(aqua!(
        "e0.map[_](x => record(x = x))
           .keyBy[_](r => r.x)
           .window[_](
               Window::tumbling(1min),
               (k, r) => record(
                   k = k,
                   a = r.map[_](r => r.x).sum(),
                   b = r.map[_](r => r.x).count()
               )
           );"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_join() {
    let a = querycomp(aqua!(
        "from x in e0
         join y in e1 on x.a == y.b;"
    ))
    .unwrap();
    let b = querycomp(aqua!(
        "e0.map[_](x => record(x = x))
           .flatMap[_](r => e1.filter(y => r.x.a == y.b)
                               .map[_](y => record(y = y, x = r.x)));"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_over_compute() {
    let a = querycomp(aqua!(
        "from x in e
         over tumbling(1min)
         compute a = sum of x,
                 b = count of x;"
    ))
    .unwrap();
    let b = querycomp(aqua!(
        "e.map[_](x => record(x = x))
          .window[_](
              tumbling(1min),
              r => record(
                  a = r.map[_](r => r.x).sum(),
                  b = r.map[_](r => r.x).count()
              )
          );"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_into() {
    let a = querycomp(aqua!("from x in e into sink();")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).sink();")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_into_run() {
    let a = querycomp(aqua!("from x in e into sink().run();")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).sink().run();")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_cross() {
    let a = querycomp(aqua!("from x in e0 cross y in e1;")).unwrap();
    let b = querycomp(aqua!(
        "e0.map[_](x => record(x = x))
           .flatMap[_](r => e1.map[_](y => record(y = y, x = r.x)));"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_order() {
    let a = querycomp(aqua!("from x in e order x;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).sortBy[_](r => r.x);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_distinct() {
    let a = querycomp(aqua!("from x in e distinct x;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).unique();")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_compute() {
    let a = querycomp(aqua!("from x in e compute y = Max of x;")).unwrap();
    let b = querycomp(aqua!(
        "e.map[_](x => record(x = x))
          .fold[_](Max::merge, Max::identity);"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_group_compute() {
    let a = querycomp(aqua!(
        "from x in e
         group k = x
            over Window::tumbling(1min)
            compute y = max of x;"
    ))
    .unwrap();
    let b = querycomp(aqua!(
        "e.map[_](x => record(x = x))
           .keyBy[_](r => r.x)
           .window[_](
               Window::tumbling(1min),
               (k, r) => record(
                   k = k,
                   y = f(r.map[_](r => r.x))
               )
           );"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_limit() {
    let a = querycomp(aqua!("from x in e limit 10;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).take(10);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_skip() {
    let a = querycomp(aqua!("from x in e skip 10;")).unwrap();
    let b = querycomp(aqua!("e.map[_](x => record(x = x)).drop(10);")).unwrap();
    check!(a, b);
}
