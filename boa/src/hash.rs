//! Fast, non-cryptographic hash used by rustc and Firefox.
//!
//! This is a modified copy of the `rustc-hash` crate [`FxHasher`] implementation.

use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;
use std::mem::size_of;
use std::ops::BitXor;

/// Type alias for a hashmap using the `fx` hash algorithm.
pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// Type alias for a hashmap using the `fx` hash algorithm.
pub type FxHashSet<V> = HashSet<V, BuildHasherDefault<FxHasher>>;

/// A speedy hash algorithm for use within rustc. The hashmap in liballoc
/// by default uses SipHash which isn't quite as speedy as we want. In the
/// compiler we're not really worried about DOS attempts, so we use a fast
/// non-cryptographic hash.
///
/// This is the same as the algorithm used by Firefox -- which is a homespun
/// one not based on any widely-known algorithm -- though modified to produce
/// 64-bit hash values instead of 32-bit hash values. It consistently
/// out-performs an FNV-based hash within rustc itself -- the collision rate is
/// similar or slightly worse than FNV, but the speed of the hash function
/// itself is much higher because it works on up to 8 bytes at a time.
#[derive(Debug, Clone, Copy)]
pub struct FxHasher {
    hash: usize,
}

#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: usize = 0x517cc1b727220a95;

impl Default for FxHasher {
    #[inline]
    fn default() -> FxHasher {
        debug_assert!(size_of::<usize>() <= 8);

        FxHasher { hash: 0 }
    }
}

impl FxHasher {
    #[inline]
    fn add_to_hash(&mut self, i: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        let mut hash = FxHasher { hash: self.hash };
        unsafe {
            while bytes.len() >= size_of::<usize>() {
                hash.add_to_hash(bytes.as_ptr().cast::<usize>().read_unaligned());
                bytes = bytes.get_unchecked(size_of::<usize>()..);
            }

            #[cfg(target_pointer_width = "64")]
            if bytes.len() >= 4 {
                hash.add_to_hash(bytes.as_ptr().cast::<u32>().read_unaligned() as usize);
                bytes = bytes.get_unchecked(size_of::<u32>()..);
            }

            if bytes.len() >= 2 {
                hash.add_to_hash(bytes.as_ptr().cast::<u16>().read_unaligned() as usize);
                bytes = bytes.get_unchecked(size_of::<u16>()..);
            }
            if !bytes.is_empty() {
                hash.add_to_hash(*bytes.as_ptr() as usize);
            }
        }
        self.hash = hash.hash;
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as usize);
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
        self.add_to_hash((i >> 32) as usize);
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash as u64
    }
}
