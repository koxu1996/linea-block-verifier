//! Serde utilities for beacon API responses.
//!
//! These provide symmetric serialization/deserialization for types that
//! come from the beacon API as strings but need to round-trip correctly.

use alloy_primitives::{Address, B256, Bloom, Bytes};
use serde::{Deserialize, Deserializer, Serializer};

// =============================================================================
// u64 as string
// =============================================================================

pub fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  s.parse().map_err(serde::de::Error::custom)
}

// =============================================================================
// u32 as string
// =============================================================================

pub fn serialize_u32_as_string<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  s.parse().map_err(serde::de::Error::custom)
}

// =============================================================================
// Address as hex string
// =============================================================================

pub fn serialize_address_as_hex<S>(value: &Address, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&format!("0x{}", hex::encode(value.as_slice())))
}

pub fn deserialize_address_from_hex<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  let bytes = hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)?;
  if bytes.len() != 20 {
    return Err(serde::de::Error::custom("expected 20 bytes for Address"));
  }
  Ok(Address::from_slice(&bytes))
}

// =============================================================================
// Bloom as hex string
// =============================================================================

pub fn serialize_bloom_as_hex<S>(value: &Bloom, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&format!("0x{}", hex::encode(value.as_slice())))
}

pub fn deserialize_bloom_from_hex<'de, D>(deserializer: D) -> Result<Bloom, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  let bytes = hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)?;
  if bytes.len() != 256 {
    return Err(serde::de::Error::custom("expected 256 bytes for Bloom"));
  }
  Ok(Bloom::from_slice(&bytes))
}

// =============================================================================
// Bytes as hex string
// =============================================================================

pub fn serialize_bytes_as_hex<S>(value: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&format!("0x{}", hex::encode(value.as_ref())))
}

pub fn deserialize_bytes_from_hex<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  let bytes = hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)?;
  Ok(Bytes::from(bytes))
}

// =============================================================================
// Vec<Bytes> as hex strings
// =============================================================================

pub fn serialize_vec_bytes_as_hex<S>(value: &Vec<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  use serde::ser::SerializeSeq;
  let mut seq = serializer.serialize_seq(Some(value.len()))?;
  for bytes in value {
    seq.serialize_element(&format!("0x{}", hex::encode(bytes.as_ref())))?;
  }
  seq.end()
}

pub fn deserialize_vec_bytes_from_hex<'de, D>(deserializer: D) -> Result<Vec<Bytes>, D::Error>
where
  D: Deserializer<'de>,
{
  let strings: Vec<String> = Deserialize::deserialize(deserializer)?;
  strings
    .into_iter()
    .map(|s| {
      let bytes = hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)?;
      Ok(Bytes::from(bytes))
    })
    .collect()
}

// =============================================================================
// B256 as hex string
// =============================================================================

pub fn serialize_b256_as_hex<S>(value: &B256, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&format!("0x{}", hex::encode(value.as_slice())))
}

pub fn deserialize_b256_from_hex<'de, D>(deserializer: D) -> Result<B256, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  let bytes = hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)?;
  if bytes.len() != 32 {
    return Err(serde::de::Error::custom("expected 32 bytes for B256"));
  }
  Ok(B256::from_slice(&bytes))
}
