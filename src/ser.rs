use std::collections::BTreeSet;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{ser, Serialize};

use crate::{
    check_universal_key_len,
    error::{Error, Result},
    LengthOctet,
};

/// Serialize to bytes
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = KLVSerializer::default();
    value.serialize(&mut serializer)?;
    // ここでKeyを合成するのが良さそう
    Ok(serializer.concat())
}

/// Serialize to bytes append CRC at last field
#[cfg(feature = "checksum")]
pub fn to_bytes_with_crc<T, C: crate::checksum::CheckSumCalc>(value: &T, calc: C) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut reserved_key = BTreeSet::new();
    reserved_key.insert(0x01);
    let mut serializer = KLVSerializer::with_reserved_key(reserved_key);
    value.serialize(&mut serializer)?;
    // ここでKeyを合成するのが良さそう
    Ok(serializer.concat_with_checksum(calc))
}

// KLVシリアライザ
// 基本的にはKLVのうちVを行う
// structに限りKLの処理が必要でTopLevelだけはuniversal_keyで保持する
// それより深い階層では個別のキーではなく親のkey
#[derive(Debug)]
struct KLVSerializer {
    universal_key: Vec<u8>,
    // 現在の階層深さ。KLのためには1階層以上でなければならない
    depth: usize,
    // 各階層ごとのKLVシリアライズ結果
    // KLVはVをシリアライズするまでLが分からないため
    // depth階層のbufferをVのシリアライズ領域に使い
    // Vのシリアライズが終わったらその長さを元にLを算出し
    // depth-1階層にKLVで書き込む
    output: Vec<Vec<u8>>,
    // 各層毎の使用済みKeyマップ
    keys: Vec<BTreeSet<u8>>,
    // checksumのような予約済みのキー
    reserved_key: BTreeSet<u8>,
}

impl Default for KLVSerializer {
    fn default() -> Self {
        Self {
            universal_key: vec![],
            depth: 0,
            output: vec![vec![]],
            keys: vec![BTreeSet::new()],
            reserved_key: BTreeSet::new(),
        }
    }
}

impl KLVSerializer {
    fn with_reserved_key(reserved_key: BTreeSet<u8>) -> Self {
        Self {
            universal_key: vec![],
            depth: 0,
            output: vec![vec![]],
            keys: vec![BTreeSet::new()],
            reserved_key,
        }
    }
    fn next_depth(&mut self) {
        self.depth += 1;
        self.output.push(vec![]);
        self.keys.push(BTreeSet::new());
    }
    fn end_depth(&mut self) -> Result<()> {
        let _cache = self.output.pop().unwrap();
        let _keys = self.keys.pop().unwrap();
        self.depth -= 1;
        Ok(())
    }
    fn write_key(&mut self, key: u8) -> Result<()> {
        let index = self.depth - 1;
        if index == 0 && self.reserved_key.contains(&key) {
            return Err(Error::Key(format!("key is reserved: {}", key)));
        }
        if let Some(n) = self.keys.get_mut(index) {
            if !n.insert(key) {
                return Err(Error::Key(format!(
                    "already use field {} in depth {}",
                    key, index
                )));
            }
        } else {
            return Err(Error::Message("has not key map".to_string()));
        }
        self.output[index].push(key);
        Ok(())
    }
    fn get_cache(&mut self) -> Result<&mut Vec<u8>> {
        Ok(self.output.last_mut().unwrap())
    }
    fn write_lv(&mut self) -> Result<()> {
        // self outputを&mut参照するのでmutable制限を超えるためにcacheを一度取り出す
        let mut cache = self.output.pop().unwrap();
        let output = self.output.last_mut().unwrap();
        let len = cache.len();
        let _len = LengthOctet::length_to_buf(output, len).map_err(Error::IO)?;
        output.append(&mut cache);
        self.output.push(cache);
        Ok(())
    }
    fn concat(self) -> Vec<u8> {
        let Self {
            universal_key: mut key,
            mut output,
            ..
        } = self;
        let output = output.pop().unwrap();
        LengthOctet::length_to_buf(&mut key, output.len()).unwrap();
        key.extend_from_slice(&output);
        key
    }
    // checksum付きのEncode
    // MISB ST 0601.8の仕様に近いものとし、ChecksumTagのL部分までがchecksum計算の対象とする
    #[cfg(feature = "checksum")]
    fn concat_with_checksum<C: crate::checksum::CheckSumCalc>(self, crc: C) -> Vec<u8> {
        use crate::checksum::CHECKSUM_KEY_LENGTH;

        let Self {
            universal_key: mut key,
            mut output,
            ..
        } = self;
        let output = output.pop().unwrap();
        // 4 = K + L + V(2)
        LengthOctet::length_to_buf(&mut key, output.len() + 4).unwrap();
        key.extend_from_slice(&output);
        key.extend_from_slice(CHECKSUM_KEY_LENGTH);
        // calc checksum and write
        let crc_code = crc.checksum(&key);
        key.write_u16::<byteorder::BigEndian>(crc_code).unwrap();
        key
    }
}

