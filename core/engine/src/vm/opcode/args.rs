use thin_vec::ThinVec;

use super::{VaryingOperand, VaryingOperandVariant};

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
unsafe impl Readable for u64 {}
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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        write_u32(bytes, self.0.value);
        write_i32(bytes, self.1);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let ((arg1, arg2), pos) = read::<(u32, i32)>(bytes, pos);
        ((arg1.into(), arg2), pos)
    }
}

impl Argument for (VaryingOperand, f32) {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0.value);
        write_f32(bytes, self.1);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let ((arg1, arg2), pos) = read::<(u32, u32)>(bytes, pos);
        ((arg1.into(), f32::from_bits(arg2)), pos)
    }
}

impl Argument for (VaryingOperand, f64) {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0.value);
        write_f64(bytes, self.1);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        let (format_byte, pos) = read::<u8>(bytes, pos);
        let format = Format::from(format_byte);

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
        write_u32(bytes, self.0);
        write_u32(bytes, self.1.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let ((arg1, arg2), pos) = read::<(u32, u32)>(bytes, pos);
        ((arg1, arg2.into()), pos)
    }
}

impl Argument for (u32, VaryingOperand, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
        write_u32(bytes, self.1.value);
        write_u32(bytes, self.2.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let ((arg1, arg2, arg3), pos) = read::<(u32, u32, u32)>(bytes, pos);
        ((arg1, arg2.into(), arg3.into()), pos)
    }
}

impl Argument for (VaryingOperand, ThinVec<VaryingOperand>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write length of all arguments
        let total_len = self.1.len();
        write_u16(bytes, total_len as u16);

        // Write first argument
        write_u32(bytes, self.0.value);

        // Write remaining arguments
        for arg in self.1 {
            write_u32(bytes, arg.value);
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
            rest.push(value.into());
        }

        ((first.into(), rest), pos + 6 + total_len * 4)
    }
}

impl Argument for (VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write length of all arguments
        let total_len = self.2.len();
        write_u16(bytes, total_len as u16);

        // Write first two arguments
        write_u32(bytes, self.0.value);
        write_u32(bytes, self.1.value);

        // Write remaining arguments
        for arg in self.2 {
            write_u32(bytes, arg.value);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the length
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;

        assert!(
            bytes.len() >= pos + 10 + total_len * 4,
            "buffer too small to read arguments"
        );

        // Read the first two arguments
        let (first, second) = unsafe { read_unchecked::<(u32, u32)>(bytes, pos + 2) };

        // Read remaining arguments
        let mut rest = ThinVec::with_capacity(total_len);
        for i in 0..total_len {
            let value = unsafe { read_unchecked::<u32>(bytes, pos + 10 + i * 4) };
            rest.push(value.into());
        }

        (
            (first.into(), second.into(), rest),
            pos + 10 + total_len * 4,
        )
    }
}

impl Argument for (u32, u64, VaryingOperand) {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
        write_u64(bytes, self.1);
        write_u32(bytes, self.2.value);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        assert!(
            bytes.len() >= pos + 16,
            "buffer too small to read arguments"
        );
        let arg1 = unsafe { read_unchecked::<u32>(bytes, pos) };
        let arg2 = unsafe { read_unchecked::<u64>(bytes, pos + 4) };
        let arg3 = unsafe { read_unchecked::<u32>(bytes, pos + 12) };
        ((arg1, arg2, arg3.into()), pos + 16)
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
        let total_len = self.2.len();
        write_u16(bytes, total_len as u16);

        // Write arguments
        write_u64(bytes, self.0);
        write_u32(bytes, self.1.value);

