pub trait Operator {
    type In;
    type Out;

    fn next(&mut self, input: Self::In) -> Option<Self::Out>;
}
