use super::{VaryingOperand, VaryingOperandVariant};
use thin_vec::ThinVec;

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
unsafe impl Readable for (u32, f32) {}
unsafe impl Readable for (u8, u8, u8) {}
unsafe impl Readable for (u16, u16, u16) {}
unsafe impl Readable for (u32, u32, u32) {}
unsafe impl Readable for (u32, u64, u8) {}
unsafe impl Readable for (u32, u64, u16) {}
unsafe impl Readable for (u32, u64, u32) {}
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

/// The opcode argument formats of the vm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Format {
    U8,
    U16,
    U32,
}

impl From<Format> for u8 {
    fn from(value: Format) -> Self {
        value as u8
    }
}

impl From<u8> for Format {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::U16,
            2 => Self::U32,
            _ => Self::U8,
        }
    }
}

pub(crate) trait Argument: Sized + std::fmt::Debug {
    /// Encode the argument into a byte slice
    fn encode(self, bytes: &mut Vec<u8>);

    /// Decode the argument from a byte slice
    /// Returns the decoded argument and the new position after reading
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize);
}

fn write_format(bytes: &mut Vec<u8>, value: Format) {
    bytes.push(value.into());
}

fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn write_i8(bytes: &mut Vec<u8>, value: i8) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn write_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

impl Argument for () {
    fn encode(self, _: &mut Vec<u8>) {}

    fn decode(_: &[u8], pos: usize) -> (Self, usize) {
        ((), pos)
    }
}

impl Argument for VaryingOperand {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                let (arg1, pos) = read::<u8>(bytes, pos);
                (arg1.into(), pos)
            }
            Format::U16 => {
                let (arg1, pos) = read::<u16>(bytes, pos);
                (arg1.into(), pos)
            }
            Format::U32 => {
                let (arg1, pos) = read::<u32>(bytes, pos);
                (arg1.into(), pos)
            }
        }
    }
}

impl Argument for (VaryingOperand, i8) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.0.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
                write_i8(bytes, self.1);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
                write_i8(bytes, self.1);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
                write_i8(bytes, self.1);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                let ((arg1, arg2), pos) = read::<(u8, i8)>(bytes, pos);
                ((arg1.into(), arg2), pos)
            }
            Format::U16 => {
                assert!(bytes.len() >= pos + 3, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u16>(bytes, pos),
                        read_unchecked::<i8>(bytes, pos + 2),
                    )
                };
                ((arg1.into(), arg2), pos + 3)
            }
            Format::U32 => {
                assert!(bytes.len() >= pos + 5, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<i8>(bytes, pos + 4),
                    )
                };
                ((arg1.into(), arg2), pos + 5)
            }
        }
    }
}

impl Argument for (VaryingOperand, i16) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.0.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
                write_i16(bytes, self.1);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
                write_i16(bytes, self.1);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
                write_i16(bytes, self.1);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= pos + 3, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u8>(bytes, pos),
                        read_unchecked::<i16>(bytes, pos + 1),
                    )
                };
                ((arg1.into(), arg2), pos + 3)
            }
            Format::U16 => {
                let ((arg1, arg2), pos) = read::<(u16, i16)>(bytes, pos);
                ((arg1.into(), arg2), pos)
            }
            Format::U32 => {
                assert!(bytes.len() >= pos + 6, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<i16>(bytes, pos + 4),
                    )
                };
                ((arg1.into(), arg2), pos + 6)
            }
        }
    }
}

impl Argument for (VaryingOperand, i32) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.0.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
                write_i32(bytes, self.1);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
                write_i32(bytes, self.1);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
                write_i32(bytes, self.1);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= pos + 5, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u8>(bytes, pos),
                        read_unchecked::<i32>(bytes, pos + 1),
                    )
                };
                ((arg1.into(), arg2), pos + 5)
            }
            Format::U16 => {
                assert!(bytes.len() >= pos + 6, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u16>(bytes, pos),
                        read_unchecked::<i32>(bytes, pos + 2),
                    )
                };
                ((arg1.into(), arg2), pos + 6)
            }
            Format::U32 => {
                let ((arg1, arg2), pos) = read::<(u32, i32)>(bytes, pos);
                ((arg1.into(), arg2), pos)
            }
        }
    }
}

impl Argument for (VaryingOperand, f32) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.0.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
                write_f32(bytes, self.1);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
                write_f32(bytes, self.1);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
                write_f32(bytes, self.1);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= pos + 5, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u8>(bytes, pos),
                        read_unchecked::<f32>(bytes, pos + 1),
                    )
                };
                ((arg1.into(), arg2), pos + 5)
            }
            Format::U16 => {
                assert!(bytes.len() >= pos + 6, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u16>(bytes, pos),
                        read_unchecked::<f32>(bytes, pos + 2),
                    )
                };
                ((arg1.into(), arg2), pos + 6)
            }
            Format::U32 => {
                let ((arg1, arg2), pos) = read::<(u32, f32)>(bytes, pos);
                ((arg1.into(), arg2), pos + 8)
            }
        }
    }
}

