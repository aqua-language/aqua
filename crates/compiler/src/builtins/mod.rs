use crate::Compiler;

mod conv;
pub mod decls;
pub mod value;

pub mod traits {
    use crate::Compiler;

    mod add;
    mod clone;
    mod copy;
    mod debug;
    mod deep_clone;
    mod default;
    mod display;
    mod div;
    mod eq;
    mod fun;
    mod hash;
    mod into_iterator;
    mod iterator;
    mod mul;
    mod neg;
    mod not;
    mod ord;
    mod partial_eq;
    mod partial_ord;
    mod serde;
    mod sub;

    impl Compiler {
        pub(super) fn declare_traits(&mut self) {
            self.declare_add();
            self.declare_sub();
            self.declare_mul();
            self.declare_div();
            self.declare_neg();
            self.declare_not();
            self.declare_ord();
            self.declare_clone();
            self.declare_copy();
            self.declare_debug();
            self.declare_deep_clone();
            self.declare_display();
            self.declare_eq();
            self.declare_into_iterator();
            self.declare_iterator();
            self.declare_partial_eq();
            self.declare_partial_ord();
            self.declare_serde();
            self.declare_default();
        }
    }
}

pub mod types {
    use crate::Compiler;

    pub mod aggregator;
    pub mod array;
    pub mod blob;
    pub mod bool;
    pub mod char;
    pub mod dataflow;
    pub mod dict;
    pub mod discretizer;
    pub mod duration;
    pub mod encoding;
    pub mod f32;
    pub mod f64;
    pub mod file;
    pub mod function;
    pub mod i128;
    pub mod i16;
    pub mod i32;
    pub mod i64;
    pub mod i8;
    pub mod image;
    pub mod instance;
    pub mod keyed_stream;
    pub mod matrix;
    pub mod model;
    pub mod never;
    pub mod option;
    pub mod ordering;
    pub mod path;
    pub mod range;
    pub mod reader;
    pub mod record;
    pub mod result;
    pub mod set;
    pub mod socket;
    pub mod stream;
    pub mod string;
    pub mod time;
    pub mod time_source;
    pub mod tuple;
    pub mod u128;
    pub mod u16;
    pub mod u32;
    pub mod u64;
    pub mod u8;
    pub mod unit;
    pub mod url;
    pub mod usize;
    pub mod variant;
    pub mod vec;
    pub mod writer;
    pub mod backend;

    impl Compiler {
        pub(super) fn declare_types(&mut self) {
            // self.declare_aggregator();
            // self.declare_array();
            // self.declare_blob();
            self.declare_bool();
            self.declare_char();
            // self.declare_dict();
            self.declare_dataflow();
            // self.declare_discretizer();
            // self.declare_duration();
            self.declare_encoding();
            self.declare_f32();
            self.declare_f64();
            // self.declare_file();
            // self.declare_function();
            self.declare_i128();
            // self.declare_i16();
            self.declare_i32();
            self.declare_i64();
            // self.declare_i8();
            // self.declare_image();
            self.declare_instance();
            // self.declare_keyed_stream();
            // self.declare_matrix();
            // self.declare_model();
            // self.declare_never();
            self.declare_option();
            self.declare_path();
            self.declare_reader();
            // self.declare_record();
            // self.declare_result();
            // self.declare_set();
            // self.declare_socket();
            self.declare_stream();
            self.declare_string();
            self.declare_time();
            // self.declare_time_source();
            self.declare_traits();
            // self.declare_tuple();
            // self.declare_u128();
            // self.declare_u16();
            // self.declare_u32();
            // self.declare_u64();
            // self.declare_u8();
            // self.declare_unit();
            // self.declare_url();
            self.declare_usize();
            // self.declare_variant();
            self.declare_vec();
            self.declare_writer();
            self.declare_ordering();
        }
    }
}

impl Compiler {
    pub fn declare(&mut self) {
        self.declare_types();
        self.declare_traits();
    }
}
