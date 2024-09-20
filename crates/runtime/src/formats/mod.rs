use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde::Serialize;

pub mod csv {
    pub mod de;
    pub mod ser;
}

pub mod json {
    pub mod de;
    pub mod ser;
}

pub trait Decode {
    type Error: std::error::Error + Send;
    fn decode<'de, T>(&mut self, input: &'de [u8]) -> Result<T, Self::Error>
    where
        T: Deserialize<'de>;

    fn decode_dyn<'de, T, Tag>(
        &mut self,
        input: &'de [u8],
        tag: Tag,
    ) -> Result<T, <Self as Decode>::Error>
    where
        Tag: Clone + DeserializeSeed<'de, Value = T>;
}

pub trait Encode {
    type Error: std::error::Error + Send;
    fn encode<T>(&mut self, input: &T, output: &mut Vec<u8>) -> Result<usize, Self::Error>
    where
        T: Serialize;
    fn content_type(&self) -> &'static str;
}
