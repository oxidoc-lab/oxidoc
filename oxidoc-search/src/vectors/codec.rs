//! Encoding and decoding of `search-vectors.bin`.
//!
//! Every number is little-endian. Fields follow each other with no padding.
//!
//! | Offset | Size | Field |
//! | :----- | :--- | :---- |
//! | 0 | 5 | magic `b"OXVEC"` |
//! | 5 | 1 | `version: u8`, currently 1 |
//! | 6 | 4 | `dimension: u32`, at least 1 |
//! | 10 | 4 | `count: u32` |
//! | 14 | `count * (8 + dimension)` | `count` vector records |
//!
//! Each vector record holds, in order:
//!
//! - `doc_id: u32`: strictly increasing across records.
//! - `scale: f32`: finite and not negative.
//! - `dimension` × `i8`: values in `[-127, 127]`. All zero when `scale` is 0.
//!
//! A value dequantises to `value * scale`. The file length is exactly
//! `14 + count * (8 + dimension)` bytes. The file holds no document text: title,
//! path and snippet come from `search-meta.bin` by doc id.

use crate::error::{SearchError, SearchResult};

use super::quantize::Q_MAX;
use super::store::QuantVectors;

/// File name of the int8 vector index.
pub const VECTORS_FILE: &str = "search-vectors.bin";
/// Format version this reader and writer support.
pub const VECTORS_VERSION: u8 = 1;

/// First bytes of every vector file.
const MAGIC: &[u8; 5] = b"OXVEC";
/// Bytes before the first vector record.
const HEADER_LEN: usize = 14;
/// Bytes of `doc_id` and `scale` in each vector record.
const RECORD_HEAD_LEN: usize = 8;

/// Returns an [`SearchError::IndexLoad`] that names the vector file.
fn load_error(message: impl std::fmt::Display) -> SearchError {
    SearchError::IndexLoad(format!("{VECTORS_FILE}: {message}; rebuild the site"))
}

/// Encodes `vectors` in the `search-vectors.bin` format.
///
/// Fails when the dimension is 0, or when the dimension or count exceeds `u32::MAX`.
pub fn encode_vectors(vectors: &QuantVectors) -> SearchResult<Vec<u8>> {
    let encode_error =
        |what: &str| SearchError::Serialization(format!("Failed to encode {VECTORS_FILE}: {what}"));
    let dimension = vectors.dimension();
    if dimension == 0 {
        return Err(encode_error("dimension is 0"));
    }
    let dim32 = u32::try_from(dimension).map_err(|_| encode_error("dimension exceeds u32"))?;
    let count = u32::try_from(vectors.len()).map_err(|_| encode_error("count exceeds u32"))?;

    let mut out = Vec::with_capacity(HEADER_LEN + vectors.len() * (RECORD_HEAD_LEN + dimension));
    out.extend_from_slice(MAGIC);
    out.push(VECTORS_VERSION);
    out.extend_from_slice(&dim32.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    for (index, (&doc_id, &scale)) in vectors.doc_ids().iter().zip(vectors.scales()).enumerate() {
        let values = vectors
            .values(index)
            .ok_or_else(|| encode_error("vector values are missing"))?;
        out.extend_from_slice(&doc_id.to_le_bytes());
        out.extend_from_slice(&scale.to_le_bytes());
        out.extend(values.iter().map(|v| v.to_le_bytes()[0]));
    }
    Ok(out)
}

/// Splits the first `N` bytes off `bytes`.
fn take<'a, const N: usize>(bytes: &mut &'a [u8]) -> Option<&'a [u8; N]> {
    let (head, rest) = bytes.split_first_chunk::<N>()?;
    *bytes = rest;
    Some(head)
}