        // Write remaining arguments
        for arg in &self.2 {
            write_u32(bytes, *arg);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the length
        let (total_len, _) = read::<u16>(bytes, pos);
        let total_len = total_len as usize;

        assert!(
            bytes.len() >= pos + 14 + total_len * 4,
            "buffer too small to read arguments"
        );

        // Read first two arguments
        let first = unsafe { read_unchecked::<u64>(bytes, pos + 2) };
        let second = unsafe { read_unchecked::<u32>(bytes, pos + 10) };

        // Read remaining arguments
        let mut rest = ThinVec::with_capacity(total_len);
        for i in 0..total_len {
            let value = unsafe { read_unchecked::<u32>(bytes, pos + 14 + i * 4) };
            rest.push(value);
        }

        ((first, second.into(), rest), pos + 14 + total_len * 4)
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

impl Argument for (u32, u32, ThinVec<u32>) {
    fn encode(self, bytes: &mut Vec<u8>) {
        // Write first argument
        write_u32(bytes, self.0);

        // Write length
        let total_len = self.2.len();
        write_u16(bytes, total_len as u16);

        // Write second argument
        write_u32(bytes, self.1);

        // Write remaining arguments
        for arg in &self.2 {
            write_u32(bytes, *arg);
        }
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        // Read the first argument
        let (first, _) = read::<u32>(bytes, pos);

        // Read the length
        let (total_len, _) = read::<u16>(bytes, pos + 4);
        let total_len = total_len as usize;

        assert!(
            bytes.len() >= pos + 10 + total_len * 4,
            "buffer too small to read arguments"
        );

        // Read the second argument
        let second = unsafe { read_unchecked::<u32>(bytes, pos + 6) };

        // Read remaining arguments
        let mut rest = ThinVec::with_capacity(total_len);
        for i in 0..total_len {
            let value = unsafe { read_unchecked::<u32>(bytes, pos + 10 + i * 4) };
            rest.push(value);
        }

        ((first, second, rest), pos + 10 + total_len * 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thin_vec::thin_vec;

    #[test]
    fn test_read_u8() {
        let bytes = [1, 2, 3];
        let (val, next) = read::<u8>(&bytes, 0);
        assert_eq!(val, 1);
        assert_eq!(next, 1);
    }

    #[test]
    #[should_panic(expected = "buffer too small to read type T")]
    fn test_read_out_of_bounds() {
        let bytes = [1, 2];
        read::<u32>(&bytes, 0);
    }

    #[test]
    fn test_argument_unit() {
        let mut bytes = Vec::new();
        ().encode(&mut bytes);
        assert!(bytes.is_empty());
        let (val, next) = <()>::decode(&bytes, 0);
        assert_eq!(val, ());
        assert_eq!(next, 0);
    }

    #[test]
    fn test_argument_varying_operand() {
        let test_cases = vec![10u32, 500u32, 100_000u32];
        for val in test_cases {
            let arg = VaryingOperand::new(val);
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) = VaryingOperand::decode(&bytes, 0);
            assert_eq!(u32::from(decoded), val);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_varying_operand_i8() {
        let test_cases = vec![(10u32, -5i8), (500u32, 120i8), (100_000u32, -100i8)];
        for (v1, v2) in test_cases {
            let arg = (VaryingOperand::new(v1), v2);
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) = <(VaryingOperand, i8)>::decode(&bytes, 0);
            assert_eq!(u32::from(decoded.0), v1);
            assert_eq!(decoded.1, v2);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_varying_operand_i16() {
        let test_cases = vec![
            (10u32, -500i16),
            (500u32, 30000i16),
            (100_000u32, -20000i16),
        ];
        for (v1, v2) in test_cases {
            let arg = (VaryingOperand::new(v1), v2);
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) = <(VaryingOperand, i16)>::decode(&bytes, 0);
            assert_eq!(u32::from(decoded.0), v1);
            assert_eq!(decoded.1, v2);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_varying_operand_i32() {
        let v1 = 100_000u32;
        let v2 = -1_000_000i32;
        let arg = (VaryingOperand::new(v1), v2);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(VaryingOperand, i32)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_varying_operand_f32() {
        let v1 = 100_000u32;
        let v2 = 3.14f32;
        let arg = (VaryingOperand::new(v1), v2);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(VaryingOperand, f32)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_varying_operand_f64() {
        let v1 = 100_000u32;
        let v2 = 2.71828f64;
        let arg = (VaryingOperand::new(v1), v2);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(VaryingOperand, f64)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_varying_operand_tuple2() {
        let test_cases = vec![
            (10u32, 20u32),
            (500u32, 10u32),
            (10u32, 500u32),
            (500u32, 600u32),
            (100_000u32, 10u32),
        ];
        for (v1, v2) in test_cases {
            let arg = (VaryingOperand::new(v1), VaryingOperand::new(v2));
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) = <(VaryingOperand, VaryingOperand)>::decode(&bytes, 0);
            assert_eq!(u32::from(decoded.0), v1);
            assert_eq!(u32::from(decoded.1), v2);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_varying_operand_tuple3() {
        let test_cases = vec![
            (10u32, 20u32, 30u32),
            (500u32, 10u32, 15u32),
            (100_000u32, 10u32, 500u32),
        ];
        for (v1, v2, v3) in test_cases {
            let arg = (
                VaryingOperand::new(v1),
                VaryingOperand::new(v2),
                VaryingOperand::new(v3),
            );
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) =
                <(VaryingOperand, VaryingOperand, VaryingOperand)>::decode(&bytes, 0);
            assert_eq!(u32::from(decoded.0), v1);
            assert_eq!(u32::from(decoded.1), v2);
            assert_eq!(u32::from(decoded.2), v3);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_varying_operand_tuple4() {
        let test_cases = vec![
            (10u32, 20u32, 30u32, 40u32),
            (500u32, 10u32, 15u32, 20u32),
            (100_000u32, 10u32, 500u32, 1000u32),
        ];
        for (v1, v2, v3, v4) in test_cases {
            let arg = (
                VaryingOperand::new(v1),
                VaryingOperand::new(v2),
                VaryingOperand::new(v3),
                VaryingOperand::new(v4),
            );
            let mut bytes = Vec::new();
            arg.encode(&mut bytes);
            let (decoded, next) = <(
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
                VaryingOperand,
            )>::decode(&bytes, 0);
            assert_eq!(u32::from(decoded.0), v1);
            assert_eq!(u32::from(decoded.1), v2);
            assert_eq!(u32::from(decoded.2), v3);
            assert_eq!(u32::from(decoded.3), v4);
            assert_eq!(next, bytes.len());
        }
    }

    #[test]
    fn test_argument_u32() {
        let val = 0x12345678u32;
        let mut bytes = Vec::new();
        val.encode(&mut bytes);
        let (decoded, next) = <u32>::decode(&bytes, 0);
        assert_eq!(decoded, val);
        assert_eq!(next, 4);
    }

    #[test]
    fn test_argument_u32_varying() {
        let v1 = 0x12345678u32;
        let v2 = 500u32;
        let arg = (v1, VaryingOperand::new(v2));
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u32, VaryingOperand)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(u32::from(decoded.1), v2);
        assert_eq!(next, 8);
    }

    #[test]
    fn test_argument_thinvec_varying() {
        let v1 = VaryingOperand::new(100u32);
        let rest = thin_vec![VaryingOperand::new(200u32), VaryingOperand::new(300u32)];
        let arg = (v1, rest.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(VaryingOperand, ThinVec<VaryingOperand>)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), 100);
        assert_eq!(decoded.1.len(), 2);
        assert_eq!(u32::from(decoded.1[0]), 200);
        assert_eq!(u32::from(decoded.1[1]), 300);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_complex_u32_u64_varying() {
        let v1 = 0x11223344u32;
        let v2 = 0x5566778899AABBCCu64;
        let v3 = VaryingOperand::new(0xDEADBEEFu32);
        let arg = (v1, v2, v3);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u32, u64, VaryingOperand)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(u32::from(decoded.2), 0xDEADBEEF);
        assert_eq!(next, 16);
    }

    #[test]
    fn test_argument_thinvec_u32() {
        let v1 = 0x12345678u32;
        let rest = thin_vec![0x11111111u32, 0x22222222u32];
        let arg = (v1, rest.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u32, ThinVec<u32>)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(decoded.1, rest);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_u32_varying_varying() {
        let v1 = 100u32;
        let v2 = VaryingOperand::new(200u32);
        let v3 = VaryingOperand::new(300u32);
        let arg = (v1, v2, v3);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u32, VaryingOperand, VaryingOperand)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(u32::from(decoded.1), 200);
        assert_eq!(u32::from(decoded.2), 300);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_varying_varying_thinvec_varying() {
        let v1 = VaryingOperand::new(100u32);
        let v2 = VaryingOperand::new(200u32);
        let v3 = thin_vec![VaryingOperand::new(300u32), VaryingOperand::new(400u32)];
        let arg = (v1, v2, v3.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) =
            <(VaryingOperand, VaryingOperand, ThinVec<VaryingOperand>)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), 100);
        assert_eq!(u32::from(decoded.1), 200);
        assert_eq!(decoded.2.len(), 2);
        assert_eq!(u32::from(decoded.2[0]), 300);
        assert_eq!(u32::from(decoded.2[1]), 400);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_u32_u32_varying_varying_varying() {
        let v1 = 1u32;
        let v2 = 2u32;
        let v3 = VaryingOperand::new(3u32);
        let v4 = VaryingOperand::new(4u32);
        let v5 = VaryingOperand::new(5u32);
        let arg = (v1, v2, v3, v4, v5);
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) =
            <(u32, u32, VaryingOperand, VaryingOperand, VaryingOperand)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(u32::from(decoded.2), 3);
        assert_eq!(u32::from(decoded.3), 4);
        assert_eq!(u32::from(decoded.4), 5);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_u64_varying_thinvec_u32() {
        let v1 = 0x1122334455667788u64;
        let v2 = VaryingOperand::new(100u32);
        let v3 = thin_vec![1u32, 2u32, 3u32];
        let arg = (v1, v2, v3.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u64, VaryingOperand, ThinVec<u32>)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(u32::from(decoded.1), 100);
        assert_eq!(decoded.2, v3);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_varying_thinvec_u32() {
        let v1 = VaryingOperand::new(100u32);
        let v2 = thin_vec![1u32, 2u32, 3u32];
        let arg = (v1, v2.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(VaryingOperand, ThinVec<u32>)>::decode(&bytes, 0);
        assert_eq!(u32::from(decoded.0), 100);
        assert_eq!(decoded.1, v2);
        assert_eq!(next, bytes.len());
    }

    #[test]
    fn test_argument_u32_u32_thinvec_u32() {
        let v1 = 100u32;
        let v2 = 200u32;
        let v3 = thin_vec![1u32, 2u32, 3u32];
        let arg = (v1, v2, v3.clone());
        let mut bytes = Vec::new();
        arg.encode(&mut bytes);
        let (decoded, next) = <(u32, u32, ThinVec<u32>)>::decode(&bytes, 0);
        assert_eq!(decoded.0, v1);
        assert_eq!(decoded.1, v2);
        assert_eq!(decoded.2, v3);
        assert_eq!(next, bytes.len());
    }

    #[test]
    #[should_panic]
    fn test_varying_operand_decode_out_of_bounds() {
        let bytes = [1]; // Format::U16, but no data
        VaryingOperand::decode(&bytes, 0);
    }

    #[test]
    #[should_panic]
    fn test_complex_tuple_decode_out_of_bounds() {
        let bytes = [0, 1]; // Format::U8, VaryingOperand::U8(1), but missing i8
        <(VaryingOperand, i8)>::decode(&bytes, 0);
    }

    #[test]
    #[should_panic]
    fn test_thinvec_decode_out_of_bounds() {
        let bytes = [2, 0, 1, 0, 0, 0]; // len=2, first=1, but missing rest
        <(VaryingOperand, ThinVec<VaryingOperand>)>::decode(&bytes, 0);
    }
}
