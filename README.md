# Serde KLV &emsp; [![Build Status]][github action] [![Latest Version]][crates.io] [![Rustc Version 1.65+]][rustc]

[Build Status]: https://github.com/uzuna/serde-klv/actions/workflows/ci.yml/badge.svg
[github action]: https://github.com/uzuna/serde-klv/actions/workflows/ci.yml
[Latest Version]: https://img.shields.io/crates/v/serde_klv.svg
[crates.io]: https://crates.io/crates/serde\_klv
[Rustc Version 1.65+]: https://img.shields.io/badge/rustc-1.65+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html

**Serde is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

You may be looking for:
- [Serde API documentation](https://docs.serde.rs/serde/)
- [Detailed documentation about Serde](https://serde.rs/)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/derive.html)

## Usage

```rust
use serde_klv::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
// Set Universal Key string or byte-literal
#[serde(rename = "TESTDATA00000000")]
// #[serde(rename = "\x06\x0e\x2b\x34\x02\x0b\x01\x01\x0e\x01\x03\x01\x01\x00\x00\x00")]
struct TestStruct<'a> {
    // rename to u8 range number
    #[serde(rename = "10")]
    u8: u8,
    #[serde(rename = "11")]
    u64: u64,
    // can use Option
    #[serde(rename = "120", skip_serializing_if = "Option::is_none")]
    none_skip_some: Option<u16>,
    #[serde(rename = "121", skip_serializing_if = "Option::is_none")]
    none_skip_none: Option<u16>,
    #[serde(rename = "60")]
    str: &'a str,
    // Use `serde_bytes` when using Vec<u8} or &[u8]: https://crates.io/crates/serde_bytes
    #[serde(rename = "61", with = "serde_bytes")]
    bytes: &'a [u8],
    // Implement a serializer when using structs
    #[serde(rename = "62", with = "timestamp_micro")]
    ts: SystemTime,
}

mod timestamp_micro {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime};

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

fn main() {
    let ts = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_micros(1_000_233_000))
            .unwrap();
    let t = TestStruct {
        u8: 127,
        u64: u32::MAX as u64 + 1,
        none_skip_some: Some(2016),
        none_skip_none: None,
        str: "this is string",
        bytes: b"this is byte",
        ts,
    };
    let buf = to_bytes(&t).unwrap();
    let x = from_bytes::<TestStruct>(&buf).unwrap();
    assert_eq!(&t, &x);

    // with checksum
    use serde_klv::{from_bytes_with_checksum, to_bytes_with_checksum, WrappedCRC};
    
    let buf = to_bytes_with_checksum(&t, WrappedCRC::default()).unwrap();
    let x: TestStruct = from_bytes_with_checksum(&buf, WrappedCRC::default()).unwrap();
    assert_eq!(&t, &x);
}
```
