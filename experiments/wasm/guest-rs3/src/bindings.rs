// Generated by `wit-bindgen` 0.18.0. DO NOT EDIT!
pub mod exports {
  pub mod pkg {
    pub mod component {
      
      #[allow(clippy::all)]
      pub mod intf {
        #[used]
        #[doc(hidden)]
        #[cfg(target_arch = "wasm32")]
        static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_section;
        
        pub use super::super::super::super::super::Image as Image;
        const _: () = {
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#[dtor]image"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn dtor(rep: usize) {
            wit_bindgen::rt::Resource::<Image>::dtor(rep)
          }
        };
        unsafe impl wit_bindgen::rt::RustResource for Image{
          unsafe fn new(_rep: usize) -> u32 {
            #[cfg(not(target_arch = "wasm32"))]
            unreachable!();
            
            #[cfg(target_arch = "wasm32")]
            {
              #[link(wasm_import_module = "[export]pkg:component/intf")]
              extern "C" {
                #[link_name = "[resource-new]image"]
                fn new(_: usize) -> u32;
              }
              new(_rep)
            }
          }
          
          unsafe fn rep(_handle: u32) -> usize {
            #[cfg(not(target_arch = "wasm32"))]
            unreachable!();
            
            #[cfg(target_arch = "wasm32")]
            {
              #[link(wasm_import_module = "[export]pkg:component/intf")]
              extern "C" {
                #[link_name = "[resource-rep]image"]
                fn rep(_: u32) -> usize;
              }
              rep(_handle)
            }
          }
        }
        pub type OwnImage = wit_bindgen::rt::Resource<Image>;
        
        
        unsafe impl wit_bindgen::rt::WasmResource for Image{
          #[inline]
          unsafe fn drop(_handle: u32) {
            #[cfg(not(target_arch = "wasm32"))]
            unreachable!();
            
            #[cfg(target_arch = "wasm32")]
            {
              #[link(wasm_import_module = "[export]pkg:component/intf")]
              extern "C" {
                #[link_name = "[resource-drop]image"]
                fn drop(_: u32);
              }
              
              drop(_handle);
            }
          }
        }
        
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#print"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_print(arg0: i32,arg1: i32,) {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let len0 = arg1 as usize;
            let bytes0 = Vec::from_raw_parts(arg0 as *mut _, len0, len0);
            <_GuestImpl as Guest>::print(wit_bindgen::rt::string_lift(bytes0));
          }
        };
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#hello"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_hello() -> i32 {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let result0 = <_GuestImpl as Guest>::hello();
            let ptr1 = _RET_AREA.0.as_mut_ptr() as i32;
            let vec2 = (result0.into_bytes()).into_boxed_slice();
            let ptr2 = vec2.as_ptr() as i32;
            let len2 = vec2.len() as i32;
            ::core::mem::forget(vec2);
            *((ptr1 + 4) as *mut i32) = len2;
            *((ptr1 + 0) as *mut i32) = ptr2;
            ptr1
          }
          
          const _: () = {
            #[doc(hidden)]
            #[export_name = "cabi_post_pkg:component/intf#hello"]
            #[allow(non_snake_case)]
            unsafe extern "C" fn __post_return_hello(arg0: i32,) {
              let l0 = *((arg0 + 0) as *const i32);
              let l1 = *((arg0 + 4) as *const i32);
              wit_bindgen::rt::dealloc(l0, (l1) as usize, 1);
            }
          };
        };
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#load-image"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_load_image(arg0: i32,arg1: i32,) -> i32 {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let len0 = arg1 as usize;
            let result1 = <_GuestImpl as Guest>::load_image(Vec::from_raw_parts(arg0 as *mut _, len0, len0));
            wit_bindgen::rt::Resource::into_handle(result1) as i32
          }
        };
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#resize-image"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_resize_image(arg0: i32,arg1: i32,arg2: i32,) -> i32 {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let result0 = <_GuestImpl as Guest>::resize_image(wit_bindgen::rt::Resource::<Image>::lift_borrow(arg0 as u32 as usize), arg1 as u32, arg2 as u32);
            wit_bindgen::rt::Resource::into_handle(result0) as i32
          }
        };
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#image-to-bytes"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_image_to_bytes(arg0: i32,) -> i32 {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let result0 = <_GuestImpl as Guest>::image_to_bytes(wit_bindgen::rt::Resource::<Image>::lift_borrow(arg0 as u32 as usize));
            let ptr1 = _RET_AREA.0.as_mut_ptr() as i32;
            let vec2 = (result0).into_boxed_slice();
            let ptr2 = vec2.as_ptr() as i32;
            let len2 = vec2.len() as i32;
            ::core::mem::forget(vec2);
            *((ptr1 + 4) as *mut i32) = len2;
            *((ptr1 + 0) as *mut i32) = ptr2;
            ptr1
          }
          
          const _: () = {
            #[doc(hidden)]
            #[export_name = "cabi_post_pkg:component/intf#image-to-bytes"]
            #[allow(non_snake_case)]
            unsafe extern "C" fn __post_return_image_to_bytes(arg0: i32,) {
              let l0 = *((arg0 + 0) as *const i32);
              let l1 = *((arg0 + 4) as *const i32);
              let base2 = l0;
              let len2 = l1;
              wit_bindgen::rt::dealloc(base2, (len2 as usize) * 1, 1);
            }
          };
        };
        const _: () = {
          
          #[doc(hidden)]
          #[export_name = "pkg:component/intf#extract-emails"]
          #[allow(non_snake_case)]
          unsafe extern "C" fn __export_extract_emails(arg0: i32,arg1: i32,) -> i32 {
            #[allow(unused_imports)]
            use wit_bindgen::rt::{alloc, vec::Vec, string::String};
            
            // Before executing any other code, use this function to run all static
            // constructors, if they have not yet been run. This is a hack required
            // to work around wasi-libc ctors calling import functions to initialize
            // the environment.
            //
            // This functionality will be removed once rust 1.69.0 is stable, at which
            // point wasi-libc will no longer have this behavior.
            //
            // See
            // https://github.com/bytecodealliance/preview2-prototyping/issues/99
            // for more details.
            #[cfg(target_arch="wasm32")]
            wit_bindgen::rt::run_ctors_once();
            
            let len0 = arg1 as usize;
            let bytes0 = Vec::from_raw_parts(arg0 as *mut _, len0, len0);
            let result1 = <_GuestImpl as Guest>::extract_emails(wit_bindgen::rt::string_lift(bytes0));
            let ptr2 = _RET_AREA.0.as_mut_ptr() as i32;
            let vec4 = result1;
            let len4 = vec4.len() as i32;
            let layout4 = alloc::Layout::from_size_align_unchecked(vec4.len() * 8, 4);
            let result4 = if layout4.size() != 0
            {
              let ptr = alloc::alloc(layout4);
              if ptr.is_null()
              {
                alloc::handle_alloc_error(layout4);
              }
              ptr
            }else {{
              ::core::ptr::null_mut()
            }};
            for (i, e) in vec4.into_iter().enumerate() {
              let base = result4 as i32 + (i as i32) * 8;
              {
                let vec3 = (e.into_bytes()).into_boxed_slice();
                let ptr3 = vec3.as_ptr() as i32;
                let len3 = vec3.len() as i32;
                ::core::mem::forget(vec3);
                *((base + 4) as *mut i32) = len3;
                *((base + 0) as *mut i32) = ptr3;
              }
            }
            *((ptr2 + 4) as *mut i32) = len4;
            *((ptr2 + 0) as *mut i32) = result4 as i32;
            ptr2
          }
          
          const _: () = {
            #[doc(hidden)]
            #[export_name = "cabi_post_pkg:component/intf#extract-emails"]
            #[allow(non_snake_case)]
            unsafe extern "C" fn __post_return_extract_emails(arg0: i32,) {
              let l2 = *((arg0 + 0) as *const i32);
              let l3 = *((arg0 + 4) as *const i32);
              let base4 = l2;
              let len4 = l3;
              for i in 0..len4 {
                let base = base4 + i *8;
                {
                  let l0 = *((base + 0) as *const i32);
                  let l1 = *((base + 4) as *const i32);
                  wit_bindgen::rt::dealloc(l0, (l1) as usize, 1);
                }
              }
              wit_bindgen::rt::dealloc(base4, (len4 as usize) * 8, 4);
            }
          };
        };
        use super::super::super::super::super::Component as _GuestImpl;
        pub trait Guest {
          fn print(input: wit_bindgen::rt::string::String,);
          fn hello() -> wit_bindgen::rt::string::String;
          fn load_image(bytes: wit_bindgen::rt::vec::Vec::<u8>,) -> OwnImage;
          fn resize_image(self_: &Image,width: u32,height: u32,) -> OwnImage;
          fn image_to_bytes(self_: &Image,) -> wit_bindgen::rt::vec::Vec::<u8>;
          fn extract_emails(input: wit_bindgen::rt::string::String,) -> wit_bindgen::rt::vec::Vec::<wit_bindgen::rt::string::String>;
        }
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        
        #[repr(align(4))]
        struct _RetArea([u8; 8]);
        static mut _RET_AREA: _RetArea = _RetArea([0; 8]);
        
      }
      
    }
  }
}

