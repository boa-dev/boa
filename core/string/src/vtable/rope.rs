use crate::vtable::{JsStringHeader, JsStringVTable};
use crate::{JsStr, JsString, JsStringKind};
use std::ptr::{self, NonNull};
use std::sync::OnceLock;

/// Fibonacci numbers for rope balancing thresholds.
/// `F[n] = Fib(n + 2)`. A rope of depth `n` is balanced if its length >= `F[n]`.
static FIBONACCI_THRESHOLDS: [usize; 41] = [
    1,
    2,
    3,
    5,
    8,
    13,
    21,
    34,
    55,
    89,
    144,
    233,
    377,
    610,
    987,
    1597,
    2584,
    4181,
    6765,
    10946,
    17711,
    28_657,
    46_368,
    75_025,
    121_393,
    196_418,
    317_811,
    514_229,
    832_040,
    1_346_269,
    2_178_309,
    3_524_578,
    5_702_887,
    9_227_465,
    14_930_352,
    24_157_817,
    39_088_169,
    63_245_986,
    102_334_155,
    165_580_141,
    267_914_296,
];

/// Static vtable for rope strings.
pub(crate) static ROPE_VTABLE: JsStringVTable = JsStringVTable {
    as_str: rope_as_str,
    code_points: rope_code_points,
    code_unit_at: rope_code_unit_at,
    dealloc: rope_dealloc,
    kind: JsStringKind::Rope,
};

/// A rope string that is a tree of other strings.
#[repr(C)]
pub(crate) struct RopeString {
    /// Standardized header for all strings.
    pub(crate) header: JsStringHeader,
    pub(crate) left: JsString,
    pub(crate) right: JsString,
    // We use `OnceLock` over `OnceCell` despite `JsString` being `!Sync`.
    // The reason is that rope flattening is structurally reentrant: `rope.as_str()` can internally
    // trigger `.as_str()` on its children, which might be identical shared references (DAG).
    // `OnceCell` explicitly panics on reentrant initialization, whereas `OnceLock` handles it.
    flattened: OnceLock<JsString>,
    pub(crate) depth: u8,
}

impl RopeString {
    /// Create a new rope string.
    ///
    /// This will rebalance if the rope becomes too deep relative to its length.
    #[inline]
    #[must_use]
    pub(crate) fn create(left: JsString, right: JsString) -> JsString {
        let depth = std::cmp::max(left.depth(), right.depth()) + 1;
        let len = left.len() + right.len();

        // Fibonacci rebalancing heuristic: A rope of depth n is balanced if its len >= Fib(n + 2).
        // If it's too deep, we rebalance or flatten.
        // This heuristic ensures rebalancing is rare (only for degenerate trees), making its O(N) cost amortized.
        if depth as usize >= FIBONACCI_THRESHOLDS.len()
            || (depth > 8 && len < FIBONACCI_THRESHOLDS[depth as usize])
        {
            // If the string is small, just flatten it to a sequence for maximum efficiency.
            if len < 512 {
                return JsString::concat_array(&[left.as_str(), right.as_str()]);
            }

            // Otherwise, collect leaves and rebuild a balanced tree.
            // We use a slightly larger capacity for leaves as depth is an under-estimate for non-degenerate trees.
            let mut leaves = Vec::with_capacity(std::cmp::max(depth as usize * 2, 16));
            Self::collect_leaves(&left, &mut leaves);
            Self::collect_leaves(&right, &mut leaves);
            return JsString::concat_leaves_balanced(&leaves);
        }

        let rope = Box::new(Self {
            header: JsStringHeader {
                vtable: &ROPE_VTABLE,
                len,
                refcount: 1,
                hash: 0,
            },
            left,
            right,
            flattened: OnceLock::new(),
            depth,
        });

        // SAFETY: The `rope` is leaked as a raw pointer and wrapped in `NonNull`.
        // The `JsStringHeader` header is at the start of `RopeString`.
        unsafe { JsString::from_raw(NonNull::from(Box::leak(rope)).cast()) }
    }

    /// Internal helper to collect all leaf strings of a rope.
    pub(crate) fn collect_leaves(s: &JsString, leaves: &mut Vec<JsString>) {
        let mut stack = vec![s.clone()];
        while let Some(current) = stack.pop() {
            if current.kind() == JsStringKind::Rope {
                // SAFETY: kind is Rope.
                let r = unsafe { Self::from_vtable(current.ptr) };
                stack.push(r.right.clone());
                stack.push(r.left.clone());
            } else if !current.is_empty() {
                leaves.push(current);
            }
        }
    }

    #[inline]
    pub(crate) fn depth(&self) -> u8 {
        self.depth
    }

