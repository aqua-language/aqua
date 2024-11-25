use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Image;",
        codegen: Some(Codegen {
            rust: "Image",
            java: "Image",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[Image]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Image",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def load(blob: Blob): Image;",
                codegen: Some(Codegen {
                    rust: "Image::load",
                    java: "Image.load",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_blob();
                    // Image::new(v0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def crop(img: Image, x: u32, y: u32, w: u32, h: u32): Image;",
                codegen: Some(Codegen {
                    rust: "Image::crop",
                    java: "Image.crop",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_image();
                    // let v1 = v[1].as_u32();
                    // let v2 = v[2].as_u32();
                    // let v3 = v[3].as_u32();
                    // let v4 = v[4].as_u32();
                    // v0.crop(v1, v2, v3, v4).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def centerCrop(img: Image, w: u32, h: u32): Image;",
                codegen: Some(Codegen {
                    rust: "Image::center_crop",
                    java: "Image.centerCrop",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // let v1 = v[1].as_u32();
                        // let v2 = v[2].as_u32();
                        // v0.center_crop(v1, v2).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def resize(img: Image, w: u32, h: u32): Image;",
                codegen: Some(Codegen {
                    rust: "Image::resize",
                    java: "Image.resize",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // let v1 = v[1].as_u32();
                        // let v2 = v[2].as_u32();
                        // v0.resize(v1, v2).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def resizeWidth(img: Image, w: u32): Image;",
                codegen: Some(Codegen {
                    rust: "Image::resize_width",
                    java: "Image.resizeWidth",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // let v1 = v[1].as_u32();
                        // v0.resize_width(v1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def resizeHeight(img: Image, h: u32): Image;",
                codegen: Some(Codegen {
                    rust: "Image::resize_height",
                    java: "Image.resizeHeight",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // let v1 = v[1].as_u32();
                        // v0.resize_height(v1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def into_matrix(img: Image): Matrix[f32];",
                codegen: Some(Codegen {
                    rust: "Image::into_matrix",
                    java: "Image.intoMatrix",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // rt::Matrix::F32(v0.into_matrix()).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def from_matrix(m: Matrix[f32]): Image;",
                codegen: Some(Codegen {
                    rust: "Image::from_matrix",
                    java: "Image.fromMatrix",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_matrix();
                        // if let rt::Matrix::F32(v) = v0 {
                        //     Image::from_matrix(v).into()
                        // } else {
                        //     unreachable!()
                        // }
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def save(img: Image, path: Path): ();",
                codegen: None,
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // let v1 = v[1].as_path();
                        // v0.save(v1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def height(img: Image): u32;",
                codegen: Some(Codegen {
                    rust: "Image::height",
                    java: "Image.height",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // v0.height().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def width(img: Image): u32;",
                codegen: Some(Codegen {
                    rust: "Image::width",
                    java: "Image.width",
                    egglog:None
                }),
                eval: |_ctx, _v| {
                    todo!()
                        // let v0 = v[0].as_image();
                        // v0.width().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua:"def draw_box(img: Image, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]): Image;",
                codegen: Some(Codegen {
                    rust: "Image::draw_box",
                    java: "Image.drawBox",
                    egglog:None
                }),
                eval: |_ctx, _v| {
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
            ImplDecl::Def {
                docs: "",
                aqua:"def preview(img: Image): ();",
                codegen: None,
                eval: |_ctx, _v| {
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
        ],
    });
}
