//! Base64 helpers for `Uint8Array` proposal methods.
//!
//! This is adapted from the `ecma262` helper in the `data-encoding` repository so that
//! `Uint8Array.{fromBase64,setFromBase64}` follow the proposal's partial-decoding rules.

use data_encoding::{Character, DecodeError, DecodeKind, DecodePartial, Encoding};
use data_encoding_macro::new_encoding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alphabet {
    Base64,
    Base64Url,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LastChunkHandling {
    Loose,
    Strict,
    StopBeforePartial,
}
use LastChunkHandling::{Loose, StopBeforePartial, Strict};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DecodeMutResult {
    pub(crate) read: usize,
    pub(crate) written: usize,
    pub(crate) error: Option<DecodeError>,
}

/// Decodes `input` in `output` according to the given parameters.
///
/// # Panics
///
/// Panics if `output.len() < 6 * input.len() / 8`. It is not an error if `max_length` is smaller
/// than `output.len()`. This function will however not optimize those cases.
pub(crate) fn decode_mut(
    input: &[u8],
    output: &mut [u8],
    alphabet: Alphabet,
    last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
) -> DecodeMutResult {
    // Select the appropriate encoding.
    let base = match alphabet {
        Alphabet::Base64 => &BASE64,
        Alphabet::Base64Url => &BASE64URL,
    };
    let max_length = max_length.unwrap_or(usize::MAX);

    // Decode as much as possible.
    let (mut read, mut written) = match base.decode_mut(input, &mut output[..6 * input.len() / 8]) {
        Ok(olen) => (input.len(), olen),
        Err(DecodePartial { read, written, .. }) => (read, written),
    };

    // Backtrack to the last complete chunk that fits below the maximum output length.
    let extra_output = written - core::cmp::min(written, max_length) / 3 * 3;
    let mut extra_input = (8 * extra_output).div_ceil(6);
    written -= extra_output;
    loop {
        // Backtrack white-spaces.
        while 0 < read && base.interpret_byte(input[read - 1]).is_ignored() {
            read -= 1;
        }
        if extra_input == 0 {
            break;
        }
        // Backtrack one symbol.
        read -= 1;
        extra_input -= 1;
        debug_assert!(base.interpret_byte(input[read]).is_symbol().is_some());
    }

    // Parse the next chunk manually.
    let mut index = [0; 4]; // maps to index in input
    let mut index_len = 0;
    let mut index_pad = 4;
    let mut ipos = read;
    let remaining = max_length - written;
    if remaining == 0 {
        return DecodeMutResult {
            read,
            written,
            error: None,
        };
    }
    while ipos < input.len() {
        let byte = input[ipos];
        let position = ipos;
        ipos += 1;
        let kind = match base.interpret_byte(byte) {
            Character::Padding => unreachable!(),
            Character::Ignored => continue,
            Character::Symbol { .. } if index_pad < 4 => Some(DecodeKind::Padding),
            Character::Symbol { .. } => None,
            Character::Invalid if byte != b'=' => Some(DecodeKind::Symbol),
            Character::Invalid if index_len < 2 => Some(DecodeKind::Padding),
            Character::Invalid => {
                index_pad = core::cmp::min(index_pad, index_len);
                None
            }
        };
        if let Some(kind) = kind {
            return DecodeMutResult {
                read,
                written,
                error: Some(DecodeError { position, kind }),
            };
        }
        if index_len == 4 {
            debug_assert!(index_pad < 4);
            let error = Some(DecodeError {
                position,
                kind: DecodeKind::Padding,
            });
            return DecodeMutResult {
                read,
                written,
                error,
            };
        }
        index[index_len] = position;
        index_len += 1;
        if matches!(
            (core::cmp::min(index_len, index_pad), remaining),
            (3, 1) | (4, 2)
        ) {
            return DecodeMutResult {
                read,
                written,
                error: None,
            };
        }
    }
    debug_assert!(index_len <= 4 && index_pad <= 4);
    debug_assert!(index_len < 4 || index_pad < 4);

    // Process the last chunk.
    if index_len == 0 {
        return DecodeMutResult {
            read: input.len(),
            written,
            error: None,
        };
    }
    let check = match (last_chunk_handling, index_len, index_pad) {
        (Loose, 1, _) | (Loose, 0..4, 0..4) | (Strict, 0..4, _) => {
            let error = Some(DecodeError {
                position: ipos,
                kind: DecodeKind::Length,
            });
            return DecodeMutResult {
                read,
                written,
                error,
            };
        }
        (Strict, _, _) => true,
        (StopBeforePartial, 0..4, _) => {
            return DecodeMutResult {
                read,
                written,
                error: None,
            };
        }
        (Loose | StopBeforePartial, _, _) => false,
    };
    let iend = core::cmp::min(index_len, index_pad);
    let oend = iend - 1;
    let mut ichunk = [b'A'; 4];
    for i in 0..iend {
        ichunk[i] = input[index[i]];
    }
    let mut ochunk = [0; 3];
    let rchunk = base.decode_mut(&ichunk, &mut ochunk);
    debug_assert_eq!(rchunk, Ok(3));
    if check && iend < 4 && ochunk[oend] != 0 {
        let error = Some(DecodeError {
            position: index[iend],
            kind: DecodeKind::Trailing,
        });
        return DecodeMutResult {
            read,
            written,
            error,
        };
    }
    output[written..][..oend].copy_from_slice(&ochunk[..oend]);
    read = input.len();
    written += oend;
    DecodeMutResult {
        read,
        written,
        error: None,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DecodeResult {
    pub(crate) read: usize,
    pub(crate) output: Vec<u8>,
    pub(crate) error: Option<DecodeError>,
}

/// Decodes `input` in `output` according to the given parameters.
pub(crate) fn decode(
    input: &[u8],
    alphabet: Alphabet,
    last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
) -> DecodeResult {
    let mut output = vec![0; 6 * input.len() / 8];
    let DecodeMutResult {
        read,
        written,
        error,
    } = decode_mut(
        input,
        &mut output,
        alphabet,
        last_chunk_handling,
        max_length,
    );
    debug_assert!(written <= output.len());
    output.truncate(written);
    DecodeResult {
        read,
        output,
        error,
    }
}

pub(crate) fn encode(input: &[u8], alphabet: Alphabet, omit_padding: bool) -> String {
    let base = match (alphabet, omit_padding) {
        (Alphabet::Base64, false) => &data_encoding::BASE64,
        (Alphabet::Base64, true) => &data_encoding::BASE64_NOPAD,
        (Alphabet::Base64Url, false) => &data_encoding::BASE64URL,
        (Alphabet::Base64Url, true) => &data_encoding::BASE64URL_NOPAD,
    };

    let mut output = String::with_capacity(base.encode_len(input.len()));
    base.encode_append(input, &mut output);
    output
}

const BASE64: Encoding = new_encoding! {
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    ignore: " \t\n\x0C\r",
    check_trailing_bits: false,
};

const BASE64URL: Encoding = new_encoding! {
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    ignore: " \t\n\x0C\r",
    check_trailing_bits: false,
};