/// Decodes and checks `search-vectors.bin` bytes.
///
/// Fails on a bad magic, an unknown version, a zero dimension, a length that is
/// not exactly the header plus `count` records, a NaN, infinite or negative
/// scale, a value of -128, non-zero values under scale 0, or doc ids out of order.
/// Allocates only after the length check, so a forged count cannot force a
/// large allocation.
pub fn decode_vectors(data: &[u8]) -> SearchResult<QuantVectors> {
    let mut rest = data;
    let header_error = || {
        load_error(format!(
            "file is {} bytes, shorter than the {HEADER_LEN}-byte header",
            data.len()
        ))
    };
    let magic = take::<5>(&mut rest).ok_or_else(header_error)?;
    if magic != MAGIC {
        return Err(load_error("bad magic, not an oxidoc vector file"));
    }
    let [version] = *take::<1>(&mut rest).ok_or_else(header_error)?;
    if version != VECTORS_VERSION {
        return Err(SearchError::UnsupportedVersion {
            file: VECTORS_FILE.to_string(),
            found: u32::from(version),
            supported: u32::from(VECTORS_VERSION),
        });
    }
    let dimension = u32::from_le_bytes(*take::<4>(&mut rest).ok_or_else(header_error)?);
    let count = u32::from_le_bytes(*take::<4>(&mut rest).ok_or_else(header_error)?);
    if dimension == 0 {
        return Err(load_error("dimension is 0"));
    }

    let record_len = RECORD_HEAD_LEN as u64 + u64::from(dimension);
    let expected = u64::from(count)
        .checked_mul(record_len)
        .and_then(|body| body.checked_add(HEADER_LEN as u64));
    let actual = data.len() as u64;
    let need = |n: &dyn std::fmt::Display| {
        format!("{count} vectors of dimension {dimension} need {n} bytes, file has {actual}")
    };
    match expected {
        Some(n) if n == actual => {}
        Some(n) if actual > n => return Err(load_error(format!("trailing bytes: {}", need(&n)))),
        Some(n) => return Err(load_error(format!("truncated: {}", need(&n)))),
        None => {
            return Err(load_error(format!(
                "truncated: {}",
                need(&"more than u64::MAX")
            )));
        }
    }

    // The length check bounds `count * dimension` by `data.len()`, so both fit in usize.
    let dimension = dimension as usize;
    let mut vectors = QuantVectors::with_capacity(dimension, count as usize);
    let mut values = Vec::with_capacity(dimension);
    for index in 0..count {
        let truncated = || load_error(format!("vector {index} is truncated"));
        let doc_id = u32::from_le_bytes(*take::<4>(&mut rest).ok_or_else(truncated)?);
        let scale = f32::from_le_bytes(*take::<4>(&mut rest).ok_or_else(truncated)?);
        let (row, tail) = rest.split_at_checked(dimension).ok_or_else(truncated)?;
        rest = tail;

        if !scale.is_finite() || scale < 0.0 {
            return Err(load_error(format!(
                "vector {index} (doc {doc_id}) has scale {scale}; a scale must be finite and not negative"
            )));
        }
        values.clear();
        values.extend(row.iter().map(|&b| i8::from_le_bytes([b])));
        if values.contains(&i8::MIN) {
            return Err(load_error(format!(
                "vector {index} (doc {doc_id}) holds -128; values must be in [-{Q_MAX}, {Q_MAX}]"
            )));
        }
        if scale == 0.0 && values.iter().any(|&v| v != 0) {
            return Err(load_error(format!(
                "vector {index} (doc {doc_id}) has scale 0 but non-zero values"
            )));
        }
        vectors
            .check_order(doc_id)
            .map_err(|e| load_error(format!("vector {index}: {e}")))?;
        vectors.push_raw(doc_id, scale, &values);
    }
    Ok(vectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> QuantVectors {
        let mut vectors = QuantVectors::new(3);
        vectors.push(0, &[0.5, -1.0, 0.25]).unwrap();
        vectors.push(2, &[0.0, 0.0, 0.0]).unwrap();
        vectors.push(7, &[3.0, 1.5, -0.75]).unwrap();
        vectors
    }

    fn assert_load_error(data: &[u8], needle: &str) {
        match decode_vectors(data) {
            Err(SearchError::IndexLoad(message)) => {
                assert!(message.contains(needle), "{message:?} lacks {needle:?}");
            }
            other => panic!("expected an index load error with {needle:?}, got {other:?}"),
        }
    }

    #[test]
    fn round_trips() {
        let vectors = sample();
        let bytes = encode_vectors(&vectors).unwrap();
        assert_eq!(bytes.len(), HEADER_LEN + 3 * (RECORD_HEAD_LEN + 3));
        assert_eq!(decode_vectors(&bytes).unwrap(), vectors);
    }

    #[test]
    fn layout_is_little_endian_without_padding() {
        let mut vectors = QuantVectors::new(2);
        vectors.push(0x0102_0304, &[1.0, -0.5]).unwrap();
        let bytes = encode_vectors(&vectors).unwrap();
        let scale = (1.0f32 / 127.0).to_le_bytes();
        let mut expected = b"OXVEC".to_vec();
        expected.extend_from_slice(&[1, 2, 0, 0, 0, 1, 0, 0, 0, 4, 3, 2, 1]);
        expected.extend_from_slice(&scale);
        expected.extend_from_slice(&[127, (-64i8).to_le_bytes()[0]]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn empty_store_round_trips() {
        let bytes = encode_vectors(&QuantVectors::new(4)).unwrap();
        assert_eq!(bytes.len(), HEADER_LEN);
        let back = decode_vectors(&bytes).unwrap();
        assert!(back.is_empty());
        assert_eq!(back.dimension(), 4);
    }

    #[test]
    fn zero_dimension_is_refused_both_ways() {
        assert!(encode_vectors(&QuantVectors::new(0)).is_err());
        let mut bytes = encode_vectors(&QuantVectors::new(1)).unwrap();
        bytes[6] = 0;
        assert_load_error(&bytes, "dimension is 0");
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[0] = b'X';
        assert_load_error(&bytes, "bad magic");
        assert_load_error(b"{\"documents\":[]}", "bad magic");
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[5] = 2;
        assert!(matches!(
            decode_vectors(&bytes),
            Err(SearchError::UnsupportedVersion {
                found: 2,
                supported: 1,
                ..
            })
        ));
    }

    #[test]
    fn short_header_is_rejected() {
        let bytes = encode_vectors(&sample()).unwrap();
        for len in 0..HEADER_LEN {
            assert!(decode_vectors(&bytes[..len]).is_err(), "length {len}");
        }
        assert_load_error(&bytes[..5], "header");
    }

    #[test]
    fn truncated_and_trailing_data_are_rejected() {
        let bytes = encode_vectors(&sample()).unwrap();
        for len in HEADER_LEN..bytes.len() {
            assert_load_error(&bytes[..len], "truncated");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert_load_error(&longer, "trailing bytes");
    }

    #[test]
    fn forged_count_is_rejected_without_allocating() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[10..14].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_load_error(&bytes, "truncated");
        bytes[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_load_error(&bytes, "truncated");
    }

    #[test]
    fn count_dimension_mismatch_is_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[6] = 4;
        assert!(decode_vectors(&bytes).is_err());
        bytes[6] = 2;
        assert!(decode_vectors(&bytes).is_err());
    }

    /// Offset of the scale of vector `index` in an encoding of [`sample`].
    fn scale_offset(index: usize) -> usize {
        HEADER_LEN + index * (RECORD_HEAD_LEN + 3) + 4
    }

    #[test]
    fn non_finite_or_negative_scale_is_rejected() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
            let mut bytes = encode_vectors(&sample()).unwrap();
            let at = scale_offset(2);
            bytes[at..at + 4].copy_from_slice(&bad.to_le_bytes());
            assert_load_error(&bytes, "scale");
        }
    }

    #[test]
    fn out_of_range_value_is_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[scale_offset(0) + 4] = 0x80;
        assert_load_error(&bytes, "-128");
    }

    #[test]
    fn non_zero_values_under_zero_scale_are_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        bytes[scale_offset(1) + 4] = 1;
        assert_load_error(&bytes, "scale 0");
    }

    #[test]
    fn doc_ids_out_of_order_are_rejected() {
        let mut bytes = encode_vectors(&sample()).unwrap();
        let at = scale_offset(2) - 4;
        bytes[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
        assert_load_error(&bytes, "doc id 2");
    }
}
