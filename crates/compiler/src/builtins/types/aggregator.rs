use runtime::builtins::aggregator::Aggregator;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_aggregator(&mut self) {
        self.declare_type(
            "type Aggregator[I,P,O];",
            BuiltinType { rust: "Aggregator" },
        );

        self.declare_def(
            "def incremental[I,P,O](lift: fun(I):P, combine: fun(P,P):P, lower: fun(P):O): Aggregator[I,P,O];",
            BuiltinDef {
                rust: "Aggregator::incremental",
                fun: |_ctx, v| {
                    let a0 = v[0].rc();
                    let a1 = v[1].rc();
                    let a2 = v[2].rc();
                    Aggregator::Incremental {
                        lift: a0,
                        combine: a1,
                        lower: a2,
                    }
                    .into()
                },
            },
        );

        self.declare_def(
            "def holistic[I,P,O](reduce: fun(I):P): Aggregator[I,P,O];",
            BuiltinDef {
                rust: "Aggregator::holistic",
                fun: |_ctx, v| {
                    let a0 = v[0].rc();
                    Aggregator::Holistic { compute: a0 }.into()
                },
            },
        );
    }
}
