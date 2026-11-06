//! The parts of the proto3 JSON mapping serde does not do by itself, for the generated
//! contracts (`crate::contracts`, `crate::dispatch::contracts`): an `int64` travels as a decimal
//! string (a number is read too), a
//! `bytes` field as standard base64, and a `double` may arrive as `"NaN"` or `"Infinity"`.

/// `int64`: written as a string, read from a string or a number.
pub mod int64 {
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        struct Int64;
        impl Visitor<'_> for Int64 {
            type Value = i64;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an int64 as a string or a number")
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<i64, E> {
                Ok(value)
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<i64, E> {
                i64::try_from(value).map_err(E::custom)
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<i64, E> {
                if value.fract() == 0.0 && value.abs() < 9.007_199_254_740_992e15 {
                    Ok(value as i64)
                } else {
                    Err(E::custom(format!("{value} is not an int64")))
                }
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<i64, E> {
                value.trim().parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(Int64)
    }
}

/// `optional int64`: as [`int64`] when present; absent (or `null`) reads as `None`, and `None`
/// is left out (the field also takes `skip_serializing_if`).
pub mod optional_int64 {
    use serde::{Deserialize, Deserializer, Serializer};

    #[derive(Deserialize)]
    struct Wrapped(#[serde(with = "super::int64")] i64);

    pub fn serialize<S: Serializer>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.collect_str(value),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<i64>, D::Error> {
        Ok(Option::<Wrapped>::deserialize(deserializer)?.map(|Wrapped(value)| value))
    }
}

/// `double`: a number, or one of the strings protojson uses for the values JSON lacks.
pub fn double<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    use serde::de::{self, Visitor};
    struct Double;
    impl Visitor<'_> for Double {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a double")
        }
        fn visit_f64<E: de::Error>(self, value: f64) -> Result<f64, E> {
            Ok(value)
        }
        fn visit_i64<E: de::Error>(self, value: i64) -> Result<f64, E> {
            Ok(value as f64)
        }
        fn visit_u64<E: de::Error>(self, value: u64) -> Result<f64, E> {
            Ok(value as f64)
        }
        fn visit_str<E: de::Error>(self, value: &str) -> Result<f64, E> {
            match value {
                "NaN" => Ok(f64::NAN),
                "Infinity" => Ok(f64::INFINITY),
                "-Infinity" => Ok(f64::NEG_INFINITY),
                other => other.parse().map_err(E::custom),
            }
        }
    }
    deserializer.deserialize_any(Double)
}

/// `bytes`: standard base64 with padding; reading also takes the URL-safe alphabet and no
/// padding, as protojson does.
pub mod bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        for chunk in data.chunks(3) {
            let b = [
                chunk[0],
                *chunk.get(1).unwrap_or(&0),
                *chunk.get(2).unwrap_or(&0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
                if i <= chunk.len() {
                    out.push(ALPHABET[((n >> shift) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    pub fn decode(text: &str) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(text.len() / 4 * 3);
        let (mut acc, mut bits) = (0u32, 0u32);
        for byte in text.bytes() {
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' | b'-' => 62,
                b'/' | b'_' => 63,
                b'=' => break,
                b'\r' | b'\n' => continue,
                other => return Err(format!("invalid base64 byte {other:#04x}")),
            };
            acc = (acc << 6) | u32::from(value);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
                acc &= (1 << bits) - 1;
            }
        }
        Ok(out)
    }

    pub fn serialize<S: Serializer>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&encode(value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(deserializer)?;
        decode(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Sample {
        #[serde(with = "super::int64")]
        seq: i64,
        #[serde(with = "super::bytes")]
        data: Vec<u8>,
        #[serde(deserialize_with = "super::double")]
        score: f64,
    }

    #[test]
    fn int64_is_a_string_and_reads_numbers_too() {
        let sample = Sample {
            seq: 9_007_199_254_740_993,
            ..Sample::default()
        };
        let text = serde_json::to_string(&sample).unwrap();
        assert!(text.contains(r#""seq":"9007199254740993""#), "{text}");
        let read: Sample = serde_json::from_str(r#"{"seq":42}"#).unwrap();
        assert_eq!(read.seq, 42);
        let read: Sample = serde_json::from_str(r#"{"seq":"-7"}"#).unwrap();
        assert_eq!(read.seq, -7);
    }

    #[test]
    fn bytes_round_trip_through_base64() {
        for data in [
            &b""[..],
            b"f",
            b"fo",
            b"foo",
            b"foob",
            b"\x00\xff\x10 polter",
        ] {
            let encoded = super::bytes::encode(data);
            assert_eq!(super::bytes::decode(&encoded).unwrap(), data, "{encoded}");
        }
        assert_eq!(super::bytes::encode(b"foob"), "Zm9vYg==");
        assert_eq!(super::bytes::decode("Zm9vYg").unwrap(), b"foob");
        assert_eq!(super::bytes::decode("-_8").unwrap(), [0xfb, 0xff]);
    }

    #[test]
    fn doubles_take_the_protojson_strings() {
        let read: Sample = serde_json::from_str(r#"{"score":"Infinity"}"#).unwrap();
        assert!(read.score.is_infinite());
        let read: Sample = serde_json::from_str(r#"{"score":1.5}"#).unwrap();
        assert_eq!(read.score, 1.5);
    }
}
