pub trait Lookahead {
    type Token;
    const FIRST: Self::Token;
    const FOLLOW: Self::Token;
}
