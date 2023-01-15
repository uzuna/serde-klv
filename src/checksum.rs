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
    use std::ops::Deref;

    use byteorder::BigEndian;
    use cosmic_ray::Ray;
    use rand::Rng;
    use serde::{Deserialize, Serialize};

    use crate::{
        checksum::WrappedCRC, de::checksum, from_bytes, from_bytes_with_checksum,
        ser::to_bytes_with_checksum, to_bytes,
    };

    use super::CheckSumCalc;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename = "TESTDATA00000000")]
    struct TestString {
        #[serde(rename = "30")]
        string: String,

        #[serde(rename = "40")]
        u64: u64,
    }

    // bit反転を検知できるか
    #[test]
    fn test_checksum_error() {
        let t = TestString {
            string: "ja895puhjekptgsh5uiltja4:rg98ue".to_string(),
            u64: 123,
        };

        let buf = to_bytes_with_checksum(&t, WrappedCRC::default()).unwrap();
        let buf_len = buf.len();
        let mut raybox = cosmic_ray::RayBoxVec::new(buf);

        assert!(checksum(raybox.deref(), WrappedCRC::default()).is_ok());
        let mut rng = rand::thread_rng();

        for i in 0..buf_len {
            raybox.attack(Ray::new(i)).unwrap();
            // detect 1bit error
            let err = checksum(raybox.deref(), WrappedCRC::default());
            assert!(err.is_err());

            // check detect error 2bit or more
            raybox
                .attack(Ray::with_pattern(rng.gen_range(0..buf_len - 1), Ray::P2BIT))
                .unwrap();
            assert!(err.is_err());

            // restore
            raybox.restore_all();
            assert!(checksum(raybox.deref(), WrappedCRC::default()).is_ok());
        }
    }

    // bit反転を検知できるか
    #[test]
    fn test_checksum_reserved() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestChecksum {
            #[serde(rename = "1")]
            checksum: u16,

            #[serde(rename = "40")]
            u64: u64,
        }
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename = "TESTDATA00000000")]
        struct TestParent {
            #[serde(rename = "10")]
            checksum: TestChecksum,
        }
        let t = TestChecksum {
            checksum: 0,
            u64: 123,
        };

        // checksumなしなら1のキーがあっても問題なし
        {
            let buf = to_bytes(&t).unwrap();
            let x: TestChecksum = from_bytes(&buf).unwrap();
            assert_eq!(&t, &x);
        }

        // checksumが有効な場合、1階層めに1のkeyを持っているとシリアライズに失敗する
        {
            let err = to_bytes_with_checksum(&t, WrappedCRC::default());
            assert!(err.is_err());
        }

        // 2階層目以降には無関係
        let t = TestParent { checksum: t };
        let buf = to_bytes_with_checksum(&t, WrappedCRC::default()).unwrap();
        let x: TestParent = from_bytes(&buf).unwrap();
        assert_eq!(&t, &x);
    }

    // checksum付きのシリアライズ、デシリアライズ
    #[test]
    fn test_checksum() {
        use byteorder::WriteBytesExt;
        let t = TestString {
            string: "123".to_string(),
            u64: 123,
        };

        let buf = to_bytes_with_checksum(&t, WrappedCRC::default()).unwrap();

        let crc = WrappedCRC::default();
        let crc_code = crc.checksum(&buf[0..buf.len() - 2]);
        let mut crc_buf = [0_u8; 2];
        crc_buf
            .as_mut_slice()
            .write_u16::<BigEndian>(crc_code)
            .unwrap();

        // deserialize
        let x: TestString = from_bytes(&buf).unwrap();
        assert_eq!(&t, &x);
        let x: TestString = from_bytes_with_checksum(&buf, WrappedCRC::default()).unwrap();
        assert_eq!(&t, &x);
    }
}
