use crate::str::JsStrVariant;
use crate::vtable::{JsStringHeader, JsStringVTable};
use crate::{JsStr, JsString, JsStringKind};
use std::cell::OnceCell;
use std::ptr::{self, NonNull};

/// Fibonacci numbers for rope balancing thresholds.
/// `F(n) = Fib(n + 2)`. A rope of depth `n` is balanced if its length >= `F(n)`.
static FIBONACCI_THRESHOLDS: [usize; 64] = [
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
    28657,
    46368,
    75025,
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
    433_494_437,
    701_408_733,
    1_134_903_170,
    1_836_311_903,
    2_971_215_073,
    4_807_526_976,
    7_778_742_049,
    12_586_269_025,
    20_365_011_074,
    32_951_280_099,
    53_316_291_173,
    86_267_571_272,
    139_583_862_445,
    225_851_433_717,
    365_435_296_162,
    591_286_729_879,
    956_722_026_041,
    1_548_008_755_920,
    2_504_730_781_961,
    4_052_739_537_881,
    6_557_470_319_842,
    10_610_209_857_723,
    17_167_680_177_565,
];

/// Static vtable for rope strings.
pub(crate) static ROPE_VTABLE: JsStringVTable = JsStringVTable {
    as_str: rope_as_str,
    code_points: rope_code_points,
    code_unit_at: rope_code_unit_at,
    dealloc: rope_dealloc,
    kind: JsStringKind::Rope,
};

pub(crate) enum Flattened {
    Latin1(Box<[u8]>),
    Utf16(Box<[u16]>),
}

/// A rope string that is a tree of other strings.
#[repr(C)]
pub(crate) struct RopeString {
    /// Standardized header for all strings.
    pub(crate) header: JsStringHeader,
    pub(crate) left: JsString,
    pub(crate) right: JsString,
    // Using a raw buffer cache instead of `JsString` in `OnceCell` solves the "refcount aliasing"
    // problem. Storing a refcounted `JsString` inside a `OnceCell` could lead to ownership cycles
    // or use-after-free errors if shared nodes were dropped while still cached.
    // By storing raw `Box<[u8]>` or `Box<[u16]>`, we decouple the cache from the refcounting system.
    flattened: OnceCell<Flattened>,
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

        let d = depth as usize;

