use runtime::builtins::aggregator::Aggregator;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_aggregator(&mut self) {
        self.declare_type(
            "type Aggregator[I,P,O];",
            BuiltinType { rust: "Aggregator" },
        );

        self.declare_def(
            "def incremental[I,P,O](lift: fun(I):P, combine: fun(P,P):P, lower: fun(P):O): Aggregator[I,P,O];",
            BuiltinDef {
                rust: "Aggregator::incremental",
                fun: |_ctx, _t, v| {
                    let a0 = v[0].as_function();
                    let a1 = v[1].as_function();
                    let a2 = v[2].as_function();
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
                fun: |_ctx, _t, v| {
                    let a0 = v[0].as_function();
                    Aggregator::Holistic { compute: a0 }.into()
                },
            },
        );
    }
}
