use crate::Compiler;

impl Compiler {
    pub(super) fn declare_iterator(&mut self) {
        self.declare_trait(
            "trait Iterator[T] {
                 type Item;
                 def next(data: T): Option[Iterator[T]::Item];
             }",
        );
    }
}
