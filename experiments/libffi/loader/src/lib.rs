#![allow(unused)]
use std::alloc::alloc;
use std::alloc::dealloc;
use std::alloc::Layout;
use std::ffi::c_void;
use std::mem::align_of;
use std::mem::size_of;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::rc::Rc;

use libffi::middle::Arg;
use libffi::middle::Builder;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use libffi::middle::Type;
use libffi::raw::ffi_abi_FFI_DEFAULT_ABI;
use libffi::raw::ffi_call;
use libffi::raw::ffi_cif;
use libffi::raw::ffi_prep_cif;
use libffi::raw::ffi_type;
use libloading::os::unix::Symbol;
use libloading::Library;

#[cfg(target_os = "macos")]
const LIB_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../library/target/debug/liblibrary.dylib"
);

#[cfg(target_os = "linux")]
const LIB_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../library/target/debug/liblibrary.so"
);

#[cfg(target_os = "windows")]
const LIB_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../library/target/debug/liblibrary.dll"
);

fn load() -> Library {
    unsafe { Library::new(LIB_PATH).unwrap() }
}

fn fun<'a>(lib: &'a Library, name: &str) -> libloading::Symbol<'a, *mut c_void> {
    unsafe { lib.get(name.as_bytes()).unwrap() }
}

fn cif<const N: usize>(arg_types: [Type; N], ret_type: Type) -> Cif {
    Builder::new().args(arg_types).res(ret_type).into_cif()
}

#[test]
fn test_i32_add() {
    let lib = load();
    let fun = fun(&lib, "add");
    let res = unsafe {
        cif([Type::i32(), Type::i32()], Type::i32())
            .call::<i32>(CodePtr(*fun), &[Arg::new(&10), Arg::new(&20)])
    };
    assert_eq!(res, 30);
}

#[derive(Debug, PartialEq)]
#[repr(C)]
struct Pair<L, R> {
    x: L,
    y: R,
}

#[test]
fn test_struct_pair() {
    let lib = load();
    let fun = fun(&lib, "pair");
    let res = unsafe {
        cif(
            [Type::i32(), Type::i32()],
            Type::structure([Type::i32(), Type::i32()]),
        )
        .call::<Pair<i32, i32>>(CodePtr(*fun), &[Arg::new(&10), Arg::new(&20)])
    };
    assert_eq!(res, Pair { x: 10, y: 20 });
}

#[test]
fn test_struct_pair2() {
    let lib = load();
    let fun = fun(&lib, "pair2");
    let res = unsafe {
        cif(
            [Type::i64(), Type::i32()],
            Type::structure([Type::i64(), Type::i32()]),
        )
        .call::<Pair<i64, i32>>(CodePtr(*fun), &[Arg::new(&10i64), Arg::new(&20i32)])
    };
    assert_eq!(res, Pair { x: 10i64, y: 20i32 });
}

#[test]
fn test_struct_fst() {
    let lib = load();
    let fun = fun(&lib, "fst");
    let res = unsafe {
        cif([Type::structure([Type::i32(), Type::i32()])], Type::i32())
            .call::<i32>(CodePtr(*fun), &[Arg::new(&Pair { x: 10, y: 20 })])
    };
    assert_eq!(res, 10);
}

#[test]
fn test_struct_fst_raw() {
    let lib = load();
    let v: Vec<u8> = vec![0; 4];
    let fun = fun(&lib, "fst");
    let rvalue = v.as_ptr() as *mut c_void;
    let avalue = &[Arg::new(&Pair { x: 10, y: 20 })];
    let avalue = avalue.as_ptr() as *mut *mut c_void;

    unsafe {
        let fun = Some(*CodePtr(*fun).as_fun());
        let cif = cif([Type::structure([Type::i32(), Type::i32()])], Type::i32());
        libffi::raw::ffi_call(cif.as_raw_ptr(), fun, rvalue, avalue);
    }

    let res = unsafe { *(rvalue as *const i32) };
    assert_eq!(res, 10);
}

#[test]
fn test_struct_snd() {
    let lib = load();
    let fun = fun(&lib, "snd");
    let res = unsafe {
        cif([Type::structure([Type::i32(), Type::i32()])], Type::i32())
            .call::<i32>(CodePtr(*fun), &[Arg::new(&Pair { x: 10, y: 20 })])
    };
    assert_eq!(res, 20);
}

#[repr(u8)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn test_enum_color() {
    let lib = load();
    let fun = fun(&lib, "is_red");
    let res = unsafe {
        cif([Type::u8()], Type::u8()).call::<u8>(CodePtr(*fun), &[Arg::new(&Color::Red)]) != 0
    };
    assert_eq!(res, true);
    let res = unsafe {
        cif([Type::u8()], Type::u8()).call::<u8>(CodePtr(*fun), &[Arg::new(&Color::Green)]) != 0
    };
    assert_eq!(res, false);
}