    /// Casts a `NonNull<JsStringHeader>` to `&Self`.
    ///
    /// # Safety
    /// The caller must ensure the pointer is valid and of the correct kind.
    #[inline]
    pub(crate) unsafe fn from_vtable<'a>(ptr: NonNull<JsStringHeader>) -> &'a Self {
        // SAFETY: The caller must ensure the pointer is valid and of the correct kind.
        unsafe { ptr.cast().as_ref() }
    }
}

#[inline]
fn rope_dealloc(ptr: NonNull<JsStringHeader>) {
    // We use a stack to iteratively drop rope nodes and avoid stack overflow.
    let mut stack = vec![ptr];
    while let Some(current_ptr) = stack.pop() {
        // SAFETY: The pointer is guaranteed to be a valid `NonNull<JsStringHeader>` pointing to a `RopeString`
        // that is ready to be deallocated (refcount reached 0).
        unsafe {
            // SAFETY: The pointer was created from a Box in `create` and hasn't been freed yet.
            let rope_ptr = current_ptr.cast::<RopeString>();
            // SAFETY: We own this pointer now conceptually.
            let mut rope_box = Box::from_raw(rope_ptr.as_ptr());

            // Check children. If they are ropes and we are the last reference, defer their deallocation.
            // This prevents the recursive drop of fields.
            let left = std::mem::replace(&mut rope_box.left, crate::StaticJsStrings::EMPTY_STRING);
            if left.kind() == JsStringKind::Rope && left.refcount() == Some(1) {
                stack.push(left.ptr);
                std::mem::forget(left);
            }
            let right =
                std::mem::replace(&mut rope_box.right, crate::StaticJsStrings::EMPTY_STRING);
            if right.kind() == JsStringKind::Rope && right.refcount() == Some(1) {
                stack.push(right.ptr);
                std::mem::forget(right);
            }
            // rope_box is dropped here. Its remaining fields (depth, OnceCell, and the empty JsStrings) are dropped normally.
        }
    }
}

#[inline]
fn rope_as_str(header: &JsStringHeader) -> JsStr<'_> {
    // SAFETY: The header is part of a RopeString and it's aligned.
    let this: &RopeString = unsafe { &*ptr::from_ref(header).cast::<RopeString>() };

    // Lazy flattening.
    let flattened = this.flattened.get_or_init(|| {
        let mut leaves = Vec::with_capacity(this.depth as usize * 2);
        let mut current_strings = Vec::with_capacity(this.depth as usize * 2);

        // We need an iterative approach to avoid stack overflow for deep trees.
        let mut stack: Vec<&JsString> = Vec::with_capacity(this.depth as usize + 1);
        stack.push(&this.right);
        stack.push(&this.left);

        while let Some(s) = stack.pop() {
            if s.kind() == JsStringKind::Rope {
                // SAFETY: s is a Rope.
                let rope: &RopeString = unsafe { s.ptr.cast().as_ref() };
                stack.push(&rope.right);
                stack.push(&rope.left);
            } else if !s.is_empty() {
                // To safely get `JsStr` with a long enough lifetime for `concat_array`,
                // we collect the `JsString`s and only then get their `as_str()`.
                // This is because `concat_array` requires `&[JsStr<'_>]`.
                current_strings.push(s.clone());
            }
        }

        for s in &current_strings {
            leaves.push(s.as_str());
        }

        JsString::concat_array(&leaves)
    });

    flattened.as_str()
}

#[inline]
fn rope_code_points(header: &JsStringHeader) -> crate::iter::CodePointsIter<'_> {
    // SAFETY: We are creating a new handle from a raw pointer, so we must increment the refcount
    // to avoid a use-after-free when the iterator's handle is dropped.
    // We also know that the kind is not static (since this is ROPE), so we can safely cast the refcount
    // pointer to an atomic for concurrent updates.
    // NOTE: Casting a non-atomic `usize` to `AtomicUsize` is technically undefined behavior in the Rust
    // strict provenance model, but it is a common pattern in JS engines where atomic and non-atomic
    // states share layout, and is practically safe on our supported platforms.
    unsafe {
        let rc_ptr = (&raw const header.refcount).cast::<std::sync::atomic::AtomicUsize>();
        (*rc_ptr).fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    // SAFETY: We just incremented the refcount, so we can safely create a new handle.
    let s = unsafe { JsString::from_raw(NonNull::from(header)) };
    crate::iter::CodePointsIter::rope(s)
}

#[inline]
fn rope_code_unit_at(header: &JsStringHeader, mut index: usize) -> Option<u16> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    // The pointer is guaranteed to be a valid `NonNull<JsStringHeader>` pointing to a `RopeString`.
    let mut current: &RopeString = unsafe { &*ptr::from_ref(header).cast::<RopeString>() };

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
