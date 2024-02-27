use egg::Analysis;
use egg::Rewrite;

use egg::rewrite as rw;

use crate::AquaLang;

pub fn arithmetic<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Addition
        rw!("add-0"; "(add ?a 0)" => "?a"),
        rw!("add-commutative"; "(add ?a ?b)" => "(add ?b ?a)"),
        rw!("add-associative"; "(add ?a (add ?b ?c))" => "(add (add ?a ?b) ?c)"),
        // Subtraction
        rw!("sub-0"; "(sub ?a 0)" => "?a"),
        rw!("sub-cancel"; "(sub ?a ?a)" => "0"),
        rw!("sub => add-sub"; "(sub ?a ?b)" => "(add ?a (mul -1 ?b))"),
        rw!("sub-semi-associative"; "(sub (sub ?a ?b) ?c)" => "(sub ?a (add ?b ?c))"),
        // Multiplication
        rw!("mul-0"; "(mul ?a 0)" => "0"),
        rw!("mul-1"; "(mul ?a 1)" => "?a"),
        rw!("mul-commutative"; "(mul ?a ?b)" => "(mul ?b ?a)"),
        rw!("mul-associative"; "(mul ?a (mul ?b ?c))" => "(mul (mul ?a ?b) ?c)"),
        // Division
        rw!("div-1"; "(div ?a 1)" => "?a"),
        rw!("div-cancel"; "(div ?a ?a)" => "1"),
        rw!("div => mul-pow"; "(div ?a ?b)" => "(mul ?a (pow ?b -1))"),
        rw!("div-semi-associative"; "(div (div ?a ?b) ?c)" => "(div ?a (mul ?b ?c))"),
        // Addition-Multiplication
        rw!("add-mul-distribute"; "(mul ?a (add ?b ?c))" => "(add (mul ?a ?b) (mul ?a ?c))"),
        rw!("add-mul-factor" ; "(add (mul ?a ?b) (mul ?a ?c))" => "(mul ?a (add ?b ?c))"),
        // Addition-Subtraction
        rw!("add-sub-cancel"; "(sub (add ?a ?x) ?a)" => "?x"),
        rw!("sub-add-cancel"; "(add (sub ?a ?x) ?a)" => "?x"),
        // Multiplication-Division
        rw!("mul-div-cancel"; "(div (mul ?a ?b) ?a)" => "?b"),
        rw!("div-mul-cancel"; "(mul (div ?a ?b) ?b)" => "?a"),
    ]
}

pub fn logarithmic<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Logarithm
        rw!("log-base-1"; "(log 1 ?a)" => "0"),
        // Logarithm-Mixed
        rw!("log-mul"; "(log ?base (mul ?a ?b))" => "(add (log ?base ?a) (log ?base ?b))"),
        rw!("log-div"; "(log ?base (div ?a ?b))" => "(sub (log ?base ?a) (log ?base ?b))"),
        rw!("log-pow"; "(log ?base1 (pow ?base2 ?exp))" => "(mul ?base2 (log ?base1 ?exp))"),
        rw!("log-log"; "(log ?base (log ?base ?a))" => "?a"),
    ]
}

pub fn power<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Power
        rw!("pow-exp-0"; "(pow ?base 0)" => "1"),
        rw!("pow-exp-1"; "(pow ?base 1)" => "?base"),
        rw!("pow-base-0"; "(pow 0 ?exp)" => "0"),
        rw!("pow-base-1"; "(pow 1 ?exp)" => "1"),
        // Power-Mixed
        rw!("pow-mul"; "(mul (pow ?base ?exp1) (pow ?base ?exp2))" => "(pow ?base (add ?exp1 ?exp2))"),
        rw!("pow-pow"; "(pow (pow ?base ?exp1) ?exp2)" => "(pow ?base (mul ?exp1 ?exp2))"),
        rw!("pow-div"; "(div (pow ?base ?exp1) (pow ?base ?exp2))" => "(pow ?base (sub ?exp1 ?exp2))"),
        // Sqrt-pow
        rw!("sqrt-pow-2"; "(sqrt (pow ?base 2))" => "?base"),
        rw!("pow-2-sqrt"; "(pow (sqrt ?base) 2)" => "?base"),
        rw!("sqrt-pow-2-inv"; "?base" => "(sqrt (pow ?base 2))"),
        rw!("pow-2-sqrt-inv"; "?base" => "(pow (sqrt ?base) 2)"),
    ]
}

