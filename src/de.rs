use byteorder::{BigEndian, ByteOrder};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::parse_length;

struct Deserializer<'de> {
    input: &'de [u8],
    position: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input, position: 0 }
    }
}

/// Deserialize from bytes
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    if s.len() < 16 {
        return Err(Error::ContentLenght);
    }
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.len() == deserializer.position {
        Ok(t)
    } else {
        Err(Error::ContentLenght)
    }
}

impl<'de> Deserializer<'de> {}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // 不明な型をParseする場合
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // 127以下はbyte長がu8の数値表現そのまま
        if self.input[self.position] != 1 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 1 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = self.input[self.position + 1] != 0;
        self.position += 2;
        visitor.visit_bool(result)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 1 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 1 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = self.input[self.position + 1] as i8;
        self.position += 2;
        visitor.visit_i8(result)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 2 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 2 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_i16(&self.input[self.position + 1..]);
        self.position += 3;
        visitor.visit_i16(result)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 4 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 4 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_i32(&self.input[self.position + 1..]);
        self.position += 5;
        visitor.visit_i32(result)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 8 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 8 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_i64(&self.input[self.position + 1..]);
        self.position += 9;
        visitor.visit_i64(result)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 1 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 1 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = self.input[self.position + 1];
        self.position += 2;
        visitor.visit_u8(result)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 2 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 2 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_u16(&self.input[self.position + 1..]);
        self.position += 3;
        visitor.visit_u16(result)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 4 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 4 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_u32(&self.input[self.position + 1..]);
        self.position += 5;
        visitor.visit_u32(result)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 8 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 8 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_u64(&self.input[self.position + 1..]);
        self.position += 9;
        visitor.visit_u64(result)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 4 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 4 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_f32(&self.input[self.position + 1..]);
        self.position += 5;
        visitor.visit_f32(result)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] != 8 {
            return Err(Error::TypeLength(format!(
                "key: {} expect 8 got {}",
                self.input[self.position - 1],
                self.input[self.position]
            )));
        }
        let result = BigEndian::read_f64(&self.input[self.position + 1..]);
        self.position += 9;
        visitor.visit_f64(result)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (length_len, content_len) =
            parse_length(&self.input[self.position..]).map_err(Error::UnsupportedLength)?;
        let pos = self.position + length_len;
        self.position += length_len + content_len;
        let s = std::str::from_utf8(&self.input[pos..pos + content_len])
            .map_err(|_e| Error::ExpectedString)?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (length_len, content_len) =
            parse_length(&self.input[self.position..]).map_err(Error::UnsupportedLength)?;
        let pos = self.position + length_len;
        self.position += length_len + content_len;
        visitor.visit_borrowed_bytes(&self.input[pos..pos + content_len])
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (length_len, content_len) =
            parse_length(&self.input[self.position..]).map_err(Error::UnsupportedLength)?;
        let pos = self.position + length_len;
        self.position += length_len + content_len;
        visitor.visit_byte_buf(Vec::from(&self.input[pos..pos + content_len]))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input[self.position] == 0 {
            self.position += 1;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.position += 1;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // 数値列はありかも知れない
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.input[self.position];
        let v = BigEndian::read_u32(&self.input[self.position + 1..]);
        let c = std::char::from_u32(v as u32);
        if let Some(x) = c {
            self.position += 1 + len as usize;
            visitor.visit_char(x)
        } else {
            Err(Error::Message(format!(
                "unexpected char {} {}",
                self.input[self.position],
                self.input[self.position + 1]
            )))
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // jsonの場合はtoplevelがMapなのでmapに飛ばしている
        // UniversalKeyとContentLengthを取り出してDeseliarizerに処理を移乗する
        // top levelstructと内蔵のstructで扱いを分ける?
        let key = &self.input[self.position..self.position + 16];
        // BERに従うとする
        let (length_len, content_len) =
            parse_length(&self.input[self.position + 16..]).map_err(Error::UnsupportedLength)?;
        if name.as_bytes() != key {
            return Err(Error::Key(format!(
                "Universal key is unmatched get {:02x?}, expect {:02x?}",
                name.as_bytes(),
                key
            )));
        }
        self.position = 16 + length_len;
        visitor.visit_map(KLVVisitor::new(self, self.position + content_len))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // jsonの場合はdeserialize_strへ飛んでいる
        // Key-Lengthを読み出す関数を作る必要がある
        let v = self.input[self.position];
        self.position += 1;
        visitor.visit_string(v.to_string())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // デシリアライズ先がない場合はデータを無視する
        let (length_len, content_len) =
            parse_length(&self.input[self.position..]).map_err(Error::UnsupportedLength)?;
        self.position += length_len + content_len;
        visitor.visit_unit()
    }
}

struct KLVVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> KLVVisitor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, len }
    }
}

impl<'de, 'a> MapAccess<'de> for KLVVisitor<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.position >= self.len {
            return Ok(None);
        }
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.de.position >= self.len {
            return Err(Error::ExpectedMapEnd);
        }
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

/// Parse for unknown KLVdata
#[derive(Debug)]
pub struct KLVMap<'m> {
    universal_key: &'m [u8],
    content_len: usize,
    values: Vec<KLVRaw<'m>>,
}

impl<'m> KLVMap<'m> {
    /// parse from bytes
    pub fn try_from_bytes(buf: &'m [u8]) -> Result<Self> {
        let buf_len = buf.len();
        if buf_len < 16 {
            return Err(Error::ContentLenght);
        }
        let universal_key = &buf[0..16];
        let (length_len, content_len) =
            parse_length(&buf[16..]).map_err(Error::UnsupportedLength)?;
        let mut position = 16 + length_len;
        let mut values = vec![];
        while position < buf_len {
            let (length_len, content_len) =
                parse_length(&buf[position + 1..]).map_err(Error::UnsupportedLength)?;
            values.push(KLVRaw::from(
                buf[position],
                position + length_len,
                content_len,
                &buf[position + length_len..],
            ));
            position += 1 + length_len + content_len;
        }

        Ok(Self {
            universal_key,
            content_len,
            values,
        })
    }

    /// get universal key
    pub fn universal_key(&'m self) -> &'m [u8] {
        self.universal_key
    }
    /// get content length
    pub fn content_len(&'m self) -> usize {
        self.content_len
    }
    /// iterate KLV records
    pub fn iter(&'m self) -> std::slice::Iter<KLVRaw<'m>> {
        self.values.iter()
    }
}

/// Single KLV Record
#[derive(Debug)]
pub struct KLVRaw<'m> {
    pub key: u8,
    pub position: usize,
    pub length: usize,
    pub value: Option<&'m [u8]>,
}

impl<'m> KLVRaw<'m> {
    pub fn from(key: u8, position: usize, length: usize, value: &'m [u8]) -> Self {
        if length > 0 {
            Self {
                key,
                position,
                length,
                value: Some(&value[..length]),
            }
        } else {
            Self {
                key,
                position,
                length,
                value: None,
            }
        }
    }
}