/// #[repr(C)]
/// pub struct MaybeRepr {
///     tag: MaybeTag,
///     payload: MaybePayload,
/// }
/// #[repr(u8)]
/// enum MaybeTag {
///     Nothing,
///     Just,
/// }
///
/// #[repr(C)]
/// pub union MaybePayload {
///    Nothing: MaybePayloadNothing,
///    Just: MaybePayloadJust,
/// }
///
/// #[repr(C)]
/// pub struct MaybePayloadNothing;
///
/// #[repr(C)]
/// pub struct MaybePayloadJust(i32);
#[derive(PartialEq, Debug)]
#[repr(C, u8)]
pub enum Maybe<T> {
    Nothing,
    Just(T),
}

fn type_enum<const N: usize>(variants: [Type; N]) -> Type {
    let t = variants
        .into_iter()
        .max_by_key(|t| unsafe { t.as_raw_ptr().read().size })
        .unwrap();
    Type::structure([Type::u8(), t])
}

#[test]
fn test_enum_just() {
    let lib = load();
    let fun = fun(&lib, "just");
    let res = unsafe {
        cif([Type::i32()], type_enum([Type::i32()]))
            .call::<Maybe<i32>>(CodePtr(*fun), &[Arg::new(&123)])
    };
    assert_eq!(res, Maybe::Just(123));
}

#[test]
fn test_enum_nothing() {
    let lib = load();
    let fun = fun(&lib, "nothing");
    let out = type_enum([Type::i32()]);
    let res = unsafe { cif([], out).call::<Maybe<i32>>(CodePtr(*fun), &[]) };
    assert_eq!(res, Maybe::Nothing);
}

#[test]
fn test_enum_just_pair() {
    let lib = load();
    let fun = fun(&lib, "nothing");
    let out = type_enum([Type::structure([Type::i32(), Type::i32()])]);
    let res = unsafe { cif([], out).call::<Maybe<Pair<i32, i32>>>(CodePtr(*fun), &[]) };
    assert_eq!(res, Maybe::Nothing);
}

#[test]
fn test_struct_pair_dynamic_heap() {
    let lib = load();
    let fun = fun(&lib, "pair2");
    unsafe {
        let (layout, offset) = Layout::new::<i64>().extend(Layout::new::<i32>()).unwrap();
        let layout = layout.pad_to_align();
        let ptr = alloc(layout);
        libffi::raw::ffi_call(
            cif(
                [Type::i64(), Type::i32()],
                Type::structure([Type::i64(), Type::i32()]),
            )
            .as_raw_ptr(),
            Some(*CodePtr(*fun).as_safe_fun()),
            ptr as *mut std::os::raw::c_void,
            [Arg::new(&10i64), Arg::new(&20i32)].as_ptr() as *mut *mut std::os::raw::c_void,
        );
        let x = ptr.cast::<i64>().read();
        let y = ptr.add(offset).cast::<i32>().read();
        assert_eq!(x, 10i64);
        assert_eq!(y, 20i32);
        dealloc(ptr, layout);
    };
}

#[test]
fn test_struct_pair_dynamic_stack() {
    let lib = load();
    let fun = fun(&lib, "pair2");
    unsafe {
        let (layout, offset) = Layout::new::<i64>().extend(Layout::new::<i32>()).unwrap();
        let layout = layout.pad_to_align();
        alloca::with_alloca_zeroed(layout.size(), |ptr| {
            let ptr = (*ptr).as_mut_ptr();
            libffi::raw::ffi_call(
                cif(
                    [Type::i64(), Type::i32()],
                    Type::structure([Type::i64(), Type::i32()]),
                )
                .as_raw_ptr(),
                Some(*CodePtr(*fun).as_safe_fun()),
                ptr as *mut std::os::raw::c_void,
                [Arg::new(&10i64), Arg::new(&20i32)].as_ptr() as *mut *mut std::os::raw::c_void,
            );
            let x = ptr.cast::<i64>().read();
            let y = ptr.add(offset).cast::<i32>().read();
            assert_eq!(x, 10i64);
            assert_eq!(y, 20i32);
        })
    };
}

#[test]
fn test_enum_just_dynamic_heap() {
    let lib = load();
    let fun = fun(&lib, "just");
    unsafe {
        let (layout, offset) = Layout::new::<u8>().extend(Layout::new::<i32>()).unwrap();
        let layout = layout.pad_to_align();
        let ptr = alloc(layout);
        libffi::raw::ffi_call(
            cif([Type::i32()], type_enum([Type::i32()])).as_raw_ptr(),
            Some(*CodePtr(*fun).as_safe_fun()),
            ptr as *mut c_void,
            [Arg::new(&123i32)].as_ptr() as *mut *mut std::os::raw::c_void,
        );
        let tag = ptr.cast::<u8>().read();
        let val = ptr.add(offset).cast::<i32>().read();
        assert_eq!(tag, 1u8);
        assert_eq!(val, 123i32);
        dealloc(ptr, layout);
    };
}