pub fn trigonometric<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Sine and Cosine
        rw!("sin-sum"; "(sin (add ?a ?b))" => "(add (mul (sin ?a) (cos ?b)) (mul (cos ?a) (sin ?b)))"),
        rw!("sin-difference"; "(sin (sub ?a ?b))" => "(sub (mul (sin ?a) (cos ?b)) (mul (cos ?a) (sin ?b)))"),
        rw!("cos-sum"; "(cos (add ?a ?b))" => "(sub (mul (cos ?a) (cos ?b)) (mul (sin ?a) (sin ?b)))"),
        rw!("cos-difference"; "(cos (sub ?a ?b))" => "(add (mul (cos ?a) (cos ?b)) (mul (sin ?a) (sin ?b)))"),
        rw!("sin-double-angle"; "(sin (mul 2 ?a))" => "(mul (mul 2 (sin ?a)) (cos ?a))"),
        rw!("cos-double-angle"; "(cos (mul 2 ?a))" => "(sub (pow (cos ?a) 2) (pow (sin ?a) 2))"),
        rw!("sin-half-angle"; "(pow (sin (div ?a 2)) 2)" => "(div (sub 1 (cos ?a)) 2)"),
        rw!("cos-half-angle"; "(pow (cos (div ?a 2)) 2)" => "(div (add 1 (cos ?a)) 2)"),
        rw!("sin-cos-product-to-sum"; "(mul (sin ?a) (cos ?b))" => "(div (add (sin (sub ?a ?b)) (sin (add ?a ?b))) 2)"),
        rw!("cos-sin-product-to-sum"; "(mul (cos ?a) (sin ?b))" => "(div (sub (sin (add ?a ?b)) (sin (sub ?a ?b))) 2)"),
        rw!("cos-cos-product-to-sum"; "(mul (cos ?a) (cos ?b))" => "(div (add (cos (sub ?a ?b)) (cos (add ?a ?b))) 2)"),
        rw!("sin-sin-product-to-sum"; "(mul (sin ?a) (sin ?b))" => "(div (sub (cos (sub ?a ?b)) (cos (add ?a ?b))) 2)"),
        rw!("sin-sum-to-product"; "(add (sin ?a) (sin ?b))" => "(mul (mul 2 (sin (div (add ?a ?b) 2))) (cos (div (sub ?a ?b) 2)))"),
        rw!("sin-difference-to-product"; "(sub (sin ?a) (sin ?b))" => "(mul (mul 2 (cos (div (add ?a ?b) 2))) (sin (div (sub ?a ?b) 2)))"),
        rw!("cos-sum-to-product"; "(add (cos ?a) (cos ?b))" => "(mul (mul 2 (cos (div (add ?a ?b) 2))) (cos (div (sub ?a ?b) 2)))"),
        rw!("cos-difference-to-product"; "(sub (cos ?a) (cos ?b))" => "(mul (mul -2 (sin (div (add ?a ?b) 2))) (sin (div (sub ?a ?b) 2)))"),
    ]
}

pub fn logical<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Not
        rw!("not-not"; "(not (not ?a))" => "?a"),
        rw!("not-true"; "(not true)" => "false"),
        rw!("not-false"; "(not false)" => "true"),
        // And
        rw!("and-true"; "(and ?a true)" => "?a"),
        rw!("and-false"; "(and ?a false)" => "false"),
        rw!("and-associative"; "(and ?a (and ?b ?c))" => "(and (and ?a ?b) ?c)"),
        rw!("and-commutative"; "(and ?a ?b)" => "(and ?b ?a)"),
        // rw!("and-idempotence"; "(and ?a ?a)" => "?a"),
        // Or
        rw!("or-true"; "(or ?a true)" => "true"),
        rw!("or-false"; "(or ?a false)" => "?a"),
        rw!("or-associative"; "(or ?a (or ?b ?c))" => "(or (or ?a ?b) ?c)"),
        rw!("or-commutative"; "(or ?a ?b)" => "(or ?b ?a)"),
        rw!("or-idempotence"; "(or ?a ?a)" => "?a"),
        // Not-And-Or
        rw!("and-or-distribute"; "(and ?a (or ?b ?c))" => "(or (and ?a ?b) (and ?a ?c))"),
        rw!("or-and-distribute"; "(or ?a (and ?b ?c))" => "(and (or ?a ?b) (or ?a ?c))"),
        rw!("not-and"; "(not (and ?a ?b))" => "(or (not ?a) (not ?b))"),
        rw!("not-or"; "(not (or ?a ?b))" => "(and (not ?a) (not ?b))"),
    ]
}

