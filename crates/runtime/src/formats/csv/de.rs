use serde::de::DeserializeSeed;
use serde::Deserialize;

use crate::formats::Decode;

pub struct Reader<const N: usize> {
    inner: csv_core::Reader,
    buffer: [u8; N],
    record_ends: [usize; 20], // To hold the ends of each field
}

pub struct Deserializer<'a, const N: usize> {
    reader: &'a mut Reader<N>,
    input: &'a [u8],
    pub nread: usize,
    current_field_index: usize, // To keep track of the current field index
    total_fields: usize,        // Total number of fields in the current record
}

impl<const N: usize> Reader<N> {
    #[allow(clippy::new_without_default)]
    pub fn new(sep: char) -> Self {
        Self {
            inner: csv_core::ReaderBuilder::new().delimiter(sep as u8).build(),
            buffer: [0; N],
            record_ends: [0; 20],
        }
    }
}

impl<const N: usize> Decode for Reader<N> {
    type Error = Error;
    fn decode<'de, T>(&mut self, input: &'de [u8]) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = Deserializer::new(self, input);
        T::deserialize(&mut deserializer)
    }

    fn decode_dyn<'de, T, Tag: Clone>(&mut self, input: &'de [u8], tag: Tag) -> Result<T>
    where
        Tag: serde::de::DeserializeSeed<'de, Value = T>,
    {
        let mut deserializer = Deserializer::new(self, input);
        serde::de::DeserializeSeed::deserialize(tag, &mut deserializer)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Buffer overflow.
    Overflow,
    /// Expected an empty field.
    ExpectedEmpty,
    /// Invalid boolean value. Expected either `true` or `false`.
    InvalidBool(String),
    /// Invalid integer.
    #[cfg(feature = "opt")]
    InvalidInt(String),
    #[cfg(not(feature = "opt"))]
    InvalidInt(std::num::ParseIntError),
    /// Invalid floating-point number.
    #[cfg(feature = "opt")]
    InvalidFloat(lexical_parse_float::Error),
    #[cfg(not(feature = "opt"))]
    InvalidFloat(std::num::ParseFloatError),
    /// Invalid UTF-8 encoded character.
    InvalidChar(String),
    /// Invalid UTF-8 encoded string.
    InvalidStr(std::str::Utf8Error),
    /// Invalid UTF-8 encoded string.
    InvalidString(std::string::FromUtf8Error),
    /// Error with a custom message had to be discard.
    Custom(String),
    BufferTooSmall,
    EndOfRecord,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Overflow => write!(f, "Buffer overflow."),
            Error::ExpectedEmpty => write!(f, "Expected an empty field."),
            Error::InvalidBool(s) => write!(f, "Invalid bool: {s}"),
            Error::InvalidInt(s) => write!(f, "Invalid integer: {s}"),
            Error::InvalidFloat(e) => write!(f, "Invalid float: {e}"),
            Error::InvalidChar(s) => write!(f, "Invalid character: {s}"),
            Error::InvalidStr(e) => write!(f, "Invalid string: {e}"),
            Error::InvalidString(e) => write!(f, "Invalid string: {e}"),
            Error::Custom(s) => write!(f, "CSV does not match deserializer's expected format: {s}"),
            Error::BufferTooSmall => write!(f, "Buffer too small"),
            Error::EndOfRecord => write!(f, "End of record"),
        }
    }
}

impl serde::de::StdError for Error {}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl<'a, const N: usize> Deserializer<'a, N> {
    pub fn new(reader: &'a mut Reader<N>, input: &'a [u8]) -> Self {
        Self {
            reader,
            input,
            nread: 0,
            current_field_index: 0,
            total_fields: 0,
        }
    }

    fn advance_record(&mut self) -> Result<()> {
        self.current_field_index = 0;

        let (result, r, _w, ends) = self.reader.inner.read_record(
            &self.input[self.nread..],
            &mut self.reader.buffer[0..],
            &mut self.reader.record_ends[0..],
        );
        self.nread += r;
        self.total_fields = ends;

        match result {
            csv_core::ReadRecordResult::InputEmpty => {}
            csv_core::ReadRecordResult::End => {}
            csv_core::ReadRecordResult::OutputFull => return Err(Error::Overflow),
            csv_core::ReadRecordResult::OutputEndsFull => return Err(Error::BufferTooSmall),
            csv_core::ReadRecordResult::Record => {}
        }

        Ok(())
    }

    fn read_bytes(&mut self) -> Result<&[u8]> {
        if self.current_field_index >= self.total_fields {
            return Err(Error::EndOfRecord); // New error variant for end of record
        }

        let start = if self.current_field_index == 0 {
            0
        } else {
            self.reader.record_ends[self.current_field_index - 1]
        };
        let end = self.reader.record_ends[self.current_field_index];
        self.current_field_index += 1;

        Ok(&self.reader.buffer[start..end])
    }

