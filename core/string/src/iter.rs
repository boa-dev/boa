use std::iter::FusedIterator;

use crate::{CodePoint, JsStr};

use super::JsStrVariant;

#[derive(Debug, Clone)]
enum IterInner<'a> {
    U8(std::iter::Copied<std::slice::Iter<'a, u8>>),
    U16(std::iter::Copied<std::slice::Iter<'a, u16>>),
}

/// Iterator over a [`JsStr`].
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: IterInner<'a>,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(s: JsStr<'a>) -> Self {
        let inner = match s.variant() {
            JsStrVariant::Latin1(s) => IterInner::U8(s.iter().copied()),
            JsStrVariant::Utf16(s) => IterInner::U16(s.iter().copied()),
        };
        Iter { inner }
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            IterInner::U8(iter) => iter.map(u16::from).next(),
            IterInner::U16(iter) => iter.next(),
        }
    }
}

impl FusedIterator for Iter<'_> {}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        match &self.inner {
            IterInner::U8(v) => v.len(),
            IterInner::U16(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
enum WindowsInner<'a> {
    U8(std::slice::Windows<'a, u8>),
    U16(std::slice::Windows<'a, u16>),
}

/// An iterator over overlapping subslices of length size.
///
/// This struct is created by the `windows` method.
#[derive(Debug, Clone)]
pub struct Windows<'a> {
    inner: WindowsInner<'a>,
}

impl<'a> Windows<'a> {
    pub(crate) fn new(string: JsStr<'a>, size: usize) -> Self {
        let inner = match string.variant() {
            JsStrVariant::Latin1(v) => WindowsInner::U8(v.windows(size)),
            JsStrVariant::Utf16(v) => WindowsInner::U16(v.windows(size)),
        };
        Self { inner }
    }
}

impl<'a> Iterator for Windows<'a> {
    type Item = JsStr<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            WindowsInner::U8(iter) => iter.next().map(JsStr::latin1),
            WindowsInner::U16(iter) => iter.next().map(JsStr::utf16),
        }
    }
}

impl FusedIterator for Windows<'_> {}

impl ExactSizeIterator for Windows<'_> {
    #[inline]
    fn len(&self) -> usize {
        match &self.inner {
            WindowsInner::U8(v) => v.len(),
            WindowsInner::U16(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
enum CodePointsIterInner<'a> {
    Latin1(std::iter::Copied<std::slice::Iter<'a, u8>>),
    Utf16(std::char::DecodeUtf16<std::iter::Copied<std::slice::Iter<'a, u16>>>),
}

#[derive(Debug, Clone)]
pub struct CodePointsIter<'a> {
    inner: CodePointsIterInner<'a>,
}

impl<'a> CodePointsIter<'a> {
    pub(crate) fn new(s: JsStr<'a>) -> Self {
        let inner = match s.variant() {
            JsStrVariant::Latin1(s) => CodePointsIterInner::Latin1(s.iter().copied()),
            JsStrVariant::Utf16(s) => {
                CodePointsIterInner::Utf16(char::decode_utf16(s.iter().copied()))
            }
        };
        CodePointsIter { inner }
    }
}

impl Iterator for CodePointsIter<'_> {
    type Item = CodePoint;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CodePointsIterInner::Latin1(iter) => {
                iter.next().map(|b| CodePoint::Unicode(char::from(b)))
            }
            CodePointsIterInner::Utf16(iter) => iter.next().map(|res| match res {
                Ok(c) => CodePoint::Unicode(c),
                Err(e) => CodePoint::UnpairedSurrogate(e.unpaired_surrogate()),
            }),
        }
    }
}

impl FusedIterator for CodePointsIter<'_> {}
