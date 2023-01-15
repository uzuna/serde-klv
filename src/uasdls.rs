//! Example impl for MISB Standard 0601
//! the Unmanned Air System (UAS) Datalink Local Set (LS)
//! reference: MISB ST 0601.8

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::checksum::CheckSumCalc;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "\x06\x0e\x2b\x34\x02\x0b\x01\x01\x0e\x01\x03\x01\x01\x00\x00\x00")]
pub struct UASDatalinkLS<'a> {
    #[serde(rename = "2", with = "timestamp_micro")]
    pub timestamp: SystemTime,
    /// Relative between longitudinal axis and True North measured in the horizontal plane.
    /// Map 0..(2^16-1) to 0..360.
    /// Resolution: ~5.5 milli degrees.
    #[serde(rename = "5")]
    pub platform_heading_angle: u16,
    /// Angle between longitudinal axis and horizontal plane.
    /// Positive angles above horizontal plane.
    /// Map -(2^15-1)..(2^15-1) to +/-20.
    /// Use -(2^15) as "out of range" indicator. -(2^15) = 0x8000.
    /// Resolution: ~610 micro degrees.
    #[serde(rename = "6")]
    pub platform_pitch_angle: i16,
    /// Angle between transverse axis and transvers-longitudinal plane.
    /// Positive angles for lowered right wing.
    /// Map (-2^15-1)..(2^15-1) to +/-50.
    /// Use -(2^15) as "out of range" indicator. -(2^15) = 0x8000.
    /// Res: ~1525 micro deg.
    #[serde(rename = "7")]
    pub platform_roll_angle: i16,
    #[serde(rename = "11", skip_serializing_if = "Option::is_none")]
    pub image_source_sensor: Option<&'a str>,
    #[serde(rename = "12", skip_serializing_if = "Option::is_none")]
    pub image_coordinate_sensor: Option<&'a str>,

    #[serde(rename = "13", skip_serializing_if = "Option::is_none")]
    pub sensor_latitude: Option<i32>,
    #[serde(rename = "14", skip_serializing_if = "Option::is_none")]
    pub sensor_longtude: Option<i32>,

    #[serde(rename = "15", skip_serializing_if = "Option::is_none")]
    pub sensor_true_altitude: Option<u16>,
    #[serde(rename = "16", skip_serializing_if = "Option::is_none")]
    pub sensor_horizontal_fov: Option<u16>,
    #[serde(rename = "17", skip_serializing_if = "Option::is_none")]
    pub sensor_vertical_fov: Option<u16>,

    #[serde(rename = "18", skip_serializing_if = "Option::is_none")]
    pub sensor_relative_azimuth_angle: Option<u32>,
    #[serde(rename = "19", skip_serializing_if = "Option::is_none")]
    pub sensor_relative_elevation_angle: Option<i32>,
    #[serde(rename = "20", skip_serializing_if = "Option::is_none")]
    pub sensor_relative_roll_angle: Option<i32>,

    #[serde(rename = "21", skip_serializing_if = "Option::is_none")]
    pub slant_range: Option<u32>,
    // ST 0601.8の仕様書ではではu16だがテストデータでは4バイトだったのでu32とする
    #[serde(rename = "22", skip_serializing_if = "Option::is_none")]
    pub target_width: Option<u32>,

    #[serde(rename = "23", skip_serializing_if = "Option::is_none")]
    pub frame_center_latitude: Option<i32>,
    #[serde(rename = "24", skip_serializing_if = "Option::is_none")]
    pub frame_center_longitude: Option<i32>,
    #[serde(rename = "25", skip_serializing_if = "Option::is_none")]
    pub frame_center_elevation: Option<u16>,

    #[serde(rename = "40", skip_serializing_if = "Option::is_none")]
    pub target_location_latitude: Option<i32>,
    #[serde(rename = "41", skip_serializing_if = "Option::is_none")]
    pub target_location_longitude: Option<i32>,
    #[serde(rename = "42", skip_serializing_if = "Option::is_none")]
    pub target_location_elecation: Option<u16>,

    #[serde(rename = "56", skip_serializing_if = "Option::is_none")]
    pub plafform_ground_speed: Option<u8>,
    #[serde(rename = "57", skip_serializing_if = "Option::is_none")]
    pub ground_range: Option<u32>,
    #[serde(rename = "65")]
    pub ls_version_number: u8,
}

/// Checksum Calculater for UAS Local Set packet
pub struct CRC;

impl CheckSumCalc for CRC {
    fn checksum(&self, bytes: &[u8]) -> u16 {
        let mut bcc: u16 = 0;
        for (i, v) in bytes.iter().enumerate() {
            let x = (*v as u16) << (8 * ((i + 1) % 2));
            bcc = bcc.wrapping_add(x);
        }
        bcc
    }
}

