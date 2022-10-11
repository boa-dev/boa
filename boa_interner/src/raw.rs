use std::hash::Hash;

use rustc_hash::FxHashMap;

use crate::{fixed_string::FixedString, interned_str::InternedStr};

/// Raw string interner, generic by a char type.
#[derive(Debug)]
pub(super) struct RawInterner<Char> {
    // COMMENT FOR DEVS:
    // This interner works on the assumption that
    // `head` won't ever be reallocated, since this could invalidate
    // some of our stored pointers inside `spans`.
    // This means that any operation on `head` and `full` should be carefully
    // reviewed to not cause Undefined Behaviour.
    // `intern` has a more thorough explanation on this.
    //
    // Also, if you want to implement `shrink_to_fit` (and friends),
    // please check out https://github.com/Robbepop/string-interner/pull/47 first.
    // This doesn't implement that method, since implementing it increases
    // our memory footprint.
    symbol_cache: FxHashMap<InternedStr<Char>, usize>,
    spans: Vec<InternedStr<Char>>,
    head: FixedString<Char>,
    full: Vec<FixedString<Char>>,
}

impl<Char> Default for RawInterner<Char> {
    fn default() -> Self {
        Self {
            symbol_cache: FxHashMap::default(),
            spans: Vec::default(),
            head: FixedString::default(),
            full: Vec::default(),
        }
    }
}

impl<Char> RawInterner<Char> {
    /// Creates a new `RawInterner` with the specified capacity.
    #[inline]
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            symbol_cache: FxHashMap::default(),
            spans: Vec::with_capacity(capacity),
            head: FixedString::new(capacity),
            full: Vec::new(),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    pub(super) fn len(&self) -> usize {
        self.spans.len()
    }

    /// Returns `true` if the interner contains no interned strings.
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

impl<Char> RawInterner<Char>
where
    Char: Hash + Eq,
{
    /// Returns the index position for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    pub(super) fn get(&self, string: &[Char]) -> Option<usize> {
        // SAFETY:
        // `string` is a valid slice that doesn't outlive the
        // created `InternedStr`, so this is safe.
        unsafe {
            self.symbol_cache
                .get(&InternedStr::new(string.into()))
                .copied()
        }
    }

    /// Interns the given `'static` string.
    ///
    /// Returns the index of `string` within the interner.
    ///
    /// # Note
    ///
    /// This is more efficient than [`RawInterner::intern`], since it
    /// avoids storing `string` inside the interner.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    pub(super) fn intern_static(&mut self, string: &'static [Char]) -> usize {
        // SAFETY:
        // A static string reference is always valid, meaning it cannot outlive
        // the lifetime of the created `InternedStr`. This makes this
        // operation safe.
        let string = unsafe { InternedStr::new(string.into()) };

        // SAFETY:
        // A `InternedStr` created from a static reference
        // cannot be invalidated by allocations and deallocations,
        // so this is safe.
        unsafe { self.next_index(string) }
    }

    /// Returns the string for the given index if any.
    #[inline]
    pub(super) fn index(&self, index: usize) -> Option<&[Char]> {
        self.spans.get(index).map(|ptr|
            // SAFETY: We always ensure the stored `InternedStr`s always
            // reference memory inside `head` and `full`
            unsafe {ptr.as_ref()})
    }

    /// Inserts a new string pointer into `spans` and returns its index.
    ///
    /// # Safety
    ///
    /// The caller must ensure `string` points to a valid
    /// memory inside `head` (or only valid in the case of statics)
    /// and that it won't be invalidated by allocations and deallocations.
    unsafe fn next_index(&mut self, string: InternedStr<Char>) -> usize {
        let next = self.len();
        self.spans.push(string);
        self.symbol_cache.insert(string, next);
        next
    }
}

impl<Char> RawInterner<Char>
where
    Char: Hash + Eq + Clone,
{
    /// Interns the given string.
    ///
    /// Returns the index of `string` within the interner.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    pub(super) fn intern(&mut self, string: &[Char]) -> usize {
        // SAFETY:
        //
        // Firstly, this interner works on the assumption that the allocated
        // memory by `head` won't ever be moved from its position on the heap,
        // which is an important point to understand why manipulating it like
        // this is safe.
        //
        // `String` (which is simply a `Vec<u8>` with additional invariants)
        // is essentially a pointer to heap memory that can be moved without
        // any problems, since copying a pointer cannot invalidate the memory
        // that it points to.
        //
        // However, `String` CAN be invalidated when pushing, extending or
        // shrinking it, since all those operations reallocate on the heap.
        //
        // To prevent that, we HAVE to ensure the capacity will succeed without
        // having to reallocate, and the only way to do that without invalidating
        // any other alive `InternedStr` is to create a brand new `head` with
        // enough capacity and push the old `head` to `full` to keep it alive
        // throughout the lifetime of the whole interner.
        //
        // `FixedString` encapsulates this by only allowing checked `push`es
        // to the internal string, but we still have to ensure the memory
        // of `head` is not deallocated until the whole interner deallocates,
        // which we can do by moving it inside the interner itself, specifically
        // on the `full` vector, where every other old `head` also lives.
        let interned_str = unsafe {
            self.head.push(string).unwrap_or_else(|| {
                let new_cap =
                    (usize::max(self.head.capacity(), string.len()) + 1).next_power_of_two();
                let new_head = FixedString::new(new_cap);
                let old_head = std::mem::replace(&mut self.head, new_head);

                // If the user creates an `Interner`
                // with `Interner::with_capacity(BIG_NUMBER)` and
                // the first interned string's length is bigger than `BIG_NUMBER`,
                // `self.full.push(old_head)` would push a big, empty string of
                // allocated size `BIG_NUMBER` into `full`.
                // This prevents that case.
                if !old_head.is_empty() {
                    self.full.push(old_head);
                }
                self.head.push_unchecked(string)
            })
        };

        // SAFETY: We are obtaining a pointer to the internal memory of
        // `head`, which is alive through the whole life of the interner, so
        // this is safe.
        unsafe { self.next_index(interned_str) }
    }
}