// TODO
// V変換を普通にやる
// StructはV結果を見てLを決める
impl<'a> ser::Serializer for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.get_cache()?.push(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.get_cache()?.push(v as u8);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.get_cache()?
            .write_i16::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error i16 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.get_cache()?
            .write_i32::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error i32 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.get_cache()?
            .write_i64::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error i64 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.get_cache()?.push(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.get_cache()?
            .write_u16::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error u16 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.get_cache()?
            .write_u32::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error u32 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.get_cache()?
            .write_u64::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error u64 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.get_cache()?
            .write_f32::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error f32 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.get_cache()?
            .write_f64::<BigEndian>(v)
            .map_err(|e| Error::Encode(format!("encodind error f64 {v} to byte. {e}")))?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_u32(v as u32)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.get_cache()?.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.get_cache()?.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
        // // UnitなEnumは名前が固定で実装に依存して値が可変であるためstrで保持するのが一般的
        // println!("serialize_unit_variant {} {}", _name, variant);
        // self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported("map is not supported".to_string()))
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        if self.depth == 0 {
            check_universal_key_len(name)?;
            self.universal_key.extend_from_slice(name.as_bytes())
        }
        self.next_depth();
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeStruct for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key
            .parse::<u8>()
            .map_err(|e| Error::Key(format!("failed t kparse key str to u8 {} {}", key, e)))?;

        // cacheにValue書き出し
        value.serialize(&mut **self)?;
        // outputにKey書き出し
        self.write_key(key)?;
        // outputにLengthValue書き出し
        self.write_lv()
    }

    fn end(self) -> Result<()> {
        // まだ階層が低い。ここではStructのKeyを書いてCacheをLVする必要がある
        self.end_depth()?;
        Ok(())
    }
}

