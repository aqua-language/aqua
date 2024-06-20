mod common;

#[test]
fn test_desugar_query_from0() {
    let a = querycomp_program!("from x in e;").unwrap();
    let b = querycomp_program!("e.map(fun(x) = record(x));").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_where1() {
    let a = querycomp_program!("from x in e where x;").unwrap();
    let b = querycomp_program!("e.map(fun(x) = record(x)).filter(fun(r) = r.x);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_from0() {
    let a = querycomp_program!("from x0 in e0 from x1 in e1;").unwrap();
    let b = querycomp_program!(
        "e0.map(fun(x0) = record(x0))
           .flatMap(fun(r) = e1.map(fun(x1) = record(x1 = x1, x0 = r.x0)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_select() {
    let a = querycomp_program!("from x in e select y=f(x);").unwrap();
    let b = querycomp_program!(
        "e.map(fun(x) = record(x = x))
          .map(fun(r) = record(y = f(r.x)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_group() {
    let a = querycomp_program!(
        "from x in e0
         group k=x
            over tumbling(1min)
            compute a = sum of x, b = count of x;"
    )
    .unwrap();
    let b = querycomp_program!(
        "e0.map(fun(x) = record(x = x))
           .keyBy(fun(r) = r.x)
           .window(
               tumbling(1min),
               fun(k, r) = record(
                   k = k,
                   a = r.map(fun(r) = r.x).sum(),
                   b = r.map(fun(r) = r.x).count()
               )
           );"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_join() {
    let a = querycomp_program!(
        "from x in e0
         join y in e1 on x.a == y.b;"
    )
    .unwrap();
    let b = querycomp_program!(
        "e0.map(fun(x) = record(x = x))
           .flatMap(fun(r) = e1.filter(fun(y) = r.x.a == y.b)
                               .map(fun(y) = record(y = y, x = r.x)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_over_compute() {
    let a = querycomp_program!(
        "from x in e
         over tumbling(1min)
         compute a = sum of x,
                 b = count of x;"
    )
    .unwrap();
    let b = querycomp_program!(
        "e.map(fun(x) = record(x = x))
          .window(
              tumbling(1min),
              fun(r) = record(
                  a = r.map(fun(r) = r.x).sum(),
                  b = r.map(fun(r) = r.x).count()
              )
          );"
    )
    .unwrap();
    check!(a, b);
}
