pub(crate) const CHECKSUM_KEY_LENGTH: &[u8; 2] = &[0x01, 0x02];

pub trait CheckSumCalc {
    fn checksum(&self, bytes: &[u8]) -> u16;
}

struct WrappedCRC {
    crc: crc::Crc<u16>,
}
impl Default for WrappedCRC {
    fn default() -> Self {
        Self {
            crc: crc::Crc::<u16>::new(&crc::CRC_16_ISO_IEC_14443_3_A),
        }
    }
}

impl CheckSumCalc for WrappedCRC {
    fn checksum(&self, bytes: &[u8]) -> u16 {
        self.crc.checksum(bytes)
    }
}

#[cfg(test)]
mod tests {
    use byteorder::BigEndian;
    use serde::{Deserialize, Serialize};

    use crate::{
        checksum::WrappedCRC, from_bytes, from_bytes_with_checksum, ser::to_bytes_with_crc,
    };

    use super::CheckSumCalc;

    #[test]
    fn test_checksum() {
        use byteorder::WriteBytesExt;
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestString {
            #[serde(rename = "30")]
            string: String,

            #[serde(rename = "40")]
            u64: u64,
        }
        let t = TestString {
            string: "123".to_string(),
            u64: 123,
        };

        let buf = to_bytes_with_crc(&t, WrappedCRC::default()).unwrap();
        println!("{:?}", buf);

        let crc = WrappedCRC::default();
        let crc_code = crc.checksum(&buf[0..buf.len() - 2]);
        let mut crc_buf = [0_u8; 2];
        crc_buf
            .as_mut_slice()
            .write_u16::<BigEndian>(crc_code)
            .unwrap();
        println!("{:?}", crc_buf);

        let x: TestString = from_bytes(&buf).unwrap();
        assert_eq!(&t, &x);
        let x: TestString = from_bytes_with_checksum(&buf, WrappedCRC::default()).unwrap();
        assert_eq!(&t, &x);
    }
}