    #[cfg(feature = "opt")]
    fn read_int<T: atoi::FromRadix10SignedChecked>(&mut self) -> Result<T> {
        let bytes = self.read_bytes()?;
        atoi::atoi(bytes)
            .ok_or_else(|| Error::InvalidInt(std::str::from_utf8(bytes).unwrap().to_string()))
    }

    #[cfg(not(feature = "opt"))]
    fn read_int<T: std::str::FromStr<Err = std::num::ParseIntError>>(&mut self) -> Result<T> {
        let bytes = self.read_bytes()?;
        std::str::from_utf8(bytes)
            .unwrap()
            .parse::<T>()
            .map_err(Error::InvalidInt)
    }

    #[cfg(feature = "opt")]
    fn read_float<T: lexical_parse_float::FromLexical>(&mut self) -> Result<T> {
        T::from_lexical(self.read_bytes()?)
            .map_err(|e: lexical_parse_float::Error| Error::InvalidFloat(e))
    }

    #[cfg(not(feature = "opt"))]
    fn read_float<T: std::str::FromStr<Err = std::num::ParseFloatError>>(&mut self) -> Result<T> {
        let bytes = self.read_bytes()?;
        std::str::from_utf8(bytes)
            .unwrap()
            .parse::<T>()
            .map_err(Error::InvalidFloat)
    }

    fn read_bool(&mut self) -> Result<bool> {
        let bytes = self.read_bytes()?;
        match bytes {
            b"true" => Ok(true),
            b"false" => Ok(false),
            _ => Err(Error::InvalidBool(
                std::str::from_utf8(bytes).unwrap().to_string(),
            )),
        }
    }

    fn read_char(&mut self) -> Result<char> {
        let str = self.read_str()?;
        let mut iter = str.chars();
        let c = iter
            .next()
            .ok_or_else(|| Error::InvalidChar(str.to_string()))?;
        if iter.next().is_some() {
            Err(Error::InvalidChar(str.to_string()))
        } else {
            Ok(c)
        }
    }

    fn read_str(&mut self) -> Result<&str> {
        std::str::from_utf8(self.read_bytes()?)
            .map_err(|e: std::str::Utf8Error| Error::InvalidStr(e))
    }

    fn read_string(&mut self) -> Result<String> {
        std::string::String::from_utf8(self.read_bytes()?.to_vec()).map_err(Error::InvalidString)
    }
}

impl<'de, 'a, 'b, const N: usize> serde::de::Deserializer<'de> for &'a mut Deserializer<'b, N> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!("`Deserializer::deserialize_any` is not supported")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(self.read_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.read_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.read_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.read_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.read_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.read_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.read_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.read_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.read_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.read_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.read_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.read_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.read_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.read_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bytes(self.read_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_bytes()?.to_vec())
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
        // if self.peek_bytes()?.is_empty() {
        //     visitor.visit_none()
        // } else {
        //     visitor.visit_some(self)
        // }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if !self.read_bytes()?.is_empty() {
            return Err(Error::ExpectedEmpty);
        }
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!("`Deserializer::deserialize_identifier` is not supported");
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let _ = self.read_bytes()?;
        visitor.visit_unit()
    }
}

impl<'de, 'a, 'b, const N: usize> serde::de::VariantAccess<'de> for &'a mut Deserializer<'b, N> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<U: DeserializeSeed<'de>>(self, _seed: U) -> Result<U::Value> {
        unimplemented!("`VariantAccess::newtype_variant_seed` is not implemented");
    }

    fn tuple_variant<V: serde::de::Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value> {
        unimplemented!("`VariantAccess::tuple_variant` is not implemented");
    }

    fn struct_variant<V: serde::de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        unimplemented!("`VariantAccess::struct_variant` is not implemented");
    }
}

impl<'de, 'a, 'b, const N: usize> serde::de::EnumAccess<'de> for &'a mut Deserializer<'b, N> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        use serde::de::IntoDeserializer;
        let variant_name = self.read_bytes()?;
        seed.deserialize(variant_name.into_deserializer())
            .map(|v| (v, self))
    }
}

impl<'de, 'a, 'b, const N: usize> serde::de::SeqAccess<'de> for &'a mut Deserializer<'b, N> {
    type Error = Error;
    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>>
    where
        V: DeserializeSeed<'de>,
    {
        if self.current_field_index >= self.total_fields {
            self.advance_record()?;
            if self.total_fields == 0 {
                return Ok(None);
            }
        }

        seed.deserialize(&mut **self).map(Some)
    }
}
