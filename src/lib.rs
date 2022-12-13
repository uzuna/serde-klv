//! # Serde KLV
//!
//! KLV(Key-Length-Value) is a data encoding standard,
//! often used to embed information in video feeds.
//!
//! This is provide KLV serialier and deserializer for struct.
//!
//! [KLV(Wikipedia)]: https://en.wikipedia.org/wiki/KLV

use std::fmt::Debug;

use byteorder::ByteOrder;

mod de;
mod error;
mod ser;

#[cfg(feature = "uasdls")]
pub mod uasdls;

pub use de::{from_bytes, KLVMap, KLVRaw};
pub use ser::to_bytes;

type LengthByteSize = usize;
type ContentByteSize = usize;

/// parse length rule by BER
pub fn parse_length(buf: &[u8]) -> Result<(LengthByteSize, ContentByteSize), String> {
    use byteorder::BigEndian;
    match LengthOctet::from_u8(buf[0]) {
        LengthOctet::Short(x) => Ok((1, x as usize)),
        LengthOctet::Long(x) => match x {
            1 => Ok((2, buf[1] as usize)),
            2 => Ok((3, BigEndian::read_u16(&buf[1..3]) as usize)),
            4 => Ok((4, BigEndian::read_u32(&buf[1..5]) as usize)),
            8 => Ok((4, BigEndian::read_u64(&buf[1..9]) as usize)),
            x => Err(format!(
                "Unsupported length [{}], supported only {{1,2,4,8}}",
                x
            )),
        },
        LengthOctet::Indefinite => Err("length is Indefinete".to_string()),
        LengthOctet::Reserved => Err("Reserved octet".to_string()),
    }
}

/// LengthはBERの仕様に従う
#[derive(Debug, PartialEq, Eq)]
enum LengthOctet {
    /// 7bit(127)以下の長さは1byteで表される
    Short(u8),
    /// 不定長でマーカオクテット\x00までの読み続ける
    Indefinite,
    /// 続くn Byteで数値を表現する。BigEndians
    Long(u8),
    /// 予約済みで到達しないはずの値
    Reserved,
}

impl LengthOctet {
    const FIRST_BIT: u8 = 0b1000_0000;
    const BIT_MASK: u8 = 0b0111_1111;
    fn from_u8(b: u8) -> Self {
        if b & Self::FIRST_BIT != Self::FIRST_BIT {
            Self::Short(b & Self::BIT_MASK)
        } else if b == 255 {
            Self::Reserved
        } else if b == 128 {
            Self::Indefinite
        } else {
            Self::Long(b & Self::BIT_MASK)
        }
    }

    pub fn length_to_buf(buf: &mut dyn std::io::Write, size: usize) -> std::io::Result<usize> {
        use byteorder::BigEndian;
        if size <= 127 {
            buf.write(&[size as u8])
        } else if size <= u8::MAX as usize {
            buf.write(&[0b1000_0001, size as u8])
        } else if size <= u16::MAX as usize {
            let mut r = [0b1000_0010, 0, 0];
            BigEndian::write_u16(&mut r[1..], size as u16);
            buf.write(&r)
        } else {
            let mut r = [0b1000_0100, 0, 0, 0, 0];
            BigEndian::write_u32(&mut r[1..], size as u32);
            buf.write(&r)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::LengthOctet;

    #[test]
    fn test_length_octets() {
        use LengthOctet::*;
        let td = [
            (0, Short(0)),
            (0b0000_0001, Short(1)),
            (0b0111_1111, Short(127)),
            (0b1000_0000, Indefinite),
            (0b1000_0001, Long(1)),
            (0b1000_0010, Long(2)),
            (0b1111_1111, Reserved),
        ];

        for (b, expect) in td {
            let lo = LengthOctet::from_u8(b);
            assert_eq!(lo, expect);
        }
    }
}
