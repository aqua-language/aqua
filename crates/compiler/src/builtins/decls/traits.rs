use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_traits(&mut self) {
        // self.declare_trait(
        //     "trait Iterator[T] {
        //          type Item;
        //          def next(data: T): Option[Iterator[T]::Item];
        //      }",
        // );
        // self.declare_trait(
        //     "trait IntoIterator[T] {
        //         type Item;
        //         type IntoIter;
        //         def into_iter(data: T): IntoIterator[T]::IntoIter;
        //     }",
        // );
        self.declare_trait(
            "trait Add[A,B] {
                 type Output;
                 def add(a:A, b:B): Add[A,B]::Output;
             }",
        );
        self.declare_trait(
            "trait Sub[A,B] {
                 type Output;
                 def sub(a:A, b:B): Sub[A,B]::Output;
             }",
        );
        self.declare_trait(
            "trait Mul[A,B] {
                 type Output;
                 def mul(a:A, b:B): Mul[A,B]::Output;
             }",
        );
        self.declare_trait(
            "trait Div[A,B] {
                 type Output;
                 def div(a:A, b:B): Div[A,B]::Output;
             }",
        );
        // self.declare_trait(
        //     "trait Eq[T] {
        //          def eq(a:T, b:T): bool;
        //      }",
        // );
        // self.declare_trait(
        //     "trait Not[T] {
        //          type Output;
        //          def not(v:T): Not[T]::Output;
        //      }",
        // );
        // self.declare_type("type Ordering;", BuiltinType { rust: "Ordering" });
        // self.declare_trait(
        //     "trait Ord[T] {
        //          def cmp(a:T, b:T): Ordering;
        //      }",
        // );
        // self.declare_trait(
        //     "trait Clone[T] {
        //          def clone(v:T): T;
        //      }",
        // );
        // self.declare_trait("trait Copy[T] {}");
        // self.declare_trait(
        //     "trait Display[T] {
        //          def to_string(v:T): String;
        //      }",
        // );
        // self.declare_trait(
        //     "trait Debug[T] {
        //          def to_string(v:T): String;
        //      }",
        // );
        // self.declare_trait(
        //     "trait DeepClone[T] {
        //          def deep_clone(v:T): T;
        //      }",
        // );
        // self.declare_trait(
        //     "trait Serde[T] {
        //          def serialize(v:T): String;
        //          def deserialize(v:String): T;
        //     }",
        // );
    }
}
