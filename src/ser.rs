use std::collections::BTreeSet;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{ser, Serialize};

use crate::{
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

// TopLevelのシリアライザ
#[derive(Debug)]
struct KLVSerializer {
    universal_key: Vec<u8>,
    output: Vec<Vec<u8>>,
    depth: usize,
    keys: Vec<BTreeSet<u8>>,
}

impl Default for KLVSerializer {
    fn default() -> Self {
        Self {
            universal_key: vec![],
            output: vec![vec![]],
            depth: 0,
            keys: vec![BTreeSet::new()],
        }
    }
}

impl KLVSerializer {
    fn next_depth(&mut self) {
        self.depth += 1;
        self.output.push(vec![]);
        self.keys.push(BTreeSet::new());
    }
    fn end_depth(&mut self) -> Result<()> {
        println!("end_depth {:?}", self.output);
        let _cache = self.output.pop().unwrap();
        let _keys = self.keys.pop().unwrap();
        self.depth -= 1;
        Ok(())
    }
    fn write_key(&mut self, key: u8) -> Result<()> {
        let index = self.depth - 1;
        if let Some(n) = self.keys.get_mut(index) {
            if !n.insert(key) {
                return Err(Error::Key(format!(
                    "already use field {} in depth {} ",
                    key, index
                )));
            }
        } else {
            return Err(Error::Message("has not key map".to_string()));
        }
        println!("write_key d: {}, k: {}", index, key);
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
        println!("serialize_i32 {}", v);
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
        println!("serialize_u32 {}", v);
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
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
        // println!("serialize_unit_variant {} {}", _name, _variant);
        // variant_index.serialize(self)
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
        println!("serialize_seq");
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        println!("serialize_tuple");
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
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
        unimplemented!()
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        println!("serialize_struct");
        match name.len() {
            1 | 2 | 4 | 16 => {}
            _ => {
                return Err(Error::Key(format!(
                    "universal key support length only {{1,2,4,16}} got {}",
                    name
                )))
            }
        }
        if self.depth == 0 {
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

        // cacheに書き出し
        value.serialize(&mut **self)?;
        // key書き出し
        self.write_key(key)?;
        // lv書き出し
        self.write_lv()
    }

    fn end(self) -> Result<()> {
        // まだ階層が低い。ここではStructのKeyを書いてCacheをLVする必要がある
        self.end_depth()?;
        Ok(())
    }
}

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

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
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

    /// シリアライズ、デシリアライズで対称性のある構造体
    #[test]
    fn test_serialize_symmetry_numbers() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        // こうすると指定しやすいけどASCII文字以外が使えないのが難点
        #[serde(rename = "TESTDATA00000000")]
        struct Test {
            #[serde(rename = "128")]
            x: bool,
            #[serde(rename = "10")]
            u8: u8,
            #[serde(rename = "11")]
            u16: u16,
            #[serde(rename = "12")]
            u32: u32,
            #[serde(rename = "13")]
            u64: u64,
            #[serde(rename = "15")]
            i8: i8,
            #[serde(rename = "16")]
            i16: i16,
            #[serde(rename = "17")]
            i32: i32,
            #[serde(rename = "18")]
            i64: i64,
            #[serde(rename = "20")]
            f32: f32,
            #[serde(rename = "21")]
            f64: f64,
        }

        let t = Test {
            x: true,
            u8: 8,
            u16: 16,
            u32: 32,
            u64: 64,
            i8: -8,
            i16: -16,
            i32: -32,
            i64: -64,
            f32: 0.1,
            f64: -123.45,
        };
        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<Test>(&s).unwrap();
        assert_eq!(t, x);
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
        println!("{:?}", s);
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
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        }
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
        };
        let s = to_bytes(&t).unwrap();
        let x = KLVMap::try_from_bytes(&s).unwrap();

        assert_eq!(x.universal_key(), "TESTDATA00000000".as_bytes());
        assert!(x.content_len() > 0);

        for v in x.iter() {
            assert!(v.key > 0);
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
        .is_some())
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
        .is_some())
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
        .is_some())
    }

    #[ignore]
    #[test]
    fn test_unit_variant() {
        #[repr(u8)]
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum V {
            A = 1,
            B = 20,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "XYZZ")]
        struct TestVariant {
            #[serde(rename = "10")]
            va: V,
            #[serde(rename = "11")]
            vb: V,
        }
        let t = TestVariant { va: V::A, vb: V::B };
        let mut serializer = KLVSerializer::default();
        t.serialize(&mut serializer).unwrap();
        println!("{:?}", serializer.get_cache().unwrap());
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
