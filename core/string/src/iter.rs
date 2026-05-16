use std::iter::FusedIterator;

use crate::str::RopeSlice;
use crate::{CodePoint, JsStr};

#[derive(Debug, Clone)]
enum IterInner<'a> {
    U8(std::iter::Copied<std::slice::Iter<'a, u8>>),
    U16(std::iter::Copied<std::slice::Iter<'a, u16>>),
    Rope(Box<RopeIter<'a>>),
}

#[derive(Debug, Clone)]
struct RopeIter<'a> {
    front_stack: Vec<RopeSlice<'a>>,
    front_current: Option<Iter<'a>>,
    back_stack: Vec<RopeSlice<'a>>,
    back_current: Option<Iter<'a>>,
    len: usize,
}

/// Iterator over a [`JsStr`].
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: IterInner<'a>,
}

impl<'a> Iter<'a> {
    #[inline]
    pub(crate) fn new(s: JsStr<'a>) -> Self {
        let len = s.len();
        let inner = match s {
            JsStr::Latin1(s) => IterInner::U8(s.iter().copied()),
            JsStr::Utf16(s) => IterInner::U16(s.iter().copied()),
            JsStr::Rope(r) => IterInner::Rope(Box::new(RopeIter {
                front_stack: vec![r],
                front_current: None,
                back_stack: Vec::new(),
                back_current: None,
                len,
            })),
        };
        Iter { inner }
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            IterInner::U8(iter) => iter.next().map(u16::from),
            IterInner::U16(iter) => iter.next(),
            IterInner::Rope(rope) => rope.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            IterInner::U8(iter) => iter.next_back().map(u16::from),
            IterInner::U16(iter) => iter.next_back(),
            IterInner::Rope(rope) => rope.next_back(),
        }
    }
}

impl Iterator for RopeIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        loop {
            if let Some(ref mut iter) = self.front_current {
                if let Some(cu) = iter.next() {
                    self.len -= 1;
                    return Some(cu);
                }
                self.front_current = None;
            }

            if let Some(slice) = self.front_stack.pop() {
                if slice.header.vtable.kind == crate::JsStringKind::Rope {
                    // SAFETY: The header is guaranteed to be a `RopeString` because the kind is `Rope`.
                    let r = unsafe {
                        &*std::ptr::from_ref(slice.header).cast::<crate::vtable::RopeString>()
                    };
                    let left_len = r.left.len();

                    if slice.start < left_len {
                        let left_end = std::cmp::min(slice.end, left_len);
                        if slice.end > left_len {
                            self.front_stack.push(RopeSlice {
                                // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                                header: unsafe { &*r.right.ptr.as_ptr().cast() },
                                start: 0,
                                end: slice.end - left_len,
                            });
                        }
                        self.front_stack.push(RopeSlice {
                            // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                            header: unsafe { &*r.left.ptr.as_ptr().cast() },
                            start: slice.start,
                            end: left_end,
                        });
                    } else {
                        self.front_stack.push(RopeSlice {
                            // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                            header: unsafe { &*r.right.ptr.as_ptr().cast() },
                            start: slice.start - left_len,
                            end: slice.end - left_len,
                        });
                    }
                } else {
                    let s = (slice.header.vtable.as_str)(slice.header)
                        .get(slice.start..slice.end)
                        .expect("valid slice");
                    self.front_current = Some(s.iter());
                }
                continue;
            }

            if let Some(cu) = self.back_current.as_mut().and_then(Iterator::next) {
                self.len -= 1;
                return Some(cu);
            }
            if let Some(slice) = self.back_stack.pop() {
                self.front_stack.push(slice);
                continue;
            }
            return None;
        }
    }
}

impl DoubleEndedIterator for RopeIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        loop {
            if let Some(ref mut iter) = self.back_current {
                if let Some(cu) = iter.next_back() {
                    self.len -= 1;
                    return Some(cu);
                }
                self.back_current = None;
            }

            if let Some(slice) = self.back_stack.pop() {
                if slice.header.vtable.kind == crate::JsStringKind::Rope {
                    // SAFETY: The header is guaranteed to be a `RopeString` because the kind is `Rope`.
                    let r = unsafe {
                        &*std::ptr::from_ref(slice.header).cast::<crate::vtable::RopeString>()
                    };
                    let left_len = r.left.len();

                    if slice.end > left_len {
                        let right_start = std::cmp::max(
                            0,
                            (slice.start.cast_signed() - left_len.cast_signed()).max(0),
                        )
                        .cast_unsigned();
                        if slice.start < left_len {
                            self.back_stack.push(RopeSlice {
                                // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                                header: unsafe { &*r.left.ptr.as_ptr().cast() },
                                start: slice.start,
                                end: left_len,
                            });
                        }
                        self.back_stack.push(RopeSlice {
                            // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                            header: unsafe { &*r.right.ptr.as_ptr().cast() },
                            start: right_start,
                            end: slice.end - left_len,
                        });
                    } else {
                        self.back_stack.push(RopeSlice {
                            // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                            header: unsafe { &*r.left.ptr.as_ptr().cast() },
                            start: slice.start,
                            end: slice.end,
                        });
                    }
                } else {
                    let s = (slice.header.vtable.as_str)(slice.header)
                        .get(slice.start..slice.end)
                        .expect("valid slice");
                    self.back_current = Some(s.iter());
                }
                continue;
            }

