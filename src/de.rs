use byteorder::{BigEndian, ByteOrder};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::{check_universal_key_len, parse_length};

struct Deserializer<'de> {
    input: &'de [u8],
    position: usize,
    depth: usize,
    next_len: Vec<(u8, usize)>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input,
            position: 0,
            depth: 0,
            next_len: vec![],
        }
    }
}

/// Deserialize from bytes
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.len() == deserializer.position {
        Ok(t)
    } else {
        Err(Error::ContentLenght)
    }
}

#[cfg(feature = "checksum")]
pub(crate) fn checksum<C: crate::checksum::CheckSumCalc>(s: &[u8], crc: C) -> Result<()> {
    use crate::checksum::CHECKSUM_KEY_LENGTH;

    let checksum_offset = s.len() - 4;
    if &s[checksum_offset..checksum_offset + 2] != CHECKSUM_KEY_LENGTH {
        return Err(Error::HasNotChecksum);
    }
    let crc_value = BigEndian::read_u16(&s[checksum_offset + 2..]);
    let crc_calced = crc.checksum(&s[0..checksum_offset + 2]);
    if crc_value != crc_calced {
        return Err(Error::UnmatcheChecksum {
            value: crc_value,
            calced: crc_calced,
        });
    }
    Ok(())
}

/// Deserialize from bytes
#[cfg(feature = "checksum")]
pub fn from_bytes_with_checksum<'a, T, C: crate::checksum::CheckSumCalc>(
    s: &'a [u8],
    crc: C,
) -> Result<T>
where
    T: Deserialize<'a>,
{
    checksum(s, crc)?;
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
        let result = self.input[self.position] != 0;
        self.position += 1;
        visitor.visit_bool(result)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = self.input[self.position] as i8;
        self.position += 1;
        visitor.visit_i8(result)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_i16(&self.input[self.position..]);
        self.position += 2;
        visitor.visit_i16(result)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_i32(&self.input[self.position..]);
        self.position += 4;
        visitor.visit_i32(result)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_i64(&self.input[self.position..]);
        self.position += 8;
        visitor.visit_i64(result)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = self.input[self.position];
        self.position += 1;
        visitor.visit_u8(result)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_u16(&self.input[self.position..]);
        self.position += 2;
        visitor.visit_u16(result)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_u32(&self.input[self.position..]);
        self.position += 4;
        visitor.visit_u32(result)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_u64(&self.input[self.position..]);
        self.position += 8;
        visitor.visit_u64(result)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_f32(&self.input[self.position..]);
        self.position += 4;
        visitor.visit_f32(result)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = BigEndian::read_f64(&self.input[self.position..]);
        self.position += 8;
        visitor.visit_f64(result)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (_key, len) = self.next_len.pop().ok_or(Error::NeedKey)?;
        let s = std::str::from_utf8(&self.input[self.position..self.position + len])
            .map_err(|_e| Error::ExpectedString)?;
        self.position += len;
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
        let (_key, len) = self.next_len.pop().ok_or(Error::NeedKey)?;
        let b = &self.input[self.position..self.position + len];
        self.position += len;
        visitor.visit_borrowed_bytes(b)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (_key, len) = self.next_len.pop().ok_or(Error::NeedKey)?;
        let b = &self.input[self.position..self.position + len];
        self.position += len;
        visitor.visit_byte_buf(Vec::from(b))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (_key, len) = self.next_len.last().ok_or(Error::NeedKey)?;
        if len == &0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // ある長さまでシリアライズを続ける
        let (_key, len) = self.next_len.last().ok_or(Error::NeedKey)?;
        visitor.visit_seq(KLVVisitor::new(self, self.position + len))
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
        unimplemented!()
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
        let (_key, len) = self.next_len.last().ok_or(Error::NeedKey)?;
        let v = BigEndian::read_u32(&self.input[self.position..]);
        let c = std::char::from_u32(v);
        if let Some(x) = c {
            self.position += len;
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
        // 0階層目のみUniversalKeyが存在する
        // それより深い階層は構造体定義にのみ依存するためUniverslkeyを必要としない
        if self.position == 0 {
            let key_len = check_universal_key_len(name)?;
            if self.input.len() <= key_len {
                return Err(Error::ContentLenght);
            }
            let key = &self.input[self.position..self.position + key_len];
            let (length_len, content_len) = parse_length(&self.input[self.position + key_len..])
                .map_err(Error::UnsupportedLength)?;
            if name.as_bytes() != key {
                return Err(Error::Key(format!(
                    "Universal key is unmatched get {:02x?}, expect {:02x?}",
                    name.as_bytes(),
                    key
                )));
            }
            self.position = key_len + length_len;
            self.depth += 1;
            visitor.visit_map(KLVVisitor::new(self, self.position + content_len))
        } else {
            self.depth += 1;
            let (_key, len) = self.next_len.last().ok_or(Error::NeedKey)?;
            visitor.visit_map(KLVVisitor::new(self, self.position + len))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // jsonの場合はdeserialize_strへ飛んでいる
        let v = self.input[self.position];
        let (length_len, content_len) =
            parse_length(&self.input[self.position + 1..]).map_err(Error::UnsupportedLength)?;
        self.position += 1 + length_len;
        // 不定長データstructやstringなどの読み出し範囲として記録
        self.next_len.push((v, content_len));
        visitor.visit_string(v.to_string())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // デシリアライズ先がない場合はデータを無視する
        let (_key, len) = self.next_len.last().ok_or(Error::NeedKey)?;
        self.position += len;
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
        if self.de.position >= self.len {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // >=ではないのはunitのような長さ0のデータが末尾に来る場合に
        // positionがValueの位置ではなくlenを超えた次のKeyに来るため
        if self.de.position > self.len {
            return Err(Error::ExpectedMapEnd);
        }
        let v = seed.deserialize(&mut *self.de)?;
        self.de.next_len.pop();
        Ok(v)
    }
}

impl<'de, 'a> SeqAccess<'de> for KLVVisitor<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.de.position {
            x if x < self.len => {}
            x if x == self.len => return Ok(None),
            x if x > self.len => return Err(Error::ExpectedSeqEnd),
            _ => unreachable!(),
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

/// Parse unknown KLVdata
///
/// Example
/// ```
/// use serde_klv::KLVMap;
///
/// let buf = vec![0,0,0,0,3,10,1,128];
/// let x = KLVMap::try_from_bytes(&buf).unwrap();
/// assert_eq!(x.universal_key(), "\0\0\0\0".as_bytes());
/// assert_eq!(x.content_len(), 3);
/// assert_eq!(x.iter().len(), 1);
/// ```
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
        // key長探索
        let uk_len = Self::find_universal_key(buf)?;
        let universal_key = &buf[0..uk_len];
        let (length_len, content_len) =
            parse_length(&buf[uk_len..]).map_err(Error::UnsupportedLength)?;
        let mut position = uk_len + length_len;
        let mut values = vec![];
        while position < buf_len {
            let (length_len, content_len) =
                parse_length(&buf[position + 1..]).map_err(Error::UnsupportedLength)?;
            values.push(KLVRaw::from(
                buf[position],
                position,
                content_len,
                &buf[position + 1 + length_len..],
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

    // データからUniversalKeyの長さを取り出す
    fn find_universal_key(buf: &'m [u8]) -> Result<usize> {
        let buf_len = buf.len();
        for l in [1, 2, 4, 16] {
            // バッファの長さが想定する長さより短い
            if l >= buf_len {
                break;
            }
            let (lenght_len, content_len) =
                parse_length(&buf[l..]).map_err(Error::UnsupportedLength)?;
            if buf_len == l + lenght_len + content_len {
                return Ok(l);
            }
        }
        Err(Error::ContentLenght)
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
