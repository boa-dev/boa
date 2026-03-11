use crate::vtable::{JsStringVTable, RawJsString};
use crate::{JsStr, JsString, JsStringKind};
use std::cell::OnceCell;
use std::ptr::NonNull;

/// Static vtable for rope strings.
pub(crate) static ROPE_VTABLE: JsStringVTable = JsStringVTable {
    as_str: rope_as_str,
    code_points: rope_code_points,
    code_unit_at: rope_code_unit_at,
    dealloc: rope_dealloc,
};

/// A rope string that is a tree of other strings.
#[repr(C)]
pub(crate) struct RopeString {
    /// Standardized header for all strings.
    pub(crate) header: RawJsString,
    pub(crate) left: JsString,
    pub(crate) right: JsString,
    flattened: OnceCell<JsString>,
    pub(crate) depth: u8,
}

impl RopeString {
    /// Create a new rope string.
    ///
    /// This will auto-flatten if the depth exceeds the limit.
    #[inline]
    #[must_use]
    pub(crate) fn create(left: JsString, right: JsString) -> JsString {
        let left_depth = left.depth();
        let right_depth = right.depth();
        let depth = std::cmp::max(left_depth, right_depth) + 1;

        if depth > 32 {
            // Auto-flatten if we hit the depth limit, unless the string is "insanely" large.
            // This bounds access time and recursion depth for other components.
            if left.len() + right.len() < 1_000_000 {
                let mut vec = Vec::with_capacity(left.len() + right.len());
                for s in [&left, &right] {
                    match s.variant() {
                        crate::JsStrVariant::Latin1(l) => {
                            vec.extend(l.iter().map(|&b| u16::from(b)));
                        }
                        crate::JsStrVariant::Utf16(u) => vec.extend_from_slice(u),
                    }
                }
                return JsString::from(&vec[..]);
            }
        }

        let rope = Box::new(Self {
            header: RawJsString {
                vtable: &ROPE_VTABLE,
                len: left.len() + right.len(),
                refcount: 1,
                kind: JsStringKind::Rope,
                hash: 0,
            },
            left,
            right,
            flattened: OnceCell::new(),
            depth,
        });

        // SAFETY: The `rope` is leaked as a raw pointer and wrapped in `NonNull`.
        // The `RawJsString` header is at the start of `RopeString`.
        unsafe { JsString::from_raw(NonNull::from(Box::leak(rope)).cast()) }
    }

    #[inline]
    pub(crate) fn depth(&self) -> u8 {
        self.depth
    }

    /// Casts a `NonNull<RawJsString>` to `&Self`.
    ///
    /// # Safety
    /// The caller must ensure the pointer is valid and of the correct kind.
    #[inline]
    pub(crate) unsafe fn from_vtable<'a>(ptr: NonNull<RawJsString>) -> &'a Self {
        // SAFETY: The caller must ensure the pointer is valid and of the correct kind.
        unsafe { ptr.cast().as_ref() }
    }
}

#[inline]
fn rope_dealloc(ptr: NonNull<RawJsString>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    // The pointer is guaranteed to be a valid `NonNull<RawJsString>` pointing to a `RopeString`.
    unsafe {
        drop(Box::from_raw(ptr.cast::<RopeString>().as_ptr()));
    }
}

#[inline]
fn rope_as_str(ptr: NonNull<RawJsString>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &RopeString = unsafe { ptr.cast().as_ref() };

    // Lazy flattening.
    let flattened = this.flattened.get_or_init(|| {
        let mut vec = Vec::with_capacity(this.header.len);
        // We need an iterative approach to avoid stack overflow for deep trees.
        let mut stack = Vec::with_capacity(this.depth as usize + 1);
        stack.push(&this.right);
        stack.push(&this.left);

        while let Some(s) = stack.pop() {
            match s.kind() {
                JsStringKind::Rope => {
                    // SAFETY: s is a Rope.
                    let rope: &RopeString = unsafe { s.ptr.cast().as_ref() };
                    stack.push(&rope.right);
                    stack.push(&rope.left);
                }
                _ => match s.variant() {
                    crate::JsStrVariant::Latin1(l) => vec.extend(l.iter().map(|&b| u16::from(b))),
                    crate::JsStrVariant::Utf16(u) => vec.extend_from_slice(u),
                },
            }
        }
        debug_assert_eq!(vec.len(), this.header.len);
        JsString::from(&vec[..])
    });

    flattened.as_str()
}

#[inline]
fn rope_code_points(ptr: NonNull<RawJsString>) -> crate::iter::CodePointsIter<'static> {
    // SAFETY: validated the string outside this function.
    let s = unsafe { JsString::from_raw(ptr) };
    crate::iter::CodePointsIter::rope(s)
}

#[inline]
fn rope_code_unit_at(ptr: NonNull<RawJsString>, mut index: usize) -> Option<u16> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let mut current: &RopeString = unsafe { ptr.cast().as_ref() };

    loop {
        if index >= current.header.len {
            return None;
        }

        let left_len = current.left.len();
        if index < left_len {
            match current.left.kind() {
                JsStringKind::Rope => {
                    // SAFETY: current.left is a Rope.
                    current = unsafe { current.left.ptr.cast().as_ref() };
                }
                _ => {
                    // SAFETY: `current.left` is not a `Rope`, so we can safely get the code unit.
                    return current.left.code_unit_at(index);
                }
            }
        } else {
            index -= left_len;
            match current.right.kind() {
                JsStringKind::Rope => {
                    // SAFETY: current.right is a Rope.
                    current = unsafe { current.right.ptr.cast().as_ref() };
                }
                _ => {
                    // SAFETY: `current.right` is not a `Rope`, so we can safely get the code unit.
                    return current.right.code_unit_at(index);
                }
            }
        }
    }
}