impl Argument for (VaryingOperand, f64) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.0.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, value);
                write_f64(bytes, self.1);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, value);
                write_f64(bytes, self.1);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, value);
                write_f64(bytes, self.1);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= pos + 9, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u8>(bytes, pos),
                        read_unchecked::<f64>(bytes, pos + 1),
                    )
                };
                ((arg1.into(), arg2), pos + 9)
            }
            Format::U16 => {
                assert!(
                    bytes.len() >= pos + 10,
                    "buffer too small to read arguments"
                );
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u16>(bytes, pos),
                        read_unchecked::<f64>(bytes, pos + 2),
                    )
                };
                ((arg1.into(), arg2), pos + 10)
            }
            Format::U32 => {
                assert!(
                    bytes.len() >= pos + 12,
                    "buffer too small to read arguments"
                );
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<f64>(bytes, pos + 4),
                    )
                };
                ((arg1.into(), arg2), pos + 12)
            }
        }
    }
}

impl Argument for (VaryingOperand, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match (self.0.variant(), self.1.variant()) {
            (VaryingOperandVariant::U8(lhs), VaryingOperandVariant::U8(rhs)) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, lhs);
                write_u8(bytes, rhs);
            }
            (VaryingOperandVariant::U8(lhs), VaryingOperandVariant::U16(rhs)) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, lhs.into());
                write_u16(bytes, rhs);
            }
            (VaryingOperandVariant::U16(lhs), VaryingOperandVariant::U8(rhs)) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, lhs);
                write_u16(bytes, rhs.into());
            }
            (VaryingOperandVariant::U16(lhs), VaryingOperandVariant::U16(rhs)) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, lhs);
                write_u16(bytes, rhs);
            }
            _ => {
                write_format(bytes, Format::U32);
                write_u32(bytes, self.0.value);
                write_u32(bytes, self.1.value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                let ((arg1, arg2), pos) = read::<(u8, u8)>(bytes, pos);
                ((arg1.into(), arg2.into()), pos)
            }
            Format::U16 => {
                let ((arg1, arg2), pos) = read::<(u16, u16)>(bytes, pos);
                ((arg1.into(), arg2.into()), pos)
            }
            Format::U32 => {
                let ((arg1, arg2), pos) = read::<(u32, u32)>(bytes, pos);
                ((arg1.into(), arg2.into()), pos)
            }
        }
    }
}

impl Argument for (VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match (self.0.variant(), self.1.variant(), self.2.variant()) {
            (
                VaryingOperandVariant::U8(lhs),
                VaryingOperandVariant::U8(mid),
                VaryingOperandVariant::U8(rhs),
            ) => {
                write_format(bytes, Format::U8);
                write_u8(bytes, lhs);
                write_u8(bytes, mid);
                write_u8(bytes, rhs);
            }
            (
                VaryingOperandVariant::U8(lhs),
                VaryingOperandVariant::U8(mid),
                VaryingOperandVariant::U16(rhs),
            ) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, lhs.into());
                write_u16(bytes, mid.into());
                write_u16(bytes, rhs);
            }
            (
                VaryingOperandVariant::U16(lhs),
                VaryingOperandVariant::U16(mid),
                VaryingOperandVariant::U16(rhs),
            ) => {
                write_format(bytes, Format::U16);
                write_u16(bytes, lhs);
                write_u16(bytes, mid);
                write_u16(bytes, rhs);
            }
            _ => {
                write_format(bytes, Format::U32);
                write_u32(bytes, self.0.value);
                write_u32(bytes, self.1.value);
                write_u32(bytes, self.2.value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                let ((arg1, arg2, arg3), pos) = read::<(u8, u8, u8)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into()), pos)
            }
            Format::U16 => {
                let ((arg1, arg2, arg3), pos) = read::<(u16, u16, u16)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into()), pos)
            }
            Format::U32 => {
                let ((arg1, arg2, arg3), pos) = read::<(u32, u32, u32)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into()), pos)
            }
        }
    }
}