            if let Some(cu) = self
                .front_current
                .as_mut()
                .and_then(DoubleEndedIterator::next_back)
            {
                self.len -= 1;
                return Some(cu);
            }
            if let Some(slice) = self.front_stack.pop() {
                self.back_stack.push(slice);
                continue;
            }
            return None;
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
            IterInner::Rope(rope) => rope.len,
        }
    }
}

#[derive(Debug, Clone)]
enum WindowsInner<'a> {
    U8(std::slice::Windows<'a, u8>),
    U16(std::slice::Windows<'a, u16>),
    Rope {
        slice: RopeSlice<'a>,
        size: usize,
        index: usize,
    },
}

/// An iterator over overlapping subslices of length size.
///
/// This struct is created by the `windows` method.
#[derive(Debug, Clone)]
pub struct Windows<'a> {
    inner: WindowsInner<'a>,
}

impl<'a> Windows<'a> {
    #[inline]
    pub(crate) fn new(string: JsStr<'a>, size: usize) -> Self {
        let inner = match string {
            JsStr::Latin1(v) => WindowsInner::U8(v.windows(size)),
            JsStr::Utf16(v) => WindowsInner::U16(v.windows(size)),
            JsStr::Rope(r) => WindowsInner::Rope {
                slice: r,
                size,
                index: 0,
            },
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
            WindowsInner::Rope { slice, size, index } => {
                if *index + *size <= slice.len() {
                    let res = JsStr::Rope(RopeSlice {
                        header: slice.header,
                        start: slice.start + *index,
                        end: slice.start + *index + *size,
                    });
                    *index += 1;
                    Some(res)
                } else {
                    None
                }
            }
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
            WindowsInner::Rope { slice, size, index } => {
                if *index + *size <= slice.len() {
                    slice.len() - *index - *size + 1
                } else {
                    0
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum CodePointsIterInner<'a> {
    Latin1(std::iter::Copied<std::slice::Iter<'a, u8>>),
    Utf16(std::char::DecodeUtf16<std::iter::Copied<std::slice::Iter<'a, u16>>>),
    Rope(Box<RopeCodePointsIter<'a>>),
}

#[derive(Debug, Clone)]
pub(crate) struct RopeCodePointsIter<'a> {
    stack: Vec<RopeSlice<'a>>,
    current: Option<(Option<crate::JsString>, CodePointsIter<'a>)>,
}

#[derive(Debug, Clone)]
pub struct CodePointsIter<'a> {
    inner: CodePointsIterInner<'a>,
}

impl<'a> CodePointsIter<'a> {
    #[inline]
    pub(crate) fn new(s: JsStr<'a>) -> Self {
        let inner = match s {
            JsStr::Latin1(s) => CodePointsIterInner::Latin1(s.iter().copied()),
            JsStr::Utf16(s) => CodePointsIterInner::Utf16(char::decode_utf16(s.iter().copied())),
            JsStr::Rope(r) => CodePointsIterInner::Rope(Box::new(RopeCodePointsIter {
                stack: vec![r],
                current: None,
            })),
        };
        CodePointsIter { inner }
    }

    #[inline]
    pub(super) fn rope(s: &crate::JsString) -> Self {
        let header =
            // SAFETY: The pointer in a `JsString` is always valid.
            unsafe { s.ptr.as_ref() };
        CodePointsIter {
            inner: CodePointsIterInner::Rope(Box::new(RopeCodePointsIter {
                stack: vec![RopeSlice {
                    // SAFETY: The pointer in a `JsString` is always valid.
                    header: unsafe { &*s.ptr.as_ptr().cast() },
                    start: 0,
                    end: header.len,
                }],
                current: None,
            })),
        }
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
            CodePointsIterInner::Rope(rope) => rope.next(),
        }
    }
}

impl<'a> Iterator for RopeCodePointsIter<'a> {
    type Item = CodePoint;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((_, ref mut iter)) = self.current {
                if let Some(cp) = iter.next() {
                    return Some(cp);
                }
                self.current = None;
            }

            let slice = self.stack.pop()?;

            if slice.header.vtable.kind == crate::JsStringKind::Rope {
                // SAFETY: The header is guaranteed to be a `RopeString` because the kind is `Rope`.
                let r = unsafe {
                    &*std::ptr::from_ref(slice.header).cast::<crate::vtable::RopeString>()
                };
                let left_len = r.left.len();

                if slice.start < left_len {
                    let left_end = std::cmp::min(slice.end, left_len);
                    if slice.end > left_len {
                        self.stack.push(RopeSlice {
                            // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                            header: unsafe { &*r.right.ptr.as_ptr().cast() },
                            start: 0,
                            end: slice.end - left_len,
                        });
                    }
                    self.stack.push(RopeSlice {
                        // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                        header: unsafe { &*r.left.ptr.as_ptr().cast() },
                        start: slice.start,
                        end: left_end,
                    });
                } else {
                    self.stack.push(RopeSlice {
                        // SAFETY: The child pointers in a `RopeString` are always valid `JsStringHeader` pointers.
                        header: unsafe { &*r.right.ptr.as_ptr().cast() },
                        start: slice.start - left_len,
                        end: slice.end - left_len,
                    });
                }
            } else {
                let s = (slice.header.vtable.as_str)(slice.header)
                    .get(slice.start..slice.end)
                    .expect("valid slice");
                let iter = s.code_points();
                // SAFETY: The lifetime of the code points is tied to the rope string, which is guaranteed to live for 'a.
                let iter =
                    unsafe { std::mem::transmute::<CodePointsIter<'_>, CodePointsIter<'a>>(iter) };
                self.current = Some((None, iter));
            }
        }
    }
}

impl FusedIterator for CodePointsIter<'_> {}