pub fn relational<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Eq
        rw!("eq-reflexive"; "(eq ?a ?a)" => "true"),
        rw!("eq-commutative"; "(eq ?a ?b)" => "(eq ?b ?a)"),
        rw!("eq-transitive"; "(and (eq ?a ?b) (eq ?b ?c))" => "(eq ?a ?c)"),
        // Le
        rw!("le-reflexive"; "(le ?a ?a)" => "true"),
        rw!("le-antisymmetric"; "(and (le ?a ?b) (le ?b ?a))" => "(eq ?a ?b)"),
        rw!("le-transitive"; "(and (le ?a ?b) (le ?b ?c))" => "(le ?a ?c)"),
        rw!("le-negate"; "(le ?a ?b)" => "(not (lt ?b ?a))"),
        rw!("le-negate-rev"; "(le ?a ?b)" => "(not (lt ?b ?a))"),
        rw!("le-decompose"; "(le ?a ?b)" => "(or (lt ?a ?b) (eq ?a ?b))"),
        // Lt
        rw!("lt-anti-reflexive"; "(lt ?a ?a)" => "false"),
        rw!("lt-transitive"; "(and (lt ?a ?b) (lt ?b ?c))" => "(lt ?a ?c)"),
        rw!("lt-assymetric"; "(lt ?a ?b)" => "(not (lt ?b ?a))"),
        rw!("lt-negate"; "(lt ?a ?b)" => "(not (le ?b ?a))"),
        rw!("lt-negate-rev"; "(not (lt ?a ?b))" => "(le ?b ?a)"),
    ]
}

pub fn if_else<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // If (Flatten)
        rw!("if-true"; "(if true ?then ?else)" => "?then"),
        rw!("if-false"; "(if false ?then ?else)" => "?else"),
        rw!("if-true-false"; "(if ?cond true false)" => "?cond"),
        rw!("if-true-true"; "(if ?cond true true)" => "true"),
        rw!("if-false-true"; "(if ?cond false true)" => "(not ?cond)"),
        rw!("if-false-false"; "(if false ?then ?else)" => "?else"),
        rw!("if-elim"; "(if ?cond ?branch ?branch)" => "?branch"),
        rw!("if-negate"; "(if (not ?cond) ?branch1 ?branch2)" => "(if ?cond ?branch2 ?branch1)"),
        rw!("if-negate-rev"; "(if ?cond ?branch1 ?branch2)" => "(if (not ?cond) ?branch2 ?branch1)"),
        rw!("if-associate-1"; "(if ?cond1 (if ?cond2 ?then ?else1) ?else2)" => "(if (and ?cond1 ?cond2) ?then (if ?cond1 ?else1 ?else2))"),
        rw!("if-associate-2"; "(if ?cond1 ?then1 (if ?cond2 ?then2 ?else))" => "(if (and ?cond1 (not ?cond2)) ?else (if ?cond1 ?then1 ?then2))"),
        rw!("if-cond-swap"; "(if (if ?cond ?a ?b) ?c ?d)" => "(if ?cond (if ?a ?c ?d) (if ?b ?c ?d))"),
        rw!("if-cond-elim-then"; "(if ?cond (if ?cond ?a ?b) ?c)" => "(if ?cond ?a ?c)"),
        rw!("if-cond-elim-else"; "(if ?cond ?a (if ?cond ?b ?c))" => "(if ?cond ?a ?c)"),
        rw!("if-cond-flatten"; "(if ?cond (if ?cond ?a ?b) ?c)" => "(if ?cond ?a ?c)"),
    ]
}

