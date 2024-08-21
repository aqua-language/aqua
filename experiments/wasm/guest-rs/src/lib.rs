pub struct Image(DynamicImage);

wit_bindgen::generate!({
    world: "component",
    exports: {
        "intf": Component,
        "intf/image": Image
    },
});

use exports::intf::Guest;
use image::DynamicImage;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::Cursor;
use wit_bindgen::Resource;

struct Component;

static REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Za-z0-9-_.]+@[A-Za-z0-9-_.]+\.[A-Za-z0-9]{2,4}").unwrap());

impl Guest for Component {
    fn print(input: String) {
        println!("{}", input);
    }

    fn hello() -> String {
        println!("Hello from Rust (WASI TEST)!");
        "Hello from Rust!".to_owned()
    }

    fn extract_emails(input: String) -> Vec<String> {
        REGEX
            .find_iter(&input)
            .map(|m| m.as_str().to_owned())
            .collect()
    }

    fn load_image(bytes: Vec<u8>) -> Resource<Image> {
        let img = Image(image::load_from_memory(&bytes).unwrap());
        let img = wit_bindgen::Resource::new(img);
        img
    }

    fn resize_image(this: &Image, width: u32, height: u32) -> Resource<Image> {
        let img = this
            .0
            .resize(width, height, image::imageops::FilterType::Lanczos3);
        let img = wit_bindgen::Resource::new(Image(img));
        img
    }

    fn image_to_bytes(this: &Image) -> Vec<u8> {
        let mut buf = Vec::new();
        this.0
            .write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)
            .unwrap();
        buf
    }
}

#[test]
fn test_extract_emails() {
    let input = "john.doe@gmail.com and jane.doe@gmail.com".to_owned();
    let expected = vec![
        "john.doe@gmail.com".to_owned(),
        "jane.doe@gmail.com".to_owned(),
    ];
    assert_eq!(Component::extract_emails(input), expected);
}
