use serde_json::Deserializer;

use crate::formats::Decode;

#[derive(Default)]
pub struct Reader {}

impl Reader {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Decode for Reader {
    type Error = serde_json::Error;

    fn decode<'de, T>(&mut self, input: &'de [u8]) -> Result<T, Self::Error>
    where
        T: serde::Deserialize<'de>,
    {
        let mut deserializer = Deserializer::from_slice(input);
        let value = T::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(value)
    }
}
