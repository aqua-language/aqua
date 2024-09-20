use alloca::with_alloca_zeroed;
use libffi::high::call;
use libffi::low::CodePtr;
use libffi::middle::Cif;
use libloading::Library;
use libloading::Symbol;

use crate::cif;
use crate::fun;
use crate::load;
use std::alloc::alloc;
use std::alloc::Layout;
use std::ffi::c_void;

#[derive(Clone)]
pub enum Type {
    Cons(&'static str),
    Func(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
}

#[derive(Clone)]
pub enum Expr {
    Index(Type, Box<Expr>, usize),
    Tuple(Type, Vec<Expr>),
    I32(Type, i32),
    FFICall(Type, Box<Expr>, Vec<Expr>),
    FFIFunc(Type, &'static str),
}

#[derive(Clone)]
pub enum Value {
    FFIFunction(&'static str),
    Tuple(Vec<Value>),
    I32(i32),
}

type Offset = usize;

pub enum MemType {
    Cons(&'static str),
    Tuple(Offset, Vec<MemType>),
}

impl<'a> From<&'a Type> for libffi::middle::Type {
    fn from(t: &'a Type) -> Self {
        match t {
            Type::Cons("i32") => libffi::middle::Type::i32(),
            Type::Cons("i64") => libffi::middle::Type::i64(),
            Type::Cons("u8") => libffi::middle::Type::u8(),
            Type::Cons("u16") => libffi::middle::Type::u16(),
            Type::Cons("u32") => libffi::middle::Type::u32(),
            Type::Cons("u64") => libffi::middle::Type::u64(),
            Type::Cons("f32") => libffi::middle::Type::f32(),
            Type::Cons("f64") => libffi::middle::Type::f64(),
            Type::Cons("bool") => libffi::middle::Type::u8(),
            Type::Cons("char") => libffi::middle::Type::u8(),
            Type::Tuple(ts) => {
                let ts = ts.into_iter().map(|t| t.into()).collect::<Vec<_>>();
                libffi::middle::Type::structure(ts)
            }
            _ => unimplemented!(),
        }
    }
}

impl Expr {
    fn type_of(&self) -> &Type {
        match self {
            Expr::Index(t, _, _) => t,
            Expr::Tuple(t, _) => t,
            Expr::I32(t, i) => t,
            Expr::FFICall(t, _, _) => t,
            Expr::FFIFunc(t, _) => t,
        }
    }

    fn eval(&self, lib: &mut Library) -> Value {
        match self {
            Expr::Index(t, e, i) => {
                let Value::Tuple(vs) = e.eval(lib) else {
                    unreachable!()
                };
                vs.get(*i).unwrap().clone()
            }
            Expr::Tuple(t, es) => Value::Tuple(es.iter().map(|e| e.eval(lib)).collect()),
            Expr::I32(t, i) => Value::I32(*i),
            Expr::FFICall(t, e, es) => {
                let Value::FFIFunction(name) = e.eval(lib) else {
                    unreachable!()
                };
                let f = fun(lib, name);
                let cif = t.cif();
                let vs = es.iter().map(|e| e.eval(lib)).collect::<Vec<_>>();
                // let vs = vs.iter().map(|v| v.alloc(t.clone())).collect::<Vec<_>>();
                // cif([Type::i32(), Type::i32()], Type::i32())
                //     .call::<i32>(CodePtr(*fun), &[Arg::new(&10i32), Arg::new(&20i32)])
                // f.call()
                // let r = lib.ffi_call(f, vs);
                todo!()
                // r
            }
            Expr::FFIFunc(t, name) => {
                // let f = lib.ffi_function(name);
                // Value::I32(f as i32)
                todo!()
            }
        }
    }
}

fn create_cif(arg_ts: &[Type], ret_t: &Type) -> Cif {
    let arg_ts = arg_ts.iter().map(Into::into).collect::<Vec<_>>();
    let ret_t = ret_t.into();
    Cif::new(arg_ts, ret_t)
}

unsafe fn call_alloca<'a, T>(
    f: libloading::Symbol<'a, *mut c_void>,
    arg_ts: &[Type],
    ret_t: &Type,
    arg_vs: &[Value],
) -> T {
    if let ([arg_v, arg_vs @ ..], [arg_t, arg_ts @ ..]) = (arg_vs, arg_ts) {
        let (layout, t) = arg_t.layout();
        with_alloca_zeroed(layout.size(), |arg_ptr| {
            call_alloca(f, arg_ts, ret_t, arg_vs)
        })
    } else {
        let (layout, t) = ret_t.layout();
        with_alloca_zeroed(layout.size(), |ret_ptr| {
            libffi::raw::ffi_call(
                create_cif(arg_ts, ret_t).as_raw_ptr(),
                Some(*CodePtr(*f).as_safe_fun()),
                (ret_ptr).as_mut_ptr() as *mut std::os::raw::c_void,
                args.as_ptr() as *mut *mut std::os::raw::c_void,
            );
        })
    }
}

impl Value {
    unsafe fn alloc(&self, t: Type) -> *mut u8 {
        let (l, mt) = t.layout();
        let ptr = alloc(l);
        ptr
    }

    unsafe fn write(&self, ptr: *mut std::ffi::c_void) {
        match self {
            Value::Tuple(vs) => {
                for (i, v) in vs.iter().enumerate() {
                    let offset = 0;
                    let ptr = ptr.add(offset);
                    v.write(ptr);
                }
            }
            Value::I32(i) => {
                unsafe { ptr.cast::<i32>().write(*i) };
            }
            Value::FFIFunction(f) => {
                unreachable!();
            }
        }
    }
}

impl Type {
    pub fn cif(&self) -> Option<Cif> {
        let Type::Func(ts, t) = self else {
            return None;
        };
        let ts: Vec<libffi::middle::Type> = ts.iter().map(Into::into).collect();
        let t: libffi::middle::Type = t.as_ref().into();
        Some(Cif::new(ts, t))
    }

    pub fn alloc(&self) -> *mut std::ffi::c_void {
        let (layout, _) = self.layout();
        unsafe { std::alloc::alloc(layout) as *mut std::ffi::c_void }
    }

    pub fn dealloc(&self, ptr: *mut std::ffi::c_void) {
        let (layout, _) = self.layout();
        unsafe { std::alloc::dealloc(ptr as *mut u8, layout) }
    }

    pub fn read<T>(&self, ptr: *mut std::ffi::c_void) -> T {
        let (layout, _) = self.layout();
        unsafe { (ptr as *mut T).read() }
    }

    pub fn layout(&self) -> (Layout, MemType) {
        let mut layout = Layout::from_size_align(0, 1).unwrap();
        let (layout, offset_type) = self._layout(layout);
        let layout = layout.pad_to_align();
        (layout, offset_type)
    }

    fn _layout(&self, mut layout: Layout) -> (Layout, MemType) {
        match self {
            Type::Cons("i32") => {
                let (l, o) = layout.extend(Layout::new::<i32>()).unwrap();
                (l, MemType::Cons("i32"))
            }
            Type::Cons("i64") => {
                let (l, o) = layout.extend(Layout::new::<i64>()).unwrap();
                (l, MemType::Cons("i64"))
            }
            Type::Cons("u8") => {
                let (l, o) = layout.extend(Layout::new::<u8>()).unwrap();
                (l, MemType::Cons("u8"))
            }
            Type::Cons("u16") => {
                let (l, o) = layout.extend(Layout::new::<u16>()).unwrap();
                (l, MemType::Cons("u16"))
            }
            Type::Cons("u32") => {
                let (l, o) = layout.extend(Layout::new::<u32>()).unwrap();
                (l, MemType::Cons("u32"))
            }
            Type::Cons("u64") => {
                let (l, o) = layout.extend(Layout::new::<u64>()).unwrap();
                (l, MemType::Cons("u64"))
            }
            Type::Cons("f32") => {
                let (l, o) = layout.extend(Layout::new::<f32>()).unwrap();
                (l, MemType::Cons("f32"))
            }
            Type::Tuple(ts) => {
                let mut offset = 0;
                let mut mem_types = Vec::new();
                for t in ts {
                    let (l0, offset_type) = t._layout(layout);
                    let (l1, o1) = layout.extend(l0).unwrap();
                    layout = l1;
                    mem_types.push(offset_type);
                }
                (layout, MemType::Tuple(offset, mem_types))
            }
            _ => unimplemented!(),
        }
    }
}

impl Value {
    fn as_i32(&self) -> i32 {
        match self {
            Value::I32(i) => *i,
            _ => unimplemented!(),
        }
    }

    fn as_tuple(&self) -> Vec<Value> {
        match self {
            Value::Tuple(vs) => vs.clone(),
            _ => unimplemented!(),
        }
    }

    fn as_function(&self) -> &'static str {
        match self {
            Value::FFIFunction(f) => f,
            _ => unimplemented!(),
        }
    }
}

#[test]
fn test_interpret1() {
    let mut lib = load();
    let expr = Expr::FFICall(
        Type::Cons("i32"),
        Box::new(Expr::FFIFunc(Type::Cons("i32"), "add")),
        vec![
            Expr::I32(Type::Cons("i32"), 1i32),
            Expr::I32(Type::Cons("i32"), 2i32),
        ],
    );
    expr.eval(&mut lib);
}