#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:component"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 377] = [0, 97, 115, 109, 13, 0, 1, 0, 0, 25, 22, 119, 105, 116, 45, 99, 111, 109, 112, 111, 110, 101, 110, 116, 45, 101, 110, 99, 111, 100, 105, 110, 103, 4, 0, 7, 250, 1, 1, 65, 2, 1, 65, 2, 1, 66, 17, 4, 0, 5, 105, 109, 97, 103, 101, 3, 1, 1, 64, 1, 5, 105, 110, 112, 117, 116, 115, 1, 0, 4, 0, 5, 112, 114, 105, 110, 116, 1, 1, 1, 64, 0, 0, 115, 4, 0, 5, 104, 101, 108, 108, 111, 1, 2, 1, 112, 125, 1, 105, 0, 1, 64, 1, 5, 98, 121, 116, 101, 115, 3, 0, 4, 4, 0, 10, 108, 111, 97, 100, 45, 105, 109, 97, 103, 101, 1, 5, 1, 104, 0, 1, 64, 3, 4, 115, 101, 108, 102, 6, 5, 119, 105, 100, 116, 104, 121, 6, 104, 101, 105, 103, 104, 116, 121, 0, 4, 4, 0, 12, 114, 101, 115, 105, 122, 101, 45, 105, 109, 97, 103, 101, 1, 7, 1, 64, 1, 4, 115, 101, 108, 102, 6, 0, 3, 4, 0, 14, 105, 109, 97, 103, 101, 45, 116, 111, 45, 98, 121, 116, 101, 115, 1, 8, 1, 112, 115, 1, 64, 1, 5, 105, 110, 112, 117, 116, 115, 0, 9, 4, 0, 14, 101, 120, 116, 114, 97, 99, 116, 45, 101, 109, 97, 105, 108, 115, 1, 10, 4, 1, 18, 112, 107, 103, 58, 99, 111, 109, 112, 111, 110, 101, 110, 116, 47, 105, 110, 116, 102, 5, 0, 4, 1, 23, 112, 107, 103, 58, 99, 111, 109, 112, 111, 110, 101, 110, 116, 47, 99, 111, 109, 112, 111, 110, 101, 110, 116, 4, 0, 11, 15, 1, 0, 9, 99, 111, 109, 112, 111, 110, 101, 110, 116, 3, 0, 0, 0, 70, 9, 112, 114, 111, 100, 117, 99, 101, 114, 115, 1, 12, 112, 114, 111, 99, 101, 115, 115, 101, 100, 45, 98, 121, 2, 13, 119, 105, 116, 45, 99, 111, 109, 112, 111, 110, 101, 110, 116, 6, 48, 46, 50, 49, 46, 48, 16, 119, 105, 116, 45, 98, 105, 110, 100, 103, 101, 110, 45, 114, 117, 115, 116, 6, 48, 46, 49, 56, 46, 48];

#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_section() {}