pub fn min_max<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        // Min, Max
        rw!("min-associative"; "(min ?a (min ?b ?c))" => "(min (min ?a ?b) ?c)"),
        rw!("max-associative"; "(max ?a (max ?b ?c))" => "(max (max ?a ?b) ?c)"),
        rw!("min-commutative"; "(min ?a ?b)" => "(min ?b ?a)"),
        rw!("max-commutative"; "(max ?a ?b)" => "(max ?b ?a)"),
    ]
}

pub fn inverse<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![rw!("inv-mul"; "(inv (add (mul ?a ?b) ?c))" => "(div (inv (add ?a ?b))")]
}

pub fn function<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("fun-lift"; "?x" => "(fun _ ?x)"),
        rw!("compose"; "(compose ?f ?g)" => "(fun a (call ?f (call ?g a)))"),
    ]
}

pub fn for_loop<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("for-to-fold"; "(for ?x ?iter ?init ?body)" => "(fold ?iter ?init (lambda (?x ?acc) ?body))"),
    ]
}

pub fn filter<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("filter-filter"; "(filter (filter ?x ?f) ?g)" => "(filter ?x ((fun a (and (call ?f a) (call ?g a))))"),
        rw!("filter-2x"; "(filter (filter ?x ?f) ?f)" => "(filter ?x ?f"),
        rw!("filter-map"; "(map (filter ?x ?f) ?g)" => "(filtermap (fun a (if (call ?f a) (enum Option Some ?a) (enum Option None unit))))"),
        rw!("filter-true"; "(filter ?x (fun ?a true))" => "?x"),
    ]
}

pub fn map<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("map-map"; "(map (map ?x ?f) ?g)" => "(map ?x (compose ?f ?g))"),
        rw!("map-id"; "(map ?x (fun ?a ?a))" => "?x"),
    ]
}

// (fold ?x ?init ?lift ?combine ?lower)
pub fn fold<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("fold-fusion";
            "(pair (fold ?x ?init0 ?combine0) (fold ?x ?init1 ?combine1))" =>
            "(fold
                 ?x
                 (fun elem (pair (call ?init0 elem) (call ?init1 elem)))
                 (fun acc (fun elem
                     (pair
                         (call (call ?combine0 (fst acc)) elem)
                         (call (call ?combine1 (snd acc)) elem)))))"),
        rw!("fold-sum-mul"; "(fold ?x 0 (fun ?acc (fun ?elem (add ?acc (mul ?const ?elem)))))" =>
            "(mul ?const (fold ?x 0 (fun ?acc (fun ?elem (add ?acc ?elem)))))"),
    ]
}

pub fn foldmap<N: Analysis<AquaLang>>() -> Vec<Rewrite<AquaLang, N>> {
    vec![
        rw!("fold-to-foldmap"; "(fold (map ?x ?f) ?init ?lift ?combine ?lower)" =>
                        "(fold ?x ?init (compose ?lift ?f) ?combine ?lower)"),
        rw!("foldmap-fusion"; "(pair
            (foldmap ?x (?id0 ?lift0 ?combine0 ?lower0))
            (foldmap ?x (?id1 ?lift1 ?combine1 ?lower1))
        )" => "(foldmap (?x
            (pair ?id0 ?id1)
            (fun a (pair (call ?lift0 a) (call ?lift1 a)))
            (fun a (fun b (pair (?combine0 (fst a) (fst b) )
                                (?combine1 (snd a) (snd b) ))))
            (fun a (pair (?lower0 (fst a))
                        (?lower1 (snd a))))))"),
        rw!("map-fold"; "(fold (map ?x ?f) ?init ?lift ?combine ?lower)" =>
                        "(fold ?x ?init (compose ?lift ?f) ?combine ?lower)"),
        rw!("fold-map"; "(map (fold ?x ?init ?lift ?combine ?lower) ?f)" =>
                        "(fold ?x ?init ?lift ?combine (compose ?lower ?f)"),
    ]
}