impl Argument
    for (
        VaryingOperand,
        VaryingOperand,
        VaryingOperand,
        VaryingOperand,
    )
{
    fn encode(self, bytes: &mut Vec<u8>) {
        let format = match (
            self.0.variant(),
            self.1.variant(),
            self.2.variant(),
            self.3.variant(),
        ) {
            (
                VaryingOperandVariant::U8(_),
                VaryingOperandVariant::U8(_),
                VaryingOperandVariant::U8(_),
                VaryingOperandVariant::U8(_),
            ) => Format::U8,
            (VaryingOperandVariant::U16(_), _, _, _)
            | (_, VaryingOperandVariant::U16(_), _, _)
            | (_, _, VaryingOperandVariant::U16(_), _)
            | (_, _, _, VaryingOperandVariant::U16(_))
                if !matches!(self.0.variant(), VaryingOperandVariant::U32(_))
                    && !matches!(self.1.variant(), VaryingOperandVariant::U32(_))
                    && !matches!(self.2.variant(), VaryingOperandVariant::U32(_))
                    && !matches!(self.3.variant(), VaryingOperandVariant::U32(_)) =>
            {
                Format::U16
            }
            _ => Format::U32,
        };

        write_format(bytes, format);

        match format {
            Format::U8 => {
                if let (
                    VaryingOperandVariant::U8(v1),
                    VaryingOperandVariant::U8(v2),
                    VaryingOperandVariant::U8(v3),
                    VaryingOperandVariant::U8(v4),
                ) = (
                    self.0.variant(),
                    self.1.variant(),
                    self.2.variant(),
                    self.3.variant(),
                ) {
                    write_u8(bytes, v1);
                    write_u8(bytes, v2);
                    write_u8(bytes, v3);
                    write_u8(bytes, v4);
                }
            }
            Format::U16 => {
                write_u16(bytes, self.0.value as u16);
                write_u16(bytes, self.1.value as u16);
                write_u16(bytes, self.2.value as u16);
                write_u16(bytes, self.3.value as u16);
            }
            Format::U32 => {
                write_u32(bytes, self.0.value);
                write_u32(bytes, self.1.value);
                write_u32(bytes, self.2.value);
                write_u32(bytes, self.3.value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                let ((arg1, arg2, arg3, arg4), pos) = read::<(u8, u8, u8, u8)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into(), arg4.into()), pos)
            }
            Format::U16 => {
                let ((arg1, arg2, arg3, arg4), pos) = read::<(u16, u16, u16, u16)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into(), arg4.into()), pos)
            }
            Format::U32 => {
                let ((arg1, arg2, arg3, arg4), pos) = read::<(u32, u32, u32, u32)>(bytes, pos);
                ((arg1.into(), arg2.into(), arg3.into(), arg4.into()), pos)
            }
        }
    }
}

impl Argument for u32 {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        read::<u32>(bytes, pos)
    }
}

impl Argument for (u32, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.1.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u32(bytes, self.0);
                write_u8(bytes, value);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u32(bytes, self.0);
                write_u16(bytes, value);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, self.0);
                write_u32(bytes, value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= pos + 5, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<u8>(bytes, pos + 4),
                    )
                };
                ((arg1.into(), arg2.into()), pos + 5)
            }
            Format::U16 => {
                assert!(bytes.len() >= pos + 6, "buffer too small to read arguments");
                let (arg1, arg2) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<u16>(bytes, pos + 4),
                    )
                };
                ((arg1.into(), arg2.into()), pos + 6)
            }
            Format::U32 => {
                let ((arg1, arg2), pos) = read::<(u32, u32)>(bytes, pos);
                ((arg1.into(), arg2.into()), pos)
            }
        }
    }
}

impl Argument for (u32, VaryingOperand, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        let format = match (self.1.variant(), self.2.variant()) {
            (VaryingOperandVariant::U8(_), VaryingOperandVariant::U8(_)) => Format::U8,
            (VaryingOperandVariant::U16(_), _) | (_, VaryingOperandVariant::U16(_))
                if !matches!(self.1.variant(), VaryingOperandVariant::U32(_))
                    && !matches!(self.2.variant(), VaryingOperandVariant::U32(_)) =>
            {
                Format::U16
            }
            _ => Format::U32,
        };

        write_format(bytes, format);

        match format {
            Format::U8 => {
                write_u32(bytes, self.0);
                write_u8(bytes, self.1.value as u8);
                write_u8(bytes, self.2.value as u8);
            }
            Format::U16 => {
                write_u32(bytes, self.0);
                write_u16(bytes, self.1.value as u16);
                write_u16(bytes, self.2.value as u16);
            }
            Format::U32 => {
                write_u32(bytes, self.0);
                write_u32(bytes, self.1.value);
                write_u32(bytes, self.2.value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;

        match format {
            Format::U8 => {
                assert!(bytes.len() >= 6, "buffer too small to read arguments");
                let (arg1, arg2, arg3) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<u8>(bytes, pos + 4),
                        read_unchecked::<u8>(bytes, pos + 5),
                    )
                };
                ((arg1, arg2.into(), arg3.into()), pos + 6)
            }
            Format::U16 => {
                assert!(bytes.len() >= 6, "buffer too small to read arguments");
                let (arg1, arg2, arg3) = unsafe {
                    (
                        read_unchecked::<u32>(bytes, pos),
                        read_unchecked::<u16>(bytes, pos + 4),
                        read_unchecked::<u16>(bytes, pos + 6),
                    )
                };
                ((arg1, arg2.into(), arg3.into()), pos + 8)
            }
            Format::U32 => {
                let ((arg1, arg2, arg3), pos) = read::<(u32, u32, u32)>(bytes, pos);
                ((arg1, arg2.into(), arg3.into()), pos)
            }
        }
    }
}