// 個別のLは省略する
// LはSeq全体長のみ、Vは全て同じ型とする
impl<'a> ser::SerializeSeq for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Seqと同じく個別のLを省略する
// シリアライズ、デシリアライズの型が同じなら長さは自明となる
impl<'a> ser::SerializeTuple for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut KLVSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use serde::{Deserialize, Serialize};

    use crate::de::{from_bytes, KLVMap};
    use crate::error::Error;
    use crate::ser::{to_bytes, KLVSerializer};

    // データが空でもエラーにならないこと
    #[test]
    fn test_empty() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "DUMY")]
        struct TestEmpty {
            #[serde(rename = "1", skip_serializing_if = "Option::is_none")]
            one: Option<String>,
            #[serde(rename = "2", skip_serializing_if = "Option::is_none")]
            two: Option<String>,
        }

        let t = TestEmpty {
            one: None,
            two: None,
        };
        let s = to_bytes(&t).unwrap();
        assert_eq!(s.len(), 5);
        let x = from_bytes::<TestEmpty>(&s).unwrap();
        assert_eq!(x, t);
        let x = KLVMap::try_from_bytes(&s).unwrap();
        assert_eq!(x.content_len(), 0);
        assert_eq!(x.iter().len(), 0);
    }

    #[test]
    fn test_serialize_error_by_key() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestKeyRangeOutFromU8 {
            #[serde(rename = "-1")]
            x: bool,
        }

        let t = TestKeyRangeOutFromU8 { x: true };
        let res = to_bytes(&t);
        match res {
            Err(Error::Key(_)) => {}
            _ => unreachable!(),
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestForgetRename {
            bbb: bool,
        }
        let t = TestForgetRename { bbb: true };
        let res = to_bytes(&t);
        match res {
            Err(Error::Key(_)) => {}
            _ => unreachable!(),
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestSameName {
            #[serde(rename = "10")]
            bbb: bool,
            #[serde(rename = "10")]
            u8: u8,
        }
        let t = TestSameName { bbb: true, u8: 128 };
        let res = to_bytes(&t);
        match res {
            Err(Error::Key(_)) => {}
            _ => unreachable!(),
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestNoUniversalKey {
            #[serde(rename = "10")]
            bbb: bool,
        }
        let t = TestNoUniversalKey { bbb: true };
        let res = to_bytes(&t);
        match res {
            Err(Error::Key(_)) => {}
            _ => unreachable!(),
        }

        //
        // Check same field struct other UniversalKey
        //
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestRef {
            #[serde(rename = "10")]
            bbb: bool,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000001")]
        struct TestTargetOtherUniversalKey {
            #[serde(rename = "10")]
            bbb: bool,
        }
        let t = TestRef { bbb: true };
        let reference = to_bytes(&t).unwrap();

        let res = from_bytes::<TestTargetOtherUniversalKey>(&reference);
        match res {
            Err(Error::Key(_)) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_serialize_str() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestStr<'a> {
            #[serde(rename = "30")]
            str: &'a str,
        }
        let t = TestStr {
            str: "this is str\09joi4t@",
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestStr>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[test]
    fn test_serialize_char() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestChar {
            #[serde(rename = "30")]
            char8: char,
            #[serde(rename = "31")]
            char16: char,
            #[serde(rename = "32")]
            char32: char,
        }
        let t = TestChar {
            char8: '\n',
            char16: std::char::from_u32(257).unwrap(),
            char32: std::char::from_u32(u16::MAX as u32 + 1).unwrap(),
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestChar>(&s).unwrap();
        assert_eq!(t, x);
    }
    #[test]
    fn test_serialize_optional_string() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestString {
            #[serde(rename = "30")]
            string: String,
            #[serde(rename = "31")]
            some: Option<String>,
            #[serde(rename = "32")]
            none: Option<String>,
            #[serde(rename = "120", skip_serializing_if = "Option::is_none")]
            none_skip_none: Option<String>,
            #[serde(rename = "121", skip_serializing_if = "Option::is_none")]
            none_skip_some: Option<String>,
        }
        let t = TestString {
            string: "this is String".to_string(),
            some: Some("this is Some".to_string()),
            none: None,
            none_skip_none: None,
            none_skip_some: Some("none skip".to_string()),
        };
        let s = to_bytes(&t).unwrap();
        // skipしない場合はLength=0
        assert!(find_subsequence(&s, &[32, 0]).is_some());
        // skipする場合はKey自体が存在しない
        assert!(find_subsequence(&s, &[120, 0]).is_none());
        // データがある場合はskipされない
        assert!(find_subsequence(&s, &[121, 9]).is_some());
        let x = from_bytes::<TestString>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[test]
    fn test_serialize_timestamp_micro() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestTimestamp<'a> {
            #[serde(rename = "30")]
            str: &'a str,
            #[serde(rename = "31", with = "timestamp_micro")]
            ts: SystemTime,
        }
        let t = TestTimestamp {
            str: "TestTimestamp struct",
            ts: SystemTime::now(),
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestTimestamp>(&s).unwrap();
        assert_eq!(t.str, x.str);
        let t_micros =
            t.ts.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
        let x_micros =
            t.ts.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
        assert_eq!(t_micros, x_micros);
    }

    #[test]
    fn test_serialize_non_ascii_universal_key() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "\x06\x0e\x2b\x34\x02\x0b\x01\x01\x0e\x01\x0e\x01\x01\x01\x00\x00")]
        struct TestTimestamp<'a> {
            #[serde(rename = "30")]
            str: &'a str,
        }
        let t = TestTimestamp {
            str: "TestTimestamp struct",
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestTimestamp>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[test]
    fn test_serialize_bytes_any() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestTimestamp<'a> {
            #[serde(rename = "60", with = "serde_bytes")]
            byte_slice: &'a [u8],
            #[serde(rename = "70", with = "serde_bytes")]
            bytes: Vec<u8>,
            #[serde(rename = "71")]
            unit: (),
        }
        let t = TestTimestamp {
            byte_slice: &[255, 128, 64, 32],
            bytes: vec![0, 1, 2, 4, 8, 16, 32, 64],
            unit: (),
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestTimestamp>(&s).unwrap();
        assert_eq!(t, x);
    }

    /// デシリアライズ時に欠損や過剰なデータなどの非対称性があるデータ
    #[test]
    fn test_serialize_asymmetry() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestLarge {
            #[serde(rename = "30")]
            require: u16,
            #[serde(rename = "31")]
            some: Option<u16>,
            #[serde(rename = "32")]
            none: Option<u16>,
            #[serde(rename = "120", skip_serializing_if = "Option::is_none")]
            none_skip_none: Option<u16>,
            #[serde(rename = "121", skip_serializing_if = "Option::is_none")]
            none_skip_some: Option<u16>,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestShort {
            #[serde(rename = "30")]
            require: u16,
        }
        let t = TestLarge {
            require: 123,
            some: Some(345),
            none: None,
            none_skip_none: None,
            none_skip_some: Some(678),
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<TestShort>(&s).unwrap();
        assert_eq!(t.require, x.require);
    }

    #[test]
    fn test_serialize_dump() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestLarge<'a> {
            #[serde(rename = "10")]
            u8: u8,
            #[serde(rename = "11")]
            u64: u64,
            #[serde(rename = "31")]
            some: Option<u16>,
            #[serde(rename = "32")]
            none: Option<u16>,
            #[serde(rename = "120", skip_serializing_if = "Option::is_none")]
            none_skip_some: Option<u16>,
            #[serde(rename = "121", skip_serializing_if = "Option::is_none")]
            none_skip_none: Option<u16>,
            #[serde(rename = "60")]
            str: &'a str,
            #[serde(rename = "61", with = "serde_bytes")]
            bytes: &'a [u8],
            #[serde(rename = "62", with = "timestamp_micro")]
            ts: SystemTime,
            #[serde(rename = "63")]
            child: TestChild,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestChild {
            #[serde(rename = "10")]
            string: String,
            #[serde(rename = "11")]
            i8: i8,
        }

        let ts = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_micros(1_000_233_000))
            .unwrap();
        let t = TestLarge {
            u8: 127,
            u64: u32::MAX as u64 + 1,
            some: Some(1016),
            none: None,
            none_skip_some: Some(2016),
            none_skip_none: None,
            str: "this is string",
            bytes: b"this is byte",
            ts,
            child: TestChild {
                string: "TestString".to_string(),
                i8: 127,
            },
        };
        let s = to_bytes(&t).unwrap();
        let x = KLVMap::try_from_bytes(&s).unwrap();

        assert_eq!(x.universal_key(), "TESTDATA00000000".as_bytes());
        assert!(x.content_len() > 0);
        assert_eq!(x.iter().len(), 9);

        for v in x.iter() {
            assert!(v.key > 0);
            println!("{:?}", v);
        }
    }

    mod timestamp_micro {
        use std::time::{Duration, SystemTime};

        use serde::{Deserialize, Deserializer, Serializer};

        pub fn serialize<S>(date: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let micros = date
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            serializer.serialize_u64(micros as u64)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
        where
            D: Deserializer<'de>,
        {
            let micros = u64::deserialize(deserializer)?;
            SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_micros(micros))
                .ok_or_else(|| serde::de::Error::custom("failed to deserialize systemtime"))
        }
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestParent {
            #[serde(rename = "10")]
            i8: i8,
            #[serde(rename = "11")]
            i64: i64,
            #[serde(rename = "20")]
            child: Option<TestChild>,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestChild {
            #[serde(rename = "10")]
            i16: i16,
            #[serde(rename = "11")]
            i32: i32,
        }

        let t = TestParent {
            i8: -64,
            i64: 1 + 2_i64.pow(16) + 2_i64.pow(32) + 2_i64.pow(48),
            child: Some(TestChild { i16: 16, i32: 32 }),
            // child: None,
        };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        assert!(find_subsequence(
            serializer.get_cache().unwrap(),
            &[20, 10, 10, 2, 0, 16, 11, 4, 0, 0, 0, 32]
        )
        .is_some());
        let s = serializer.concat();
        let x = from_bytes::<TestParent>(&s).unwrap();
        assert_eq!(t, x);
    }
    #[test]
    fn test_sequence() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestParent {
            #[serde(rename = "10")]
            i8: i8,
            #[serde(rename = "11")]
            i64: i64,
            #[serde(rename = "20")]
            seq: Option<Vec<i32>>,
        }

        let t = TestParent {
            i8: -64,
            i64: 1 + 2_i64.pow(16) + 2_i64.pow(32) + 2_i64.pow(48),
            seq: Some(vec![
                1,
                2_i32.pow(8) + 1,
                2_i32.pow(16) + 1,
                2_i32.pow(24) + 1,
            ]),
            // child: None,
        };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        assert!(find_subsequence(
            serializer.get_cache().unwrap(),
            &[20, 16, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1]
        )
        .is_some());
        let s = serializer.concat();
        let x = from_bytes::<TestParent>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[test]
    fn test_tuple() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestParent {
            #[serde(rename = "10")]
            i8: i8,
            #[serde(rename = "11")]
            i64: i64,
            #[serde(rename = "20")]
            seq: Option<(i8, i16, i32, i64)>,
        }

        let t = TestParent {
            i8: -64,
            i64: 1 + 2_i64.pow(16) + 2_i64.pow(32) + 2_i64.pow(48),
            seq: Some((i8::MIN, i16::MIN, i32::MIN, i64::MIN)),
            // child: None,
        };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        assert!(find_subsequence(
            serializer.get_cache().unwrap(),
            &[20, 15, 128, 128, 0, 128, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0]
        )
        .is_some());
        let s = serializer.concat();
        let x = from_bytes::<TestParent>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Abxy(i8, i16, i32, i64);

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestParent {
            #[serde(rename = "10")]
            i8: i8,
            #[serde(rename = "11")]
            i64: i64,
            #[serde(rename = "20")]
            seq: Option<Abxy>,
        }

        let t = TestParent {
            i8: -64,
            i64: 1 + 2_i64.pow(16) + 2_i64.pow(32) + 2_i64.pow(48),
            seq: Some(Abxy(i8::MIN, i16::MIN, i32::MIN, i64::MIN)),
            // child: None,
        };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        assert!(find_subsequence(
            serializer.get_cache().unwrap(),
            &[20, 15, 128, 128, 0, 128, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0]
        )
        .is_some());
        let s = serializer.concat();
        let x = from_bytes::<TestParent>(&s).unwrap();
        assert_eq!(t, x);
    }

    #[ignore]
    #[test]
    fn test_enum() {
        #[repr(u8)]
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum V {
            A = 1,
            MediumLongVariant = 20,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestVariant {
            #[serde(rename = "10")]
            va: V,
            #[serde(rename = "11")]
            vb: V,
        }
        let t = TestVariant {
            va: V::A,
            vb: V::MediumLongVariant,
        };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        assert!(find_subsequence(
            serializer.get_cache().unwrap(),
            &[
                10, 1, 65, 11, 17, 77, 101, 100, 105, 117, 109, 76, 111, 110, 103, 86, 97, 114,
                105, 97, 110, 116
            ]
        )
        .is_some());
        let s = serializer.concat();
        let x = from_bytes::<TestVariant>(&s).unwrap();
        assert_eq!(t, x);
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
