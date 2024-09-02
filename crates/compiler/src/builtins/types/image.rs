use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_image(&mut self) {
        self.declare_type(
            "type Image;",
            BuiltinType {
                rust: "image::Image",
            },
        );

        self.declare_def(
            "def load_image(blob: Blob): Image;",
            BuiltinDef {
                rust: "Image::load",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_blob();
                    // Image::new(v0).into()
                },
            },
        );

        self.declare_def(
            "def crop(img: Image, x: u32, y: u32, w: u32, h: u32): Image;",
            BuiltinDef {
                rust: "Image::crop",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // let v2 = v[2].as_u32();
                    // let v3 = v[3].as_u32();
                    // let v4 = v[4].as_u32();
                    // v0.crop(v1, v2, v3, v4).into()
                },
            },
        );

        self.declare_def(
            "def center_crop(img: Image, w: u32, h: u32): Image;",
            BuiltinDef {
                rust: "Image::center_crop",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // let v2 = v[2].as_u32();
                    // v0.center_crop(v1, v2).into()
                },
            },
        );

        self.declare_def(
            "def resize(img: Image, w: u32, h: u32): Image;",
            BuiltinDef {
                rust: "Image::resize",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // let v2 = v[2].as_u32();
                    // v0.resize(v1, v2).into()
                },
            },
        );

        self.declare_def(
            "def resize_width(img: Image, w: u32): Image;",
            BuiltinDef {
                rust: "Image::resize_width",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // v0.resize_width(v1).into()
                },
            },
        );

        self.declare_def(
            "def resize_height(img: Image, h: u32): Image;",
            BuiltinDef {
                rust: "Image::height",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // v0.resize_height(v1).into()
                },
            },
        );

        self.declare_def(
            "def into_matrix(img: Image): Matrix[f32];",
            BuiltinDef {
                rust: "Image::into_matrix",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // rt::Matrix::F32(v0.into_matrix()).into()
                },
            },
        );

        self.declare_def(
            "def from_matrix(m: Matrix[f32]): Image;",
            BuiltinDef {
                rust: "Image::from_matrix",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // if let rt::Matrix::F32(v) = v0 {
                    //     Image::from_matrix(v).into()
                    // } else {
                    //     unreachable!()
                    // }
                },
            },
        );

        self.declare_def(
            "def save(img: Image, path: Path): ();",
            BuiltinDef {
                rust: "Image::from_array",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_path();
                    // v0.save(v1).into()
                },
            },
        );

        self.declare_def(
            "def height(img: Image): u32;",
            BuiltinDef {
                rust: "Image::height",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // v0.height().into()
                },
            },
        );

        self.declare_def(
            "def width(img: Image): u32;",
            BuiltinDef {
                rust: "Image::width",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // v0.width().into()
                },
            },
        );

        self.declare_def(
            "def draw_box(img: Image, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]): Image;",
            BuiltinDef {
                rust: "Image::draw_box",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // let v2 = v[2].as_u32();
                    // let v3 = v[3].as_u32();
                    // let v4 = v[4].as_u32();
                    // let v5 = v[5].as_array().0.iter().map(|v| v.as_u8()).collect::<Vec<_>>().try_into().map(|v: [u8; 4]| Array::from(v)).unwrap();
                    // v0.draw_box(v1, v2, v3, v4, v5.into()).into()
                },
            },
        );

        self.declare_def(
            "def preview(img: Image): ();",
            BuiltinDef {
                rust: "Image::preview",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let conf = viuer::Config {
                    //     // set dimensions
                    //     width: Some(80),
                    //     height: Some(25),
                    //     absolute_offset: false,
                    //     use_kitty: true,
                    //     use_iterm: true,
                    //     .stmt_def_builtinault::default()
                    // };
                    // viuer::print(&v0.0.as_ref().0, &conf).unwrap();
                    // ().into()
                },
            },
        );
    }
}