impl Argument for (VaryingOperand, ThinVec<VaryingOperand>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        let variants: Vec<VaryingOperandVariant> = self.1.iter().map(|op| op.variant()).collect();
        let has_u32 = variants
            .iter()
            .any(|variant| matches!(variant, VaryingOperandVariant::U32(_)));
        let has_u16 = variants
            .iter()
            .any(|variant| matches!(variant, VaryingOperandVariant::U16(_)));

        let fmt0 = match self.0.variant() {
            VaryingOperandVariant::U8(_) => Format::U8,
            VaryingOperandVariant::U16(_) => Format::U16,
            VaryingOperandVariant::U32(_) => Format::U32,
        };

        let format = if has_u32 || fmt0 == Format::U32 {
            Format::U32
        } else if has_u16 || fmt0 == Format::U16 {
            Format::U16
        } else {
            Format::U8
        };

        let total_len = self.1.len();

        match format {
            Format::U8 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u8(bytes, self.0.value as u8);
                for arg in self.1 {
                    write_u8(bytes, arg.value as u8);
                }
            }
            Format::U16 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u16(bytes, self.0.value as u16);
                for arg in self.1 {
                    write_u16(bytes, arg.value as u16);
                }
            }
            Format::U32 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u32(bytes, self.0.value);
                for arg in self.1 {
                    write_u32(bytes, arg.value);
                }
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;
        let pos = pos + 2;

        match format {
            Format::U8 => {
                let fst = read::<u8>(bytes, pos).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u8>(bytes, pos + 1 + i);
                    rest.push(val.0.into())
                }
                ((fst.into(), rest), pos + 1 + total_len)
            }
            Format::U16 => {
                let fst = read::<u16>(bytes, pos).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u16>(bytes, pos + 2 + i * 2);
                    rest.push(val.0.into())
                }
                ((fst.into(), rest), pos + 2 + total_len * 2)
            }
            Format::U32 => {
                let fst = read::<u32>(bytes, pos).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u32>(bytes, pos + 4 + i * 4);
                    rest.push(val.0.into())
                }
                ((fst.into(), rest), pos + 4 + total_len * 4)
            }
        }
    }
}

impl Argument for (VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        let variants: Vec<VaryingOperandVariant> = self.2.iter().map(|op| op.variant()).collect();
        let has_u32 = variants
            .iter()
            .any(|variant| matches!(variant, VaryingOperandVariant::U32(_)));
        let has_u16 = variants
            .iter()
            .any(|variant| matches!(variant, VaryingOperandVariant::U16(_)));

        let fmt0 = match self.0.variant() {
            VaryingOperandVariant::U8(_) => Format::U8,
            VaryingOperandVariant::U16(_) => Format::U16,
            VaryingOperandVariant::U32(_) => Format::U32,
        };

        let fmt1 = match self.1.variant() {
            VaryingOperandVariant::U8(_) => Format::U8,
            VaryingOperandVariant::U16(_) => Format::U16,
            VaryingOperandVariant::U32(_) => Format::U32,
        };

        let format = if has_u32 || fmt0 == Format::U32 || fmt1 == Format::U32 {
            Format::U32
        } else if has_u16 || fmt0 == Format::U16 || fmt1 == Format::U16 {
            Format::U16
        } else {
            Format::U8
        };

        let total_len = self.2.len();

        match format {
            Format::U8 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u8(bytes, self.0.value as u8);
                write_u8(bytes, self.1.value as u8);
                for arg in self.2 {
                    write_u8(bytes, arg.value as u8);
                }
            }
            Format::U16 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u16(bytes, self.0.value as u16);
                write_u16(bytes, self.1.value as u16);
                for arg in self.2 {
                    write_u16(bytes, arg.value as u16);
                }
            }
            Format::U32 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u32(bytes, self.0.value);
                write_u32(bytes, self.1.value);
                for arg in self.2 {
                    write_u32(bytes, arg.value);
                }
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;
        let pos = pos + 2;

        match format {
            Format::U8 => {
                let fst = read::<u8>(bytes, pos).0;
                let snd = read::<u8>(bytes, pos + 1).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u8>(bytes, pos + 2 + i);
                    rest.push(val.0.into())
                }
                ((fst.into(), snd.into(), rest), pos + 2 + total_len)
            }
            Format::U16 => {
                let fst = read::<u16>(bytes, pos).0;
                let snd = read::<u16>(bytes, pos + 2).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u16>(bytes, pos + 4 + i * 2);
                    rest.push(val.0.into())
                }
                ((fst.into(), snd.into(), rest), pos + 4 + total_len * 2)
            }
            Format::U32 => {
                let fst = read::<u32>(bytes, pos).0;
                let snd = read::<u32>(bytes, pos + 4).0;
                let mut rest: ThinVec<VaryingOperand> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u32>(bytes, pos + 8 + i * 4);
                    rest.push(val.0.into())
                }
                ((fst.into(), snd.into(), rest), pos + 8 + total_len * 4)
            }
        }
    }
}

