//! Hex helpers for `Uint8Array` proposal methods.

use data_encoding::{DecodeError, DecodePartial, HEXLOWER, HEXLOWER_PERMISSIVE};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DecodeMutResult {
    pub(crate) read: usize,
    pub(crate) written: usize,
    pub(crate) error: Option<DecodeError>,
}

pub(crate) fn decode_mut(
    input: &[u8],
    output: &mut [u8],
    max_length: Option<usize>,
) -> DecodeMutResult {
    let max_length = max_length.unwrap_or(usize::MAX);

    if let Err(error) = HEXLOWER_PERMISSIVE.decode_len(input.len()) {
        return DecodeMutResult {
            read: 0,
            written: 0,
            error: Some(error),
        };
    }

    let read = core::cmp::min(input.len(), max_length.saturating_mul(2));
    match HEXLOWER_PERMISSIVE.decode_mut(&input[..read], &mut output[..read / 2]) {
        Ok(written) => DecodeMutResult {
            read,
            written,
            error: None,
        },
        Err(DecodePartial {
            read,
            written,
            error,
        }) => DecodeMutResult {
            read,
            written,
            error: Some(error),
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DecodeResult {
    pub(crate) read: usize,
    pub(crate) output: Vec<u8>,
    pub(crate) error: Option<DecodeError>,
}

pub(crate) fn decode(input: &[u8], max_length: Option<usize>) -> DecodeResult {
    let output_len = core::cmp::min(input.len() / 2, max_length.unwrap_or(usize::MAX));
    let mut output = vec![0; output_len];
    let DecodeMutResult {
        read,
        written,
        error,
    } = decode_mut(input, &mut output, max_length);
    debug_assert!(written <= output.len());
    output.truncate(written);
    DecodeResult {
        read,
        output,
        error,
    }
}

pub(crate) fn encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(HEXLOWER.encode_len(input.len()));
    HEXLOWER.encode_append(input, &mut output);
    output
}
