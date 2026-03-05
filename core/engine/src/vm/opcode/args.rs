use thin_vec::ThinVec;

use super::{Address, RegisterOperand, VaryingOperand};

/// A trait for types that can be read from a byte slice.
///
/// # Safety
///
/// - The implementor must ensure that the type can be safely read from a byte slice.
pub(super) unsafe trait Readable: Copy + Sized {}

unsafe impl Readable for u8 {}
unsafe impl Readable for i8 {}
unsafe impl Readable for u16 {}
unsafe impl Readable for i16 {}
unsafe impl Readable for u32 {}
unsafe impl Readable for i32 {}
unsafe impl Readable for u64 {}
unsafe impl Readable for f32 {}
unsafe impl Readable for f64 {}
unsafe impl Readable for (u8, u8) {}
unsafe impl Readable for (u8, i8) {}
unsafe impl Readable for (u16, u16) {}
unsafe impl Readable for (u16, i16) {}
unsafe impl Readable for (u32, u32) {}
unsafe impl Readable for (u32, i32) {}
unsafe impl Readable for (u8, u8, u8) {}
unsafe impl Readable for (u16, u16, u16) {}
unsafe impl Readable for (u32, u32, u32) {}
unsafe impl Readable for (u8, u8, u8, u8) {}
unsafe impl Readable for (u16, u16, u16, u16) {}
unsafe impl Readable for (u32, u32, u32, u32) {}
unsafe impl Readable for (u32, u32, u32, u32, u32) {}

#[inline(always)]
#[track_caller]
/// Read a value of type T from the byte slice at the given offset.
pub(super) fn read<T: Readable>(bytes: &[u8], offset: usize) -> (T, usize) {
    let new_offset = offset + size_of::<T>();

    assert!(bytes.len() >= new_offset, "buffer too small to read type T");

    // Safety: The assertion above ensures that the slice is large enough to read T.
    let result = unsafe { read_unchecked(bytes, offset) };

    (result, new_offset)
}

#[inline(always)]
#[track_caller]
/// Read a value of type T from the byte slice at the given offset.
///
/// # Safety
///
/// - The caller must ensure that the byte slice is large enough to contain a value of type T at the given offset.
unsafe fn read_unchecked<T: Readable>(bytes: &[u8], offset: usize) -> T {
    unsafe { bytes.as_ptr().add(offset).cast::<T>().read_unaligned() }
}

pub(crate) trait Argument: Sized + std::fmt::Debug {
    /// Encode the argument into a byte slice
    fn encode(self, bytes: &mut Vec<u8>);

    /// Decode the argument from a byte slice
    /// Returns the decoded argument and the new position after reading
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize);
}

#[inline(always)]
fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i8(bytes: &mut Vec<u8>, value: i8) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn write_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

impl<T: Argument> Argument for ThinVec<T> {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.len() as u32);
        for arg in self {
            arg.encode(bytes);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (len, mut pos) = read::<u32>(bytes, pos);
        let total_len = len as usize;
        let mut result = ThinVec::with_capacity(total_len);
        for _ in 0..total_len {
            let (arg, new_pos) = T::decode(bytes, pos);
            result.push(arg);
            pos = new_pos;
        }
        (result, pos)
    }
}

impl Argument for () {
    fn encode(self, _: &mut Vec<u8>) {}

    fn decode(_: &[u8], pos: usize) -> (Self, usize) {
        ((), pos)
    }
}

impl Argument for VaryingOperand {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = read::<u32>(bytes, pos);
        (arg1.into(), pos)
    }
}

impl Argument for RegisterOperand {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = read::<u32>(bytes, pos);
        (Self::new(arg1), pos)
    }
}

impl Argument for Address {
    #[inline(always)]
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.value);
    }

    #[inline(always)]
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (value, pos) = read::<u32>(bytes, pos);
        (Self::new(value), pos)
    }
}

macro_rules! impl_argument_for_tuple {
    ($( $i: ident  $t: ident ),*) => {
        impl<$( $t: Argument, )*> Argument for ($( $t, )*) {
            #[inline(always)]
            fn encode(self, bytes: &mut Vec<u8>) {
                let ($($i, )*) = self;
                $( $i.encode(bytes); )*
            }

            #[inline(always)]
            fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
                $( let ($i, pos) = $t::decode(bytes, pos); )*
                (($($i,)*), pos)
            }
        }
    };
}

impl_argument_for_tuple!(a A);
impl_argument_for_tuple!(a A, b B);
impl_argument_for_tuple!(a A, b B, c C);
impl_argument_for_tuple!(a A, b B, c C, d D);
impl_argument_for_tuple!(a A, b B, c C, d D, e E);

macro_rules! impl_argument_for_int {
    ($( $t: ty )*) => {
        $(
        impl Argument for $t {
            #[inline(always)]
            fn encode(self, bytes: &mut Vec<u8>) {
                paste::paste! {
                    [<write_ $t>](bytes, self);
                }
            }

            #[inline(always)]
            fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
                read::<$t>(bytes, pos)
            }
        }
        )*
    };
}

impl_argument_for_int!(u8 u16 u32 u64 i8 i16 i32 f32 f64);