impl Argument for (u32, u64, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        match self.2.variant() {
            VaryingOperandVariant::U8(value) => {
                write_format(bytes, Format::U8);
                write_u32(bytes, self.0);
                write_u64(bytes, self.1);
                write_u8(bytes, value);
            }
            VaryingOperandVariant::U16(value) => {
                write_format(bytes, Format::U16);
                write_u32(bytes, self.0);
                write_u64(bytes, self.1);
                write_u16(bytes, value);
            }
            VaryingOperandVariant::U32(value) => {
                write_format(bytes, Format::U32);
                write_u32(bytes, self.0);
                write_u64(bytes, self.1);
                write_u32(bytes, value);
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;
        assert!(
            bytes.len() >= pos + 16,
            "buffer too small to read arguments"
        );
        match format {
            Format::U8 => {
                let ((arg1, arg2, arg3), pos) = read::<(u32, u64, u8)>(bytes, pos);
                ((arg1, arg2, arg3.into()), pos)
            }
            Format::U16 => {
                let ((arg1, arg2, arg3), pos) = read::<(u32, u64, u16)>(bytes, pos);
                ((arg1, arg2, arg3.into()), pos)
            }
            Format::U32 => {
                let ((arg1, arg2, arg3), pos) = read::<(u32, u64, u32)>(bytes, pos);
                ((arg1, arg2, arg3.into()), pos)
            }
        }
    }
}

impl Argument for (u32, u32, VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
        write_u32(bytes, self.1);
        write_u32(bytes, self.2.value);
        write_u32(bytes, self.3.value);
        write_u32(bytes, self.4.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let ((arg1, arg2, arg3, arg4, arg5), pos) = read::<(u32, u32, u32, u32, u32)>(bytes, pos);
        ((arg1, arg2, arg3.into(), arg4.into(), arg5.into()), pos)
    }
}

impl Argument for (u32, ThinVec<u32>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write length
        let total_len = self.1.len();
        write_u16(bytes, total_len as u16);

        // Write first argument
        write_u32(bytes, self.0);

        // Write remaining arguments
        for arg in &self.1 {
            write_u32(bytes, *arg);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the length
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;

        assert!(
            bytes.len() >= pos + 6 + total_len * 4,
            "buffer too small to read arguments"
        );

        // Read the first argument
        let first = unsafe { read_unchecked::<u32>(bytes, pos + 2) };

        // Read remaining arguments
        let mut rest = ThinVec::with_capacity(total_len);
        for i in 0..total_len {
            let value = unsafe { read_unchecked::<u32>(bytes, pos + 6 + i * 4) };
            rest.push(value);
        }

        ((first, rest), pos + 6 + total_len * 4)
    }
}

impl Argument for (u64, VaryingOperand, ThinVec<u32>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write length

        let format = match self.1.variant() {
            VaryingOperandVariant::U8(_) => Format::U8,
            VaryingOperandVariant::U16(_) => Format::U16,
            VaryingOperandVariant::U32(_) => Format::U32,
        };

        let total_len = self.2.len();

        match format {
            Format::U8 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u64(bytes, self.0);
                write_u8(bytes, self.1.value as u8);
                for arg in &self.2 {
                    write_u32(bytes, *arg);
                }
            }
            Format::U16 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u64(bytes, self.0);
                write_u16(bytes, self.1.value as u16);
                for arg in &self.2 {
                    write_u32(bytes, *arg);
                }
            }
            Format::U32 => {
                write_format(bytes, format);
                write_u16(bytes, total_len as u16);
                write_u64(bytes, self.0);
                write_u32(bytes, self.1.value);
                for arg in &self.2 {
                    write_u32(bytes, *arg);
                }
            }
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the length
        let format = Format::from(bytes[pos]);
        let pos = pos + 1;
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;
        let pos = pos + 2;

        match format {
            Format::U8 => {
                let fst = read::<u64>(bytes, pos).0;
                let snd = read::<u8>(bytes, pos + 8).0;
                let mut rest: ThinVec<u32> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u32>(bytes, pos + 9 + i * 4).0;
                    rest.push(val);
                }
                ((fst, snd.into(), rest), pos + 9 + total_len * 4)
            }
            Format::U16 => {
                let fst = read::<u64>(bytes, pos).0;
                let snd = read::<u16>(bytes, pos + 8).0;
                let mut rest: ThinVec<u32> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u32>(bytes, pos + 10 + i * 4).0;
                    rest.push(val);
                }
                ((fst, snd.into(), rest), pos + 10 + total_len * 4)
            }
            Format::U32 => {
                let fst = read::<u64>(bytes, pos).0;
                let snd = read::<u32>(bytes, pos + 8).0;
                let mut rest: ThinVec<u32> = ThinVec::with_capacity(total_len);
                for i in 0..total_len {
                    let val = read::<u32>(bytes, pos + 12 + i * 4).0;
                    rest.push(val);
                }
                ((fst, snd.into(), rest), pos + 12 + total_len * 4)
            }
        }
    }
}

