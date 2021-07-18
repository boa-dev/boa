use crate::gc::{empty_trace, Finalize, Trace};
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::Cell,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr::{copy_nonoverlapping, NonNull},
};

use rustc_hash::FxHasher;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The inner representation of a [`JsString`].
#[repr(C)]
struct Inner {
    /// The utf8 length, the number of bytes.
    len: usize,

    /// The number of references to the string.
    ///
    /// When this reaches `0` the string is deallocated.
    refcount: Cell<usize>,

    /// Lazely computed hash
    hash: Cell<Option<u64>>,

    /// An empty array which is used to get the offset of string data.
    data: [u8; 0],
}

impl Inner {
    /// Create a new `Inner` from `&str`.
    #[inline]
    fn new(s: &str) -> NonNull<Self> {
        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let inner_layout = Layout::new::<Inner>();
        let (layout, offset) = inner_layout
            .extend(Layout::array::<u8>(s.len()).unwrap())
            .unwrap();

        let inner = unsafe {
            let inner = alloc(layout) as *mut Inner;

            // Write the first part, the Inner.
            inner.write(Inner {
                len: s.len(),
                refcount: Cell::new(1),
                hash: Cell::new(None),
                data: [0; 0],
            });

            // Get offset into the string data.
            let data = (*inner).data.as_mut_ptr();

            debug_assert!(std::ptr::eq(inner.cast::<u8>().add(offset), data));

            // Copy string data into data offset.
            copy_nonoverlapping(s.as_ptr(), data, s.len());

            inner
        };

        // Safety: We already know it's not null, so this is safe.
        unsafe { NonNull::new_unchecked(inner) }
    }

    /// Concatinate two string.
    #[inline]
    fn concat(x: &str, y: &str) -> NonNull<Inner> {
        let total_string_size = x.len() + y.len();

        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let inner_layout = Layout::new::<Inner>();
        let (layout, offset) = inner_layout
            .extend(Layout::array::<u8>(total_string_size).unwrap())
            .unwrap();

        let inner = unsafe {
            let inner = alloc(layout) as *mut Inner;

            // Write the first part, the Inner.
            inner.write(Inner {
                len: total_string_size,
                refcount: Cell::new(1),
                hash: Cell::new(None),
                data: [0; 0],
            });

            // Get offset into the string data.
            let data = (*inner).data.as_mut_ptr();

            debug_assert!(std::ptr::eq(inner.cast::<u8>().add(offset), data));

            // Copy the two string data into data offset.
            copy_nonoverlapping(x.as_ptr(), data, x.len());
            copy_nonoverlapping(y.as_ptr(), data.add(x.len()), y.len());

            inner
        };

        // Safety: We already know it's not null, so this is safe.
        unsafe { NonNull::new_unchecked(inner) }
    }

    /// Deallocate inner type with string data.
    #[inline]
    unsafe fn dealloc(x: NonNull<Inner>) {
        let len = (*x.as_ptr()).len;

        let inner_layout = Layout::new::<Inner>();
        let (layout, _offset) = inner_layout
            .extend(Layout::array::<u8>(len).unwrap())
            .unwrap();

        dealloc(x.as_ptr() as _, layout);
    }
}

/// This represents a JavaScript primitive string.
///
/// This is similar to `Rc<str>`. But unlike `Rc<str>` which stores the length
/// on the stack and a pointer to the data (this is also known as fat pointers).
/// The `JsString` length and data is stored on the heap. and just an non-null
/// pointer is kept, so its size is the size of a pointer.
pub struct JsString {
    inner: NonNull<Inner>,
    _marker: PhantomData<std::rc::Rc<str>>,
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(feature = "deser")]
impl Serialize for JsString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature = "deser")]
impl<'de> Deserialize<'de> for JsString {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl JsString {
    /// Create a new JavaScript string.
    #[inline]
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref();
        Self {
            inner: Inner::new(s),
            _marker: PhantomData,
        }
    }

    /// Concatinate two string.
    pub fn concat<T, U>(x: T, y: U) -> JsString
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let x = x.as_ref();
        let y = y.as_ref();

