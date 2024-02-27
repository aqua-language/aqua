use egg::define_language;
use egg::Id;

define_language! {
    pub enum AquaLang {
        Num(i32),
        Bool(bool),

        "add" = Add([Id; 2]),
        "mul" = Mul([Id; 2]),
        "div" = Div([Id; 2]),
        "sub" = Sub([Id; 2]),
        "pow" = Pow([Id; 2]),
        "log" = Ln([Id; 2]),
        "sqrt" = Sqrt([Id; 1]),
        "sin" = Sin([Id; 1]),
        "cos" = Cos([Id; 1]),

        "inv" = Inv([Id; 1]),

        "eq" = Eq([Id; 2]),
        "le" = Le([Id; 2]),
        "lt" = Lt([Id; 2]),
        "and" = And([Id; 2]),
        "or" = Or([Id; 2]),
        "not" = Not([Id; 1]),

        "if" = If([Id; 3]),

        "min" = Min([Id; 2]),
        "max" = Max([Id; 2]),

        "fun" = Fun(Vec<Id>),
        "call" = Call([Id; 2]),
        "compose" = Compose([Id; 2]),
        "_" = Wild,
        "seq" = Seq([Id; 2]),
        "val" = Val([Id; 3]),
        "use" = Use([Id; 1]),

        "tuple" = Tuple(Vec<Id>),

        "map" = Map([Id; 1]),
        "filter" = Filter([Id; 1]),
        "filtermap" = FilterMap([Id; 1]),
        "fold" = Fold([Id; 3]),
        "foldmap" = FoldMap([Id; 4]),

        "enum" = Enum([Id; 3]),
        "pair" = Pair([Id; 2]),
        "fst" = Fst([Id; 1]),
        "snd" = Snd([Id; 1]),

        Symbol(egg::Symbol),
    }
}