impl Argument for (VaryingOperand, ThinVec<u32>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write length
        let total_len = self.1.len();
        write_u16(bytes, total_len as u16);

        // Write first argument
        write_u32(bytes, self.0.value);

        // Write remaining arguments
        for arg in &self.1 {
            write_u32(bytes, *arg);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the length
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;

        assert!(
            bytes.len() >= pos + 6 + total_len * 4,
            "buffer too small to read arguments"
        );

        // Read the first argument
        let first = unsafe { read_unchecked::<u32>(bytes, pos + 2) };

        // Read remaining arguments
        let mut rest = ThinVec::with_capacity(total_len);
        for i in 0..total_len {
            let value = unsafe { read_unchecked::<u32>(bytes, pos + 6 + i * 4) };
            rest.push(value);
        }

        ((first.into(), rest), pos + 6 + total_len * 4)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::vec;
    use thin_vec::thin_vec;

    fn test_write_arg<T: Argument>(test_data: T) -> Vec<u8> {
        /* Doesn't pass by reference here because I am not sure about implementing Copy for Argument */
        let mut v: Vec<u8> = Vec::new();
        test_data.encode(&mut v);
        v
    }

    fn test_read_arg<T: Argument, B: AsRef<[u8]>>(bytecode: B) -> (T, usize) {
        let r: &[u8] = bytecode.as_ref();
        T::decode(r, 0)
    }

    //
    // VariableOperand
    //
    #[test]
    fn arg_vop_u8() {
        let arg = VaryingOperand::from(u8::MAX);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF], v);
        assert_eq!(test_read_arg::<VaryingOperand, _>(v), (arg, 2));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8() {
        test_read_arg::<VaryingOperand, _>([0]);
    }

    #[test]
    fn arg_vop_u16() {
        let arg = VaryingOperand::from(u16::MAX);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF], v);
        assert_eq!(test_read_arg::<VaryingOperand, _>(v), (arg, 3));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16() {
        test_read_arg::<VaryingOperand, _>([1]);
    }

    #[test]
    fn arg_vop_u32() {
        let arg = VaryingOperand::from(u32::MAX);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(test_read_arg::<VaryingOperand, _>(v), (arg, 5))
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32() {
        test_read_arg::<VaryingOperand, _>([2, 0xFF, 0xFF, 0xFF]);
    }

    //
    // u32
    //
    #[test]
    fn arg_u32() {
        let arg = u32::MAX;
        let v = test_write_arg(arg);
        assert_eq!(vec![0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(test_read_arg::<u32, _>(v), (arg, 4));
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32() {
        test_read_arg::<u32, _>([0xFF, 0xFF, 0xFF]);
    }

    //
    // (VariableOperarand, i8)
    //
    #[test]
    fn arg_vop_u8_i8() {
        let arg = (VaryingOperand::from(u8::MAX), i8::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i8), _>(v), (arg, 3));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_i8() {
        test_read_arg::<(VaryingOperand, i8), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_i8() {
        let arg = (VaryingOperand::from(u16::MAX), i8::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i8), _>(v), (arg, 4));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_i8() {
        test_read_arg::<(VaryingOperand, i8), _>([1, 0xFF, 0xFF]);
    }

    #[test]
    fn arg_vop_u32_i8() {
        let arg = (VaryingOperand::from(u32::MAX), i8::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i8), _>(v), (arg, 6));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_i8() {
        test_read_arg::<(VaryingOperand, i8), _>([2, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    //
    // (VariableOperarand, i16)
    //
    #[test]
    fn arg_vop_u8_i16() {
        let arg = (VaryingOperand::from(u8::MAX), i16::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i16), _>(v), (arg, 4));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_i16() {
        test_read_arg::<(VaryingOperand, i16), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_i16() {
        let arg = (VaryingOperand::from(u16::MAX), i16::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i16), _>(v), (arg, 5));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_i16() {
        test_read_arg::<(VaryingOperand, i16), _>([1, 0xFF, 0xFF]);
    }

    #[test]
    fn arg_vop_u32_i16() {
        let arg = (VaryingOperand::from(u32::MAX), i16::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i16), _>(v), (arg, 7));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_i16() {
        test_read_arg::<(VaryingOperand, i16), _>([2, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    //
    // (VariableOperarand, i32)
    //
    #[test]
    fn arg_vop_u8_i32() {
        let arg = (VaryingOperand::from(u8::MAX), i32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0x00, 0x00, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i32), _>(v), (arg, 6));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_i32() {
        test_read_arg::<(VaryingOperand, i32), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_i32() {
        let arg = (VaryingOperand::from(u16::MAX), i32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i32), _>(v), (arg, 7));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_i32() {
        test_read_arg::<(VaryingOperand, i32), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u32_i32() {
        let arg = (VaryingOperand::from(u32::MAX), i32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x80], v);
        assert_eq!(test_read_arg::<(VaryingOperand, i32), _>(v), (arg, 9));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_i32() {
        test_read_arg::<(VaryingOperand, i32), _>([2, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    //
    // (VaryingOperand, f32)
    //
    #[test]
    fn arg_vop_u8_f32() {
        let arg = (VaryingOperand::from(u8::MAX), f32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF], v);
        assert_eq!(test_read_arg::<(VaryingOperand, f32), _>(v), (arg, 6));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_f32() {
        test_read_arg::<(VaryingOperand, f32), _>([0]);
    }

    #[test]
    fn arg_vop_u16_f32() {
        let arg = (VaryingOperand::from(u16::MAX), f32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF], v);
        assert_eq!(test_read_arg::<(VaryingOperand, f32), _>(v), (arg, 7));
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_f32() {
        test_read_arg::<(VaryingOperand, f32), _>([1]);
    }

    #[test]
    fn arg_vop_u32_f32() {
        let arg = (VaryingOperand::from(u32::MAX), f32::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF], v);
        let a = test_read_arg::<(VaryingOperand, f32), _>(v);
        assert_eq!(f32::MIN, a.0 .1);
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_f32() {
        test_read_arg::<(VaryingOperand, f32), _>([2]);
    }

    //
    // (VaryingOperand, f64)
    //
    #[test]
    fn arg_vop_u8_f64() {
        let arg = (VaryingOperand::from(u8::MAX), f64::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xFF],
            v
        );
        let a = test_read_arg::<(VaryingOperand, f64), _>(v);
        assert_eq!(f64::MIN, a.0 .1);
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_f64() {
        test_read_arg::<(VaryingOperand, f64), _>([0]);
    }

    #[test]
    fn arg_vop_u16_f64() {
        let arg = (VaryingOperand::from(u16::MAX), f64::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xFF],
            v
        );
        let a = test_read_arg::<(VaryingOperand, f64), _>(v);
        assert_eq!(f64::MIN, a.0 .1);
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_f64() {
        test_read_arg::<(VaryingOperand, f64), _>([1]);
    }

    #[test]
    fn arg_vop_u32_f64() {
        let arg = (VaryingOperand::from(u32::MAX), f64::MIN);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0xFF],
            v
        );
        let a = test_read_arg::<(VaryingOperand, f64), _>(v);
        assert_eq!(f64::MIN, a.0 .1);
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_f64() {
        test_read_arg::<(VaryingOperand, f64), _>([2]);
    }

    //
    // (VaryingOperand, VaryingOperand)
    //
    #[test]
    fn arg_vop_u8_vop_u8() {
        let arg = (VaryingOperand::from(u8::MAX), VaryingOperand::from(u8::MAX));
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand), _>(v),
            (arg, 3)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_vop_u8() {
        test_read_arg::<(VaryingOperand, VaryingOperand), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u8_vop_u16() {
        let arg = (
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0x00, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand), _>(v),
            (arg, 5)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_vop_u16() {
        test_read_arg::<(VaryingOperand, VaryingOperand), _>([1, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_vop_u8() {
        let arg = (
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u8::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0x00], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand), _>(v),
            (arg, 5)
        );
    }

    #[test]
    fn arg_vop_u16_vop_u16() {
        let arg = (
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand), _>(v),
            (arg, 5)
        );
    }

    #[test]
    fn arg_vop_u32_vop_u32() {
        let arg = (
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand), _>(v),
            (arg, 9)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_vop_u32() {
        test_read_arg::<(VaryingOperand, VaryingOperand), _>([2, 0xFF]);
    }

    //
    // (VaryingOperand, VaryingOperand, VaryingOperand)
    //
    #[test]
    fn arg_vop_u8_vop_u8_vop_u8() {
        let arg = (
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>(v),
            (arg, 4)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_vop_u8_vop_u8() {
        test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u8_vop_u8_vop_u16() {
        let arg = (
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>(v),
            (arg, 7)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_vop_u8_vop_u16() {
        test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>([1, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_vop_u16_vop_u16() {
        let arg = (
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>(v),
            (arg, 7)
        );
    }

    #[test]
    fn arg_vop_u32_vop_u32_vop_u32() {
        let arg = (
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            v
        );
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>(v),
            (arg, 13)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_vop_u32_vop_u32() {
        test_read_arg::<(VaryingOperand, VaryingOperand, VaryingOperand), _>([2, 0xFF]);
    }

    //
    // (VaryingOperand, VaryingOperand, VaryingOperand, VaryingOperand)
    //
    #[test]
    fn arg_vop_u8_vop_u8_vop_u8_vop_u8() {
        let arg = (
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<
                (
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                ),
                _,
            >(v),
            (arg, 5)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u8_vop_u8_vop_u8_vop_u8() {
        test_read_arg::<
            (
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
            ),
            _,
        >([0, 0xFF]);
    }

    #[test]
    fn arg_vop_u16_vop_u16_vop_u16_vop_u16() {
        let arg = (
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<
                (
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                ),
                _,
            >(v),
            (arg, 9)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u16_vop_u16_vop_u16_vop_u16() {
        test_read_arg::<
            (
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
            ),
            _,
        >([1, 0xFF]);
    }

    #[test]
    fn arg_vop_u32_vop_u32_vop_u32_vop_u32() {
        let arg = (
            VaryingOperand::from(u16::MAX),
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![
                2, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF
            ],
            v
        );
        assert_eq!(
            test_read_arg::<
                (
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                    VaryingOperand,
                ),
                _,
            >(v),
            (arg, 17)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_u32_vop_u32_vop_u32_vop_u32() {
        test_read_arg::<
            (
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
            ),
            _,
        >([2, 0xFF]);
    }

    //
    // (u32, VaryingOperand)
    //

    #[test]
    fn arg_u32_vop_u8() {
        let arg = (u32::MAX, VaryingOperand::from(u8::MAX));
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(test_read_arg::<(u32, VaryingOperand), _>(v), (arg, 6));
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32_vop() {
        test_read_arg::<(u32, VaryingOperand), _>([0xFF]);
    }

    #[test]
    fn arg_u32_vop_u16() {
        let arg = (u32::MAX, VaryingOperand::from(u16::MAX));
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(test_read_arg::<(u32, VaryingOperand), _>(v), (arg, 7));
    }

    #[test]
    fn arg_u32_vop_u32() {
        let arg = (u32::MAX, VaryingOperand::from(u32::MAX));
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,0xFF,0xFF,0xFF], v);
        assert_eq!(test_read_arg::<(u32, VaryingOperand), _>(v), (arg, 9));
    }

    //
    // (u32, VaryingOperand, VaryingOperand)
    //
    #[test]
    fn arg_u32_vop_u8_vop_u8() {
        let arg = (
            u32::MAX,
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u8::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(u32, VaryingOperand, VaryingOperand), _>(v),
            (arg, 7)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32_vop_vop() {
        test_read_arg::<(u32, VaryingOperand, VaryingOperand), _>([0xFF]);
    }

    #[test]
    fn arg_u32_vop_u8_vop_u16() {
        let arg = (
            u32::MAX,
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u16::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF], v);
        assert_eq!(
            test_read_arg::<(u32, VaryingOperand, VaryingOperand), _>(v),
            (arg, 9)
        );
    }

    #[test]
    fn arg_u32_vop_u8_vop_u32() {
        let arg = (
            u32::MAX,
            VaryingOperand::from(u8::MAX),
            VaryingOperand::from(u32::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(vec![2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,0x00,0x00,0x00,0xFF,0xFF,0xFF,0xFF], v);
        assert_eq!(
            test_read_arg::<(u32, VaryingOperand, VaryingOperand), _>(v),
            (arg, 13)
        );
    }
    //
    // (VaryingOperand, ThinVec<VaryingOperand>)
    //
    #[test]
    fn arg_vop_thin_vec_vop() {
        let v1 = thin_vec![VaryingOperand::from(u32::MAX)];
        let arg = (VaryingOperand::from(u32::MAX), v1);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![2, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            v
        );
        assert_eq!(
            test_read_arg::<(VaryingOperand, ThinVec<VaryingOperand>), _>(v),
            (arg, 11)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_thin_vec_vop() {
        test_read_arg::<(VaryingOperand, ThinVec<VaryingOperand>), _>([0x00]);
    }

    //
    // (VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>)
    //
    #[test]
    fn arg_vop_vop_thin_vec_vop() {
        let v1 = thin_vec![VaryingOperand::from(u32::MAX)];
        let arg = (
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
            v1,
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![
                2, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF
            ],
            v
        );
        assert_eq!(
            test_read_arg::<(VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>), _>(v),
            (arg, 15)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_vop_thin_vec_vop() {
        test_read_arg::<(VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>), _>([0x00]);
    }

    //
    // (u32, u64, VaryingOperand)
    //
    #[test]
    fn arg_u32_u64_vop() {
        let arg = (u32::MAX, u64::MAX, VaryingOperand::from(u32::MAX));
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![
                2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF
            ],
            v
        );
        assert_eq!(test_read_arg::<(u32, u64, VaryingOperand), _>(v), (arg, 17));
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32_u64_vop() {
        test_read_arg::<(u32, u64, VaryingOperand), _>([0]);
    }

    //
    // (u32, u64, VaryingOperand, VaryingOperand, VaryingOperaand)
    //
    #[test]
    fn arg_u32_u32_vop_vop_vop() {
        let arg = (
            u32::MAX,
            u32::MAX,
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
            VaryingOperand::from(u32::MAX),
        );
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
            ],
            v
        );
        assert_eq!(
            test_read_arg::<(u32, u32, VaryingOperand, VaryingOperand, VaryingOperand), _>(v),
            (arg, 20)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32_u64_vop_vop_vop() {
        test_read_arg::<(u32, u32, VaryingOperand, VaryingOperand, VaryingOperand), _>([0]);
    }

    //
    // (u32, ThinVec<u32>)
    //
    #[test]
    fn arg_u32_thin_vec_u32() {
        let v1 = thin_vec![u32::MAX, u32::MAX];
        let arg = (u32::MAX, v1);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![2, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            v
        );
        assert_eq!(
            test_read_arg::<(u32, ThinVec<u32>), _>([
                2, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ]),
            (arg, 14)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_u32_thin_vec_u32() {
        test_read_arg::<(u32, ThinVec<u32>), _>([0xFF]);
    }

    //
    // (u64,VaryingOperand,ThinVec<u32>)
    //
    #[test]
    fn arg_u64_vop_thin_vec_u32() {
        let v1 = thin_vec![u32::MAX];
        let arg = (u64::MAX, VaryingOperand::from(u32::MAX), v1);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![
                2, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF
            ],
            v
        );
        assert_eq!(
            test_read_arg::<(u64, VaryingOperand, ThinVec<u32>), _>(v),
            (arg, 19)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_u64_vop_thin_vec_u32() {
        test_read_arg::<(u64, VaryingOperand, ThinVec<u32>), _>([0xFF]);
    }

    //
    // (VaryingOperand, ThinVec<u32>)
    //
    #[test]
    fn arg_vop_thin_vec_u32() {
        let v1 = thin_vec![u32::MAX];
        let arg = (VaryingOperand::from(u32::MAX), v1);
        let v = test_write_arg(arg.clone());
        assert_eq!(
            vec![1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            v
        );
        assert_eq!(
            test_read_arg::<(VaryingOperand, ThinVec<u32>), _>(v),
            (arg, 10)
        );
    }

    #[test]
    #[should_panic]
    fn fail_arg_vop_thin_vec_u32() {
        test_read_arg::<(VaryingOperand, ThinVec<u32>), _>([0xFF]);
    }
}
