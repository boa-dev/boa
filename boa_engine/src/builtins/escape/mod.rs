//! Boa's implementation of ECMAScript's string escaping functions.
//!
//! The `escape()` function replaces all characters with escape sequences, with the exception of ASCII
//! word characters (A–Z, a–z, 0–9, _) and @*_+-./.
//!
//! The `unescape()` function replaces any escape sequence with the character that it represents.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-additional-properties-of-the-global-object

use crate::{
    context::intrinsics::Intrinsics, js_string, Context, JsArgs, JsObject, JsResult, JsValue,
};

use super::{BuiltInBuilder, BuiltInObject, IntrinsicObject};

/// The `escape` function
#[derive(Debug, Clone, Copy)]
pub(crate) struct Escape;

impl IntrinsicObject for Escape {
    fn init(intrinsics: &Intrinsics) {
        BuiltInBuilder::with_intrinsic::<Self>(intrinsics)
            .callable(escape)
            .name(Self::NAME)
            .length(1)
            .build();
    }
    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().escape().into()
    }
}

impl BuiltInObject for Escape {
    const NAME: &'static str = "escape";
}

/// Builtin JavaScript `escape ( string )` function.
fn escape(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    /// Returns `true` if the codepoint `cp` is part of the `unescapedSet`.
    fn is_unescaped(cp: u16) -> bool {
        let Ok(cp) = TryInto::<u8>::try_into(cp) else {
            return false;
        };

        // 4. Let unescapedSet be the string-concatenation of the ASCII word characters and "@*+-./".
        cp.is_ascii_alphanumeric() || [b'_', b'@', b'*', b'+', b'-', b'.', b'/'].contains(&cp)
    }

    // 1. Set string to ? ToString(string).
    let string = args.get_or_undefined(0).to_string(context)?;

    // 3. Let R be the empty String.
    let mut vec = Vec::with_capacity(string.len());

    // 2. Let len be the length of string.
    // 5. Let k be 0.
    // 6. Repeat, while k < len,
    //     a. Let C be the code unit at index k within string.
    for &cp in &*string {
        // b. If unescapedSet contains C, then
        if is_unescaped(cp) {
            // i. Let S be C.
            vec.push(cp);
            continue;
        }
        // c. Else,
        //     i. Let n be the numeric value of C.
        //     ii. If n < 256, then
        let c = if cp < 256 {
            //     1. Let hex be the String representation of n, formatted as an uppercase hexadecimal number.
            //     2. Let S be the string-concatenation of "%" and ! StringPad(hex, 2𝔽, "0", start).
            format!("%{cp:02X}")
        }
        //     iii. Else,
        else {
            //     1. Let hex be the String representation of n, formatted as an uppercase hexadecimal number.
            //     2. Let S be the string-concatenation of "%u" and ! StringPad(hex, 4𝔽, "0", start).
            format!("%u{cp:04X}")
        };
        // d. Set R to the string-concatenation of R and S.
        // e. Set k to k + 1.
        vec.extend(c.encode_utf16());
    }

    // 7. Return R.
    Ok(js_string!(vec).into())
}

/// The `unescape` function
#[derive(Debug, Clone, Copy)]
pub(crate) struct Unescape;

impl IntrinsicObject for Unescape {
    fn init(intrinsics: &Intrinsics) {
        BuiltInBuilder::with_intrinsic::<Self>(intrinsics)
            .callable(unescape)
            .name(Self::NAME)
            .length(1)
            .build();
    }
    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().unescape().into()
    }
}

impl BuiltInObject for Unescape {
    const NAME: &'static str = "unescape";
}