use stabby::alloc::libc_alloc::LibcAlloc;
type StabbyString = stabby::string::String<LibcAlloc>;

#[test]
fn test_pair_raw() {
    let lib = load();
    let fun = fun(&lib, "pair");
    unsafe {
        let (layout, offset) = Layout::new::<i32>().extend(Layout::new::<i32>()).unwrap();
        let layout = layout.pad_to_align();
        let ptr = alloc(layout);
        let mut return_ty = libffi::low::ffi_type {
            size: layout.size(),
            alignment: layout.align() as u16,
            type_: libffi::raw::FFI_TYPE_STRUCT as u16,
            elements: [Type::structure([]).as_raw_ptr()].as_mut_ptr(),
        };
        let mut arg_types = [Type::i32().as_raw_ptr(), Type::i32().as_raw_ptr()];
        let nargs = arg_types.len() as u32;
        let mut cif = ffi_cif::default();
        ffi_prep_cif(
            &mut cif,
            ffi_abi_FFI_DEFAULT_ABI,
            nargs,
            &mut return_ty as *mut ffi_type,
            arg_types.as_mut_ptr(),
        );
        let mut ptr = alloc(layout);
        ffi_call(
            &mut cif,
            Some(*CodePtr(*fun).as_safe_fun()),
            ptr as *mut c_void,
            [Arg::new(&100i32), Arg::new(&123i32)].as_ptr() as *mut *mut c_void,
        );
        let tag = ptr.cast::<i32>().read();
        let val = ptr.add(offset).cast::<i32>().read();
        assert_eq!(tag, 100i32);
        assert_eq!(val, 123i32);
        dealloc(ptr, layout);
    };
}

#[test]
fn test_string_raw() {
    let lib = load();
    let fun = fun(&lib, "stabby_string");
    unsafe {
        let layout = Layout::new::<StabbyString>();
        let ptr = alloc(layout);
        let mut return_ty = libffi::low::ffi_type {
            size: layout.size(),
            alignment: layout.align() as u16,
            type_: libffi::raw::FFI_TYPE_STRUCT as u16,
            elements: [Type::structure([]).as_raw_ptr()].as_mut_ptr(),
        };
        let mut arg_types = [Type::i32().as_raw_ptr()];
        let nargs = arg_types.len() as u32;
        let mut cif = ffi_cif::default();
        ffi_prep_cif(
            &mut cif,
            ffi_abi_FFI_DEFAULT_ABI,
            nargs,
            &mut return_ty as *mut ffi_type,
            arg_types.as_mut_ptr(),
        );
        let mut ptr = alloc(layout);
        ffi_call(
            &mut cif,
            Some(*CodePtr(*fun).as_safe_fun()),
            ptr as *mut c_void,
            [Arg::new(&100i32)].as_ptr() as *mut *mut c_void,
        );
        let s = ptr.cast::<StabbyString>().read();
        assert_eq!(s, "hello 100");
        dealloc(ptr, layout);
    };
}

#[test]
fn test_string_concat() {
    let lib = load();
    let fun = fun(&lib, "concat");
    unsafe {
        let layout = Layout::new::<StabbyString>();
        let ptr = alloc(layout);
        // Create string type
        let string_ty = &mut libffi::low::ffi_type {
            size: layout.size(),
            alignment: layout.align() as u16,
            type_: libffi::raw::FFI_TYPE_STRUCT as u16,
            elements: [Type::structure([]).as_raw_ptr()].as_mut_ptr(),
        } as *mut ffi_type;

        let mut arg_types = [string_ty, string_ty];
        let nargs = arg_types.len() as u32;
        let mut cif = ffi_cif::default();
        ffi_prep_cif(
            &mut cif,
            ffi_abi_FFI_DEFAULT_ABI,
            nargs,
            string_ty,
            arg_types.as_mut_ptr(),
        );
        let a0 = StabbyString::from("hello");
        let a1 = StabbyString::from("world");
        ffi_call(
            &mut cif,
            Some(*CodePtr(*fun).as_safe_fun()),
            ptr as *mut c_void,
            [Arg::new(&a0), Arg::new(&a1)].as_ptr() as *mut *mut c_void,
        );
        std::mem::forget(a0);
        std::mem::forget(a1);
        let s = ptr.cast::<StabbyString>().read();
        assert_eq!(s, "helloworld");
    };
}