        Self {
            inner: Inner::concat(x, y),
            _marker: PhantomData,
        }
    }

    /// Return the inner representation.
    #[inline]
    fn inner(&self) -> &Inner {
        unsafe { self.inner.as_ref() }
    }

    /// Return the JavaScript string as a rust `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        let inner = self.inner();

        unsafe {
            let slice = std::slice::from_raw_parts(inner.data.as_ptr(), inner.len);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Gets the number of `JsString`s which point to this allocation.
    #[inline]
    pub fn refcount(this: &Self) -> usize {
        this.inner().refcount.get()
    }

    /// Has the hash been computed for this string.
    #[inline]
    pub fn has_hash(this: &Self) -> bool {
        this.inner().hash.get().is_some()
    }

    /// Compute the hash for this string, if not already and return it.
    #[inline]
    pub fn hash(this: &Self) -> u64 {
        let inner = this.inner();
        if let Some(hash) = inner.hash.get() {
            hash
        } else {
            let hash = {
                let mut hasher = FxHasher::default();
                this.as_str().hash(&mut hasher);
                hasher.finish()
            };

            inner.hash.set(Some(hash));
            hash
        }
    }

    /// Returns `true` if the two `JsString`s point to the same allocation (in a vein similar to [`ptr::eq`]).
    ///
    /// [`ptr::eq`]: std::ptr::eq
    #[inline]
    pub fn ptr_eq(x: &Self, y: &Self) -> bool {
        x.inner == y.inner
    }
}

impl Finalize for JsString {}

// Safety: [`JsString`] does not contain any objects which recquire trace,
// so this is safe.
unsafe impl Trace for JsString {
    empty_trace!();
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.inner();
        inner.refcount.set(inner.refcount.get() + 1);

        JsString {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl Drop for JsString {
    #[inline]
    fn drop(&mut self) {
        let inner = self.inner();
        if inner.refcount.get() == 1 {
            // Safety: If refcount is 1 and we call drop, that means this is the last
            // JsString which points to this memory allocation, so deallocating it is safe.
            unsafe {
                Inner::dealloc(self.inner);
            }
        } else {
            inner.refcount.set(inner.refcount.get() - 1);
        }
    }
}

impl std::fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Box<str>> for JsString {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::new(s)
    }
}

impl From<String> for JsString {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for JsString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for JsString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<JsString> for JsString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // If they point at the same memory allocation, then they are equal.
        if Self::ptr_eq(self, other) {
            return true;
        }

        self.as_str() == other.as_str()
    }
}

impl Eq for JsString {}

impl Hash for JsString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Self::hash(self).hash(state)
    }
}

impl PartialOrd for JsString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for JsString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq<str> for JsString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<JsString> for str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&str> for JsString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for &str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        *self == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::JsString;
    use std::mem::size_of;

    use rustc_hash::FxHasher;

    #[test]
    fn empty() {
        let _ = JsString::new("");
    }

    #[test]
    fn pointer_size() {
        assert_eq!(size_of::<JsString>(), size_of::<*const u8>());
        assert_eq!(size_of::<Option<JsString>>(), size_of::<*const u8>());
    }

    #[test]
    fn refcount() {
        let x = JsString::new("Hello wrold");
        assert_eq!(JsString::refcount(&x), 1);

        {
            let y = x.clone();
            assert_eq!(JsString::refcount(&x), 2);
            assert_eq!(JsString::refcount(&y), 2);

            {
                let z = y.clone();
                assert_eq!(JsString::refcount(&x), 3);
                assert_eq!(JsString::refcount(&y), 3);
                assert_eq!(JsString::refcount(&z), 3);
            }

            assert_eq!(JsString::refcount(&x), 2);
            assert_eq!(JsString::refcount(&y), 2);
        }

        assert_eq!(JsString::refcount(&x), 1);
    }

    #[test]
    fn ptr_eq() {
        let x = JsString::new("Hello");
        let y = x.clone();

        assert!(JsString::ptr_eq(&x, &y));

        let z = JsString::new("Hello");
        assert!(!JsString::ptr_eq(&x, &z));
        assert!(!JsString::ptr_eq(&y, &z));
    }

    #[test]
    fn as_str() {
        let s = "Hello";
        let x = JsString::new(s);

        assert_eq!(x.as_str(), s);
    }

    #[test]
    fn hash() {
        use std::hash::{Hash, Hasher};

        let s = "Hello, world!";
        let x = JsString::new(s);

        assert_eq!(x.as_str(), s);

        assert!(!JsString::has_hash(&x));

        let mut hasher = FxHasher::default();
        s.hash(&mut hasher);
        let s_hash = hasher.finish();

        let x_hash = JsString::hash(&x);

        assert!(JsString::has_hash(&x));

        assert_eq!(s_hash, x_hash);
    }

    #[test]
    fn concat() {
        let x = JsString::new("hello");
        let y = ", ";
        let z = JsString::new("world");
        let w = String::from("!");

        let xy = JsString::concat(x, y);
        assert_eq!(xy, "hello, ");
        assert_eq!(JsString::refcount(&xy), 1);

        let xyz = JsString::concat(xy, z);
        assert_eq!(xyz, "hello, world");
        assert_eq!(JsString::refcount(&xyz), 1);

        let xyzw = JsString::concat(xyz, w);
        assert_eq!(xyzw, "hello, world!");
        assert_eq!(JsString::refcount(&xyzw), 1);
    }
}
