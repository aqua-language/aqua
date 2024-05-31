use crate::Compiler;

impl Compiler {
    pub(super) fn declare_into_iterator(&mut self) {
        self.declare_trait(
            "trait IntoIterator[T]
             where Iterator[IntoIterator[T]::IntoIter,
                             Item = IntoIterator[T]::Item] {
                 type Item;
                 type IntoIter;
                 def into_iter(data: T): IntoIterator[T]::IntoIter;
             }",
        );
    }
}
