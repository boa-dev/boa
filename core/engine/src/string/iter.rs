use std::iter::FusedIterator;

use super::{slice::JsStringSliceVariant, JsStringSlice};

#[derive(Debug, Clone)]
enum Inner<'a> {
    Ascii(std::iter::Copied<std::slice::Iter<'a, u8>>),
    U8(std::str::EncodeUtf16<'a>, usize),
    U16(std::iter::Copied<std::slice::Iter<'a, u16>>),
}

/// Iterator over a [`JsString`].
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: Inner<'a>,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(s: JsStringSlice<'a>) -> Self {
        let inner = match s.variant() {
            JsStringSliceVariant::U8Ascii(s) => Inner::Ascii(s.iter().copied()),
            JsStringSliceVariant::U8NonAscii(s, len) => Inner::U8(s.encode_utf16(), len),
            JsStringSliceVariant::U16Ascii(s) | JsStringSliceVariant::U16NonAscii(s) => {
                Inner::U16(s.iter().copied())
            }
        };
        Iter { inner }
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Inner::Ascii(iter) => iter.map(u16::from).next(),
            Inner::U8(iter, _) => iter.next(),
            Inner::U16(iter) => iter.next(),
        }
    }
}

impl FusedIterator for Iter<'_> {}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        match &self.inner {
            Inner::Ascii(v) => v.len(),
            Inner::U8(_, len) => *len,
            Inner::U16(v) => v.len(),
        }
    }
}