impl<'a> Default for UASDatalinkLS<'a> {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::UNIX_EPOCH,
            platform_heading_angle: Default::default(),
            platform_pitch_angle: Default::default(),
            platform_roll_angle: Default::default(),
            image_source_sensor: Default::default(),
            image_coordinate_sensor: Default::default(),
            sensor_latitude: Default::default(),
            sensor_longtude: Default::default(),
            sensor_true_altitude: Default::default(),
            sensor_horizontal_fov: Default::default(),
            sensor_vertical_fov: Default::default(),
            sensor_relative_azimuth_angle: Default::default(),
            sensor_relative_elevation_angle: Default::default(),
            sensor_relative_roll_angle: Default::default(),
            slant_range: Default::default(),
            target_width: Default::default(),
            frame_center_latitude: Default::default(),
            frame_center_longitude: Default::default(),
            frame_center_elevation: Default::default(),
            target_location_latitude: Default::default(),
            target_location_longitude: Default::default(),
            target_location_elecation: Default::default(),
            plafform_ground_speed: Default::default(),
            ground_range: Default::default(),
            ls_version_number: Default::default(),
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        checksum::CheckSumCalc,
        de::from_bytes,
        from_bytes_with_checksum,
        ser::to_bytes,
        uasdls::{UASDatalinkLS, CRC},
    };
    use byteorder::{BigEndian, ByteOrder};
    use chrono::{DateTime, Utc};
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_checksum() {
        let testdata = &[0x06_u8, 0x0e, 0x2b, 0x34, 0x02, 0x00, 0x81, 0xbb];
        let c = CRC {};
        let checksum = c.checksum(testdata);
        let expect = BigEndian::read_u16(&[0xb4, 0xfd]);
        assert_eq!(checksum, expect);
    }

    #[test]
    fn test_uas_datalink_ls() {
        #[rustfmt::skip]
        let buf = vec![
            0x06, 0x0e, 0x2b, 0x34, 0x02, 0x0b, 0x01, 0x01, 0x0e, 0x01, 0x03, 0x01, 0x01, 0x00, 0x00,0x00,
            129, 0x91,
            2, 8, 0, 0x4, 0x6c, 0x8e, 0x20, 0x03, 0x83, 0x85,
            65, 1, 1,
            5, 2, 0x3d, 0x3b,
            6, 2, 0x15, 0x80,
            7, 2, 0x01, 0x52,
            11, 3, 0x45, 0x4f, 0x4e,
            12, 14, 0x47, 0x65, 0x6f, 0x64, 0x65, 0x74, 0x69, 0x63, 0x20, 0x57, 0x47, 0x53, 0x38, 0x34,
            13, 4, 0x4d, 0xc4, 0xdc, 0xbb,
            14, 4, 0xb1, 0xa8, 0x6c, 0xfe,
            15, 2, 0x1f, 0x4a,
            16, 2, 0x00, 0x85,
            17, 2, 0x00, 0x4b,
            18, 4, 0x20, 0xc8, 0xd2, 0x7d,
            19, 4, 0xfc, 0xdd, 0x02, 0xd8,
            20, 4, 0xfe, 0xb8, 0xcb, 0x61,
            21, 4, 0x00, 0x8f, 0x3e, 0x61,
            22, 4, 0x00, 0x00, 0x01, 0xc9,
            23, 4, 0x4d, 0xdd, 0x8c, 0x2a,
            24, 4, 0xb1, 0xbe, 0x9e, 0xf4,
            25, 2, 0x0b, 0x85,
            40, 4, 0x4d, 0xdd, 0x8c, 0x2a,
            41, 4, 0xb1, 0xbe, 0x9e, 0xf4,
            42, 2, 0x0b, 0x85,
            56, 1, 0x2e,
            57, 4, 0x00, 0x8d, 0xd4, 0x29,
            1, 2, 0x1c, 0x5f
            ];

        let x: UASDatalinkLS = from_bytes_with_checksum(&buf, CRC {}).unwrap();
        let datetime: DateTime<Utc> = x.timestamp.into();
        assert_eq!(
            DateTime::parse_from_rfc3339("2009-06-17T16:53:05.099653+00:00").unwrap(),
            datetime
        );
        assert_eq!(x.ls_version_number, 1);
        assert_eq!(x.platform_heading_angle, 15675);
        assert_eq!(x.sensor_latitude, Some(1304747195));
        assert_eq!(x.image_source_sensor, Some("EON"));
        assert_eq!(x.image_coordinate_sensor, Some("Geodetic WGS84"));
    }

    #[test]
    fn test_serialize() {
        let ts = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_micros(1_000_233_000))
            .unwrap();
        let t = UASDatalinkLS {
            timestamp: ts,
            platform_heading_angle: 123,
            platform_pitch_angle: -345,
            platform_roll_angle: 456,
            ..Default::default()
        };

        let s = to_bytes(&t).unwrap();
        let x = from_bytes::<UASDatalinkLS>(&s).unwrap();
        assert_eq!(t, x);
    }
    #[test]
    fn test_deserialize_error() {
        let buf = vec![
            0x06, 0x0e, 0x2b, 0x34, 0x02, 0x0b, 0x01, 0x01, 0x0e, 0x01, 0x03, 0x01, 0x01, 0x00,
            0x00, 0x01, 0x01,
        ];
        let err = from_bytes::<UASDatalinkLS>(&buf).unwrap_err();
        match err {
            crate::error::Error::Key(_) => {}
            _ => unreachable!(),
        }
        let buf = vec![
            0x06, 0x0e, 0x2b, 0x34, 0x02, 0x0b, 0x01, 0x01, 0x0e, 0x01, 0x03, 0x01, 0x01, 0x00,
            0x00, 0x00,
        ];
        let err = from_bytes::<UASDatalinkLS>(&buf).unwrap_err();
        match err {
            crate::error::Error::ContentLenght => {}
            _ => unreachable!(),
        }
    }
}
