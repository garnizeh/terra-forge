//! Minimal, dependency-free ULID value type.
//!
//! The Engine never *generates* identifiers — ULID generation embeds a
//! millisecond timestamp, which would require wall-clock access the Engine
//! is not allowed to have. It only parses, compares, and re-serializes
//! ULIDs minted elsewhere (the Platform Server, or test/preset fixtures).
//!
//! This type exists instead of depending on the `ulid` crate because that
//! crate unconditionally pulls in `web-time` (a wall-clock polyfill) as a
//! dependency on `wasm32-unknown-unknown`, regardless of feature flags —
//! which would violate the Engine's zero-wall-clock-time constraint via its
//! dependency tree even though no generation function is ever called.

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Crockford's Base32 alphabet, as used by the ULID spec. Excludes I, L, O,
/// U to avoid visual confusion with 1, 1, 0, V.
const ENCODING: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A ULID's canonical string form is always 26 characters.
const ENCODED_LEN: usize = 26;

/// A 128-bit ULID value, stored and compared as an opaque identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ulid(u128);

impl Ulid {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(&self) -> u128 {
        self.0
    }
}

/// A string failed to parse as a valid 26-character Crockford Base32 ULID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseUlidError;

impl fmt::Display for ParseUlidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid ULID string")
    }
}

impl std::error::Error for ParseUlidError {}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; ENCODED_LEN];
        let mut value = self.0;
        for slot in buf.iter_mut().rev() {
            *slot = ENCODING[(value & 0x1F) as usize];
            value >>= 5;
        }
        f.write_str(std::str::from_utf8(&buf).expect("ULID encoding is always ASCII"))
    }
}

impl FromStr for Ulid {
    type Err = ParseUlidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != ENCODED_LEN {
            return Err(ParseUlidError);
        }

        let mut value: u128 = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            let digit = decode_char(byte).ok_or(ParseUlidError)?;
            // 26 chars * 5 bits = 130 bits, 2 more than fit in 128 — only
            // the first character may carry those extra (always-zero) bits,
            // which caps its value at 7.
            if i == 0 && digit > 7 {
                return Err(ParseUlidError);
            }
            value = (value << 5) | u128::from(digit);
        }

        Ok(Self(value))
    }
}

fn decode_char(byte: u8) -> Option<u8> {
    let upper = byte.to_ascii_uppercase();
    ENCODING
        .iter()
        .position(|&candidate| candidate == upper)
        .map(|pos| pos as u8)
}

impl Serialize for Ulid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Ulid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_string() {
        let ulid = Ulid::from_u128(0x0123_4567_89AB_CDEF_0123_4567_89AB_CDEF);
        let encoded = ulid.to_string();
        assert_eq!(encoded.len(), ENCODED_LEN);
        let decoded: Ulid = encoded.parse().expect("valid ULID string");
        assert_eq!(ulid, decoded);
    }

    #[test]
    fn round_trips_through_serde() {
        let ulid = Ulid::from_u128(42);
        let json = serde_json::to_string(&ulid).unwrap();
        let decoded: Ulid = serde_json::from_str(&json).unwrap();
        assert_eq!(ulid, decoded);
    }

    #[test]
    fn rejects_wrong_length() {
        assert!("TOO_SHORT".parse::<Ulid>().is_err());
    }

    #[test]
    fn rejects_invalid_characters() {
        // 'I' is excluded from Crockford Base32.
        let s = "I0000000000000000000000000".get(..ENCODED_LEN).unwrap();
        assert!(s.parse::<Ulid>().is_err());
    }

    #[test]
    fn rejects_overflowing_first_character() {
        // 'Z' decodes to 31; only 0-7 are valid for the first character.
        let s = "Z0000000000000000000000000".get(..ENCODED_LEN).unwrap();
        assert!(s.parse::<Ulid>().is_err());
    }

    #[test]
    fn known_vector_round_trips() {
        // Canonical example from the ULID spec (github.com/ulid/spec).
        let text = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let ulid: Ulid = text.parse().expect("valid ULID string");
        assert_eq!(ulid.to_string(), text);
    }
}