/// Builtin JavaScript `unescape ( string )` function.
fn unescape(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    /// Converts a char `cp` to its corresponding hex digit value.
    fn to_hex_digit(cp: u16) -> Option<u16> {
        char::from_u32(u32::from(cp))
            .and_then(|c| c.to_digit(16))
            .and_then(|d| d.try_into().ok())
    }

    // 1. Set string to ? ToString(string).
    let string = args.get_or_undefined(0).to_string(context)?;

    // 3. Let R be the empty String.
    let mut vec = Vec::with_capacity(string.len());

    let mut codepoints = <BufferedIterator<_, 6>>::new(string.iter().copied().fuse());

    // 2. Let len be the length of string.
    // 4. Let k be 0.
    // 5. Repeat, while k < len,
    loop {
        vec.extend_from_slice(codepoints.reset_buffer());

        // a. Let C be the code unit at index k within string.
        let Some(cp) = codepoints.next() else {
            break;
        };

        // b. If C is the code unit 0x0025 (PERCENT SIGN), then
        if cp != u16::from(b'%') {
            continue;
        }

        //     i. Let hexDigits be the empty String.
        //     ii. Let optionalAdvance be 0.
        let Some(next1) = codepoints.next() else {
            continue;
        };

        //     iii. If k + 5 < len and the code unit at index k + 1 within string is the code unit 0x0075 (LATIN SMALL LETTER U), then
        let unescaped_cp = if next1 == u16::from(b'u') {
            //         1. Set hexDigits to the substring of string from k + 2 to k + 6.
            //         2. Set optionalAdvance to 5.
            let Some(next1) = codepoints.next().and_then(to_hex_digit) else {
                continue;
            };

            let Some(next2) = codepoints.next().and_then(to_hex_digit) else {
                continue;
            };

            let Some(next3) = codepoints.next().and_then(to_hex_digit) else {
                continue;
            };

            let Some(next4) = codepoints.next().and_then(to_hex_digit) else {
                continue;
            };

            (next1 << 12) + (next2 << 8) + (next3 << 4) + next4
        }
        //     iv. Else if k + 3 ≤ len, then
        else {
            //         1. Set hexDigits to the substring of string from k + 1 to k + 3.
            //         2. Set optionalAdvance to 2.
            let Some(next1) = to_hex_digit(cp) else {
                continue;
            };

            let Some(next2) = codepoints.next().and_then(to_hex_digit) else {
                continue;
            };

            (next1 << 4) + next2
        };

        //     v. Let parseResult be ParseText(StringToCodePoints(hexDigits), HexDigits[~Sep]).
        //     vi. If parseResult is a Parse Node, then
        //         1. Let n be the MV of parseResult.
        //         2. Set C to the code unit whose numeric value is n.
        //         3. Set k to k + optionalAdvance.
        // c. Set R to the string-concatenation of R and C.
        // d. Set k to k + 1.
        vec.push(unescaped_cp);
        codepoints.reset_buffer();
    }
    // 6. Return R.
    Ok(js_string!(vec).into())
}

/// An iterator that buffers the result of `SIZE` items.
struct BufferedIterator<I, const SIZE: usize>
where
    I: Iterator,
{
    iterator: I,
    buffer: [I::Item; SIZE],
    buffered_count: usize,
}

impl<I, const SIZE: usize> BufferedIterator<I, SIZE>
where
    I: Iterator,
    I::Item: Default + Copy,
{
    /// Creates a new `BufferedIterator`.
    fn new(iterator: I) -> Self {
        Self {
            iterator,
            buffer: [I::Item::default(); SIZE],
            buffered_count: 0,
        }
    }

    /// Resets the inner buffer and returns the buffered items.
    fn reset_buffer(&mut self) -> &[I::Item] {
        let buffered = &self.buffer[..self.buffered_count];
        self.buffered_count = 0;
        buffered
    }
}

impl<I, const SIZE: usize> Iterator for BufferedIterator<I, SIZE>
where
    I: Iterator,
    I::Item: Copy,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iterator.next()?;
        self.buffer[self.buffered_count] = item;
        self.buffered_count += 1;
        Some(item)
    }
}
