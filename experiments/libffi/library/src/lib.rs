#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[repr(C)]
pub struct Pair<L, R> {
    pub x: L,
    pub y: R,
}

#[no_mangle]
pub extern "C" fn pair_i32_i32(x: i32, y: i32) -> Pair<i32, i32> {
    Pair { x, y }
}

#[no_mangle]
pub extern "C" fn pair_i64_i32(x: i64, y: i32) -> Pair<i64, i32> {
    Pair { x, y }
}

#[no_mangle]
pub extern "C" fn pair_i32_i64(x: i32, y: i64) -> Pair<i32, i64> {
    Pair { x, y }
}

#[no_mangle]
pub extern "C" fn fst(p: Pair<i32, i32>) -> i32 {
    p.x
}

#[no_mangle]
pub extern "C" fn snd(p: Pair<i32, i32>) -> i32 {
    p.y
}

#[repr(u8)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[no_mangle]
pub extern "C" fn is_red(c: Color) -> bool {
    matches!(c, Color::Red)
}

#[repr(C, u8)]
pub enum Maybe<T> {
    Nothing,
    Just(T),
}

#[no_mangle]
pub extern "C" fn just(just: i32) -> Maybe<i32> {
    Maybe::Just(just)
}

#[no_mangle]
pub extern "C" fn nothing() -> Maybe<i32> {
    Maybe::Nothing
}

#[no_mangle]
pub extern "C" fn unwrap(m: Maybe<i32>) -> i32 {
    match m {
        Maybe::Just(x) => x,
        Maybe::Nothing => panic!("unwrap called on Nothing"),
    }
}

// NOTE: Stabby does not work in the latest stable Rust version.
//
// #[no_mangle]
// pub extern "C" fn len(x: StabbyString) -> usize {
//     x.len()
// }
//
// use stabby::string::String as StabbyString;
//
// #[no_mangle]
// pub extern "C" fn stabby_string(x: i32) -> StabbyString {
//     StabbyString::from(format!("hello {x}").as_str())
// }
//
// #[no_mangle]
// pub extern "C" fn concat(x: StabbyString, y: StabbyString) -> StabbyString {
//     StabbyString::from(format!("{x}{y}").as_str())
// }

#[repr(C)]
pub struct Tuple3 {
    x: i32,
    y: i64,
    z: i32,
}

#[no_mangle]
pub extern "C" fn tuple_i32_i64_i32(x: i32, y: i64, z: i32) -> Tuple3 {
    Tuple3 { x, y, z }
}