        // Classical Fibonacci Weight Invariant:
        // A rope of depth `d` is considered balanced if its length is at least `Fib(d + 2)`.
        // If the current length is less than the threshold for the current depth, we rebalance.
        // This alone guarantees logarithmic depth while ensuring rebalancing happens only O(log n) times.
        if d >= FIBONACCI_THRESHOLDS.len() || len < FIBONACCI_THRESHOLDS[d] {
            // If the string is small, just flatten it to a sequence for maximum efficiency.
            if len < 512 {
                return JsString::concat_array(&[left.as_str(), right.as_str()]);
            }

            // Otherwise, collect leaves and rebuild a balanced tree.
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
            flattened: OnceCell::new(),
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

                // If the child is already flattened, don't descend into it; just use its cached result.
                // This prevents exponential traversal of shared subtrees (DAGs) during rebalancing.
                if let Some(flat) = r.flattened.get() {
                    let s = match flat {
                        Flattened::Latin1(b) => JsString::from(JsStr::latin1(b)),
                        Flattened::Utf16(b) => JsString::from(JsStr::utf16(b)),
                    };
                    leaves.push(s);
                } else {
                    stack.push(r.right.clone());
                    stack.push(r.left.clone());
                }
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

    let flattened = this.flattened.get_or_init(|| {
        // SAFETY: Temporary handle for traversal.
        let root_handle = unsafe {
            let rc_ptr = (&raw const header.refcount).cast::<std::sync::atomic::AtomicUsize>();
            (*rc_ptr).fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            JsString::from_raw(NonNull::new_unchecked(ptr::from_ref(header).cast_mut()))
        };

        // Pass 1: Determine encoding using a seen-set to handle DAGs in O(Nodes).
        // This prevents exponential traversal (O(2^depth)) in highly shared trees.
        // By checking `flattened.get()` first, we effectively prune the traversal
        // using previously computed results, ensuring each internal node is visited
        // effectively only once per flattening call.
        let mut is_latin1 = true;
        let mut stack = vec![root_handle.clone()];
        let mut seen = rustc_hash::FxHashSet::default();
        while let Some(current) = stack.pop() {
            if current.kind() == JsStringKind::Rope {
                // SAFETY: We know the kind is `Rope`, so it's safe to cast the pointer to `RopeString`.
                let r: &RopeString = unsafe { &*current.ptr.as_ptr().cast::<RopeString>() };

                // If the child is already flattened, use its cached result to determine encoding.
                // This is a major optimization for DAGs.
                if let Some(flat) = r.flattened.get() {
                    match flat {
                        Flattened::Utf16(_) => {
                            is_latin1 = false;
                            break;
                        }
                        Flattened::Latin1(_) => {}
                    }
                } else if seen.insert(current.ptr.as_ptr()) {
                    stack.push(r.right.clone());
                    stack.push(r.left.clone());
                }
            } else if !current.as_str().is_latin1() {
                is_latin1 = false;
                break;
            }
        }

        let len = header.len;
        if is_latin1 {
            let mut buffer = Vec::with_capacity(len);
            let mut stack = vec![root_handle];
            while let Some(current) = stack.pop() {
                if current.kind() == JsStringKind::Rope {
                    // SAFETY: We know the kind is `Rope`, so it's safe to cast the pointer to `RopeString`.
                    let r: &RopeString = unsafe { &*current.ptr.as_ptr().cast::<RopeString>() };
                    if let Some(Flattened::Latin1(b)) = r.flattened.get() {
                        buffer.extend_from_slice(b);
                    } else {
                        stack.push(r.right.clone());
                        stack.push(r.left.clone());
                    }
                } else {
                    buffer.extend_from_slice(current.as_str().as_latin1().unwrap());
                }
            }
            Flattened::Latin1(buffer.into_boxed_slice())
        } else {
            let mut buffer = Vec::with_capacity(len);
            let mut stack = vec![root_handle];
            while let Some(current) = stack.pop() {
                if current.kind() == JsStringKind::Rope {
                    // SAFETY: We know the kind is `Rope`, so it's safe to cast the pointer to `RopeString`.
                    let r: &RopeString = unsafe { &*current.ptr.as_ptr().cast::<RopeString>() };
                    if let Some(flat) = r.flattened.get() {
                        match flat {
                            Flattened::Latin1(b) => buffer.extend(b.iter().copied().map(u16::from)),
                            Flattened::Utf16(b) => buffer.extend_from_slice(b),
                        }
                    } else {
                        stack.push(r.right.clone());
                        stack.push(r.left.clone());
                    }
                } else {
                    let variant = current.as_str().variant();
                    match variant {
                        JsStrVariant::Latin1(s) => buffer.extend(s.iter().copied().map(u16::from)),
                        JsStrVariant::Utf16(s) => buffer.extend_from_slice(s),
                    }
                }
            }
            Flattened::Utf16(buffer.into_boxed_slice())
        }
    });

    match flattened {
        Flattened::Latin1(b) => JsStr::latin1(b),
        Flattened::Utf16(b) => JsStr::utf16(b),
    }
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
    // We derive the refcount pointer from a raw pointer to avoid Frozen tags from the shared reference `header`.
    unsafe {
        let header_ptr: *const JsStringHeader = header;
        let rc_ptr = (&raw const (*header_ptr).refcount).cast::<std::sync::atomic::AtomicUsize>();
        (*rc_ptr).fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // SAFETY: We just incremented the refcount, so we can safely create a new handle.
        let s = JsString::from_raw(NonNull::new_unchecked(header_ptr.cast_mut()));
        crate::iter::CodePointsIter::rope(s)
    }
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
