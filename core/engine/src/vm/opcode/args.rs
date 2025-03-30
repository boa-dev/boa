use super::{VaryingOperand, VaryingOperandValue};

/// The opcode argument formats of the vm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ArgumentsFormat {
    OpcodeOnly,
    OneArgU8,
    OneArgU16,
    OneArgU32,
    TwoArgU8,
    TwoArgU16,
    TwoArgU32,
    ThreeArgU8,
    ThreeArgU16,
    ThreeArgU32,
    FourArgU32,
    VariableArgsU32,
    Reserved,
}

impl ArgumentsFormat {
    const fn encode(self) -> u64 {
        (self as u64) << 48
    }

    pub(crate) const fn decode(instruction: u64) -> Self {
        match (instruction >> 48) as u8 {
            0 => Self::OpcodeOnly,
            1 => Self::OneArgU8,
            2 => Self::OneArgU16,
            3 => Self::OneArgU32,
            4 => Self::TwoArgU8,
            5 => Self::TwoArgU16,
            6 => Self::TwoArgU32,
            7 => Self::ThreeArgU8,
            8 => Self::ThreeArgU16,
            9 => Self::ThreeArgU32,
            10 => Self::FourArgU32,
            11 => Self::VariableArgsU32,
            _ => Self::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OneArgU8(u8);

impl OneArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::OneArgU8.encode() | ((self.0 as u64) << 40)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 40) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OneArgU16(u16);

impl OneArgU16 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::OneArgU16.encode() | ((self.0 as u64) << 32)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 32) as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OneArgU32(u32);

impl OneArgU32 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::OneArgU32.encode() | ((self.0 as u64) << 16)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 16) as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoArgU8(u8, u8);

impl TwoArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::TwoArgU8.encode() | ((self.0 as u64) << 40) | ((self.1 as u64) << 32)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 40) as u8, (instruction >> 32) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoArgU16(u16, u16);

impl TwoArgU16 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::TwoArgU16.encode() | ((self.0 as u64) << 32) | ((self.1 as u64) << 16)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 32) as u16, (instruction >> 16) as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoArgU32(u32, u32);

impl TwoArgU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1);
        ArgumentsFormat::TwoArgU32.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(extended[index], extended[index + 1])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThreeArgU8(u8, u8, u8);

impl ThreeArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::ThreeArgU8.encode()
            | ((self.0 as u64) << 40)
            | ((self.1 as u64) << 32)
            | ((self.2 as u64) << 24)
    }

    const fn decode(instruction: u64) -> Self {
        Self(
            (instruction >> 40) as u8,
            (instruction >> 32) as u8,
            (instruction >> 24) as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThreeArgU16(u16, u16, u16);

impl ThreeArgU16 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::ThreeArgU16.encode()
            | ((self.0 as u64) << 32)
            | ((self.1 as u64) << 16)
            | self.2 as u64
    }

    const fn decode(instruction: u64) -> Self {
        Self(
            (instruction >> 32) as u16,
            (instruction >> 16) as u16,
            instruction as u16,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThreeArgU32(u32, u32, u32);

impl ThreeArgU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1);
        extended.push(self.2);
        ArgumentsFormat::ThreeArgU32.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(extended[index], extended[index + 1], extended[index + 2])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FourArgU32(u32, u32, u32, u32);

impl FourArgU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1);
        extended.push(self.2);
        extended.push(self.3);
        ArgumentsFormat::FourArgU32.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(
            extended[index],
            extended[index + 1],
            extended[index + 2],
            extended[index + 3],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariableArgsU32(Vec<u32>);

impl VariableArgsU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        let len = self.0.len();
        for arg in self.0 {
            extended.push(arg);
        }
        ArgumentsFormat::VariableArgsU32.encode() | ((index as u64) << 16) | u64::from(len as u16)
    }

    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        let len = instruction as u16 as usize;
        let mut args = Vec::with_capacity(len);
        for i in 0..len {
            args.push(extended[index + i]);
        }
        Self(args)
    }
}

pub(crate) trait Argument: Sized + std::fmt::Debug {
    fn encode(self, extended: &mut Vec<u32>) -> u64;
    fn decode(instruction: u64, extended: &[u32]) -> Self;
}

impl Argument for () {
    fn encode(self, _: &mut Vec<u32>) -> u64 {
        ArgumentsFormat::OpcodeOnly.encode()
    }

    #[inline(always)]
    fn decode(_: u64, _: &[u32]) -> Self {}
}

impl Argument for VaryingOperand {
    fn encode(self, _: &mut Vec<u32>) -> u64 {
        match self.value() {
            VaryingOperandValue::U8(value) => OneArgU8(value).encode(),
            VaryingOperandValue::U16(value) => OneArgU16(value).encode(),
            VaryingOperandValue::U32(value) => OneArgU32(value).encode(),
        }
    }

    #[inline(always)]
    fn decode(instruction: u64, _: &[u32]) -> Self {
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::OneArgU8 => OneArgU8::decode(instruction).0.into(),
            ArgumentsFormat::OneArgU16 => OneArgU16::decode(instruction).0.into(),
            ArgumentsFormat::OneArgU32 => OneArgU32::decode(instruction).0.into(),
            _ => unreachable!(),
        }
    }
}

impl Argument for (VaryingOperand, i8) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        match self.0.value() {
            VaryingOperandValue::U8(value) => TwoArgU8(value, self.1 as u8).encode(),
            VaryingOperandValue::U16(value) => TwoArgU16(value, self.1 as u16).encode(),
            VaryingOperandValue::U32(value) => TwoArgU32(value, self.1 as u32).encode(extended),
        }
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::TwoArgU8 => {
                let args = TwoArgU8::decode(instruction);
                (args.0.into(), args.1 as i8)
            }
            ArgumentsFormat::TwoArgU16 => {
                let args = TwoArgU16::decode(instruction);
                (args.0.into(), args.1 as i8)
            }
            ArgumentsFormat::TwoArgU32 => {
                let args = TwoArgU32::decode(instruction, extended);
                (args.0.into(), args.1 as i8)
            }
            _ => unreachable!(),
        }
    }
}

impl Argument for (VaryingOperand, i16) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        match self.0.value() {
            VaryingOperandValue::U8(value) => TwoArgU16(value.into(), self.1 as u16).encode(),
            VaryingOperandValue::U16(value) => TwoArgU16(value, self.1 as u16).encode(),
            VaryingOperandValue::U32(value) => TwoArgU32(value, self.1 as u32).encode(extended),
        }
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::TwoArgU16 => {
                let args = TwoArgU16::decode(instruction);
                (args.0.into(), args.1 as i16)
            }
            ArgumentsFormat::TwoArgU32 => {
                let args = TwoArgU32::decode(instruction, extended);
                (args.0.into(), args.1 as i16)
            }
            _ => unreachable!(),
        }
    }
}

impl Argument for (VaryingOperand, i32) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        TwoArgU32(self.0.value, self.1 as u32).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        (args.0.into(), args.1 as i32)
    }
}

impl Argument for (VaryingOperand, f32) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        TwoArgU32(self.0.value, self.1.to_bits()).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        (args.0.into(), f32::from_bits(args.1))
    }
}

impl Argument for (VaryingOperand, f64) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let float = self.1.to_bits();
        let low = (float & 0xFFFF_FFFF) as u32;
        let high = (float >> 32) as u32;
        ThreeArgU32(self.0.value, low, high).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = ThreeArgU32::decode(instruction, extended);
        let low: u64 = args.1.into();
        let high: u64 = args.2.into();
        let float = (high << 32) | low;
        (args.0.into(), f64::from_bits(float))
    }
}

impl Argument for (VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        match (self.0.value(), self.1.value()) {
            (VaryingOperandValue::U8(lhs), VaryingOperandValue::U8(rhs)) => {
                TwoArgU8(lhs, rhs).encode()
            }
            (VaryingOperandValue::U8(lhs), VaryingOperandValue::U16(rhs)) => {
                TwoArgU16(lhs.into(), rhs).encode()
            }
            (VaryingOperandValue::U16(lhs), VaryingOperandValue::U8(rhs)) => {
                TwoArgU16(lhs, rhs.into()).encode()
            }
            (VaryingOperandValue::U16(lhs), VaryingOperandValue::U16(rhs)) => {
                TwoArgU16(lhs, rhs).encode()
            }
            _ => TwoArgU32(self.0.value, self.1.value).encode(extended),
        }
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::TwoArgU8 => {
                let args = TwoArgU8::decode(instruction);
                (args.0.into(), args.1.into())
            }
            ArgumentsFormat::TwoArgU16 => {
                let args = TwoArgU16::decode(instruction);
                (args.0.into(), args.1.into())
            }
            ArgumentsFormat::TwoArgU32 => {
                let args = TwoArgU32::decode(instruction, extended);
                (args.0.into(), args.1.into())
            }
            _ => unreachable!(),
        }
    }
}

impl Argument for (VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        match (self.0.value(), self.1.value(), self.2.value()) {
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U8(rhs),
            ) => ThreeArgU8(lhs, mid, rhs).encode(),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U16(rhs),
            ) => ThreeArgU16(lhs.into(), mid.into(), rhs).encode(),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U8(rhs),
            ) => ThreeArgU16(lhs.into(), mid, rhs.into()).encode(),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U16(rhs),
            ) => ThreeArgU16(lhs.into(), mid, rhs).encode(),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U8(rhs),
            ) => ThreeArgU16(lhs, mid.into(), rhs.into()).encode(),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U16(rhs),
            ) => ThreeArgU16(lhs, mid.into(), rhs).encode(),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U8(rhs),
            ) => ThreeArgU16(lhs, mid, rhs.into()).encode(),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U16(rhs),
            ) => ThreeArgU16(lhs, mid, rhs).encode(),
            _ => ThreeArgU32(self.0.value, self.1.value, self.2.value).encode(extended),
        }
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::ThreeArgU8 => {
                let args = ThreeArgU8::decode(instruction);
                (args.0.into(), args.1.into(), args.2.into())
            }
            ArgumentsFormat::ThreeArgU16 => {
                let args = ThreeArgU16::decode(instruction);
                (args.0.into(), args.1.into(), args.2.into())
            }
            ArgumentsFormat::ThreeArgU32 => {
                let args = ThreeArgU32::decode(instruction, extended);
                (args.0.into(), args.1.into(), args.2.into())
            }
            _ => unreachable!(),
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
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        FourArgU32(self.0.value, self.1.value, self.2.value, self.3.value).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = FourArgU32::decode(instruction, extended);
        (args.0.into(), args.1.into(), args.2.into(), args.3.into())
    }
}

impl Argument for u32 {
    fn encode(self, _: &mut Vec<u32>) -> u64 {
        OneArgU32(self).encode()
    }

    #[inline(always)]
    fn decode(instruction: u64, _: &[u32]) -> Self {
        OneArgU32::decode(instruction).0
    }
}

impl Argument for (u32, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        TwoArgU32(self.0, self.1.value).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        (args.0, args.1.into())
    }
}

impl Argument for (u32, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        ThreeArgU32(self.0, self.1.value, self.2.value).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = ThreeArgU32::decode(instruction, extended);
        (args.0, args.1.into(), args.2.into())
    }
}

impl Argument for (VaryingOperand, Vec<VaryingOperand>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0.value);
        for arg in self.1 {
            args.push(arg.value);
        }
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        let rest = rest.iter().map(|v| (*v).into()).collect();
        (one[0].into(), rest)
    }
}

impl Argument for (VaryingOperand, VaryingOperand, Vec<VaryingOperand>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.2.len() + 2);
        args.push(self.0.value);
        args.push(self.1.value);
        for arg in self.2 {
            args.push(arg.value);
        }
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(2);
        let rest = rest.iter().map(|v| (*v).into()).collect();
        (one[0].into(), one[1].into(), rest)
    }
}

impl Argument for (u32, u64, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Vec::from([
            self.0,
            (self.1 & 0xFFFF_FFFF) as u32,
            (self.1 >> 32) as u32,
            self.2.value,
        ]);
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let low = args.0[1];
        let high = args.0[2];
        let two = (u64::from(high) << 32) | u64::from(low);
        (args.0[0], two, args.0[3].into())
    }
}

impl Argument for (u32, u32, VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Vec::from([self.0, self.1, self.2.value, self.3.value, self.4.value]);
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        (
            args.0[0],
            args.0[1],
            args.0[2].into(),
            args.0[3].into(),
            args.0[4].into(),
        )
    }
}

impl Argument for (u32, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0);
        for arg in self.1 {
            args.push(arg);
        }
        VariableArgsU32(args).encode(extended)
    }

    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        (one[0], Vec::from(rest))
    }
}

impl Argument for (u64, VaryingOperand, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity((self.2.len()) + 3);
        args.push((self.0 & 0xFFFF_FFFF) as u32);
        args.push((self.0 >> 32) as u32);
        args.push(self.1.value);
        for arg in self.2 {
            args.push(arg);
        }
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(3);
        let low = args.0[0];
        let high = args.0[1];
        let two = (u64::from(high) << 32) | u64::from(low);
        (two, one[2].into(), Vec::from(rest))
    }
}

impl Argument for (VaryingOperand, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0.value);
        for arg in self.1 {
            args.push(arg);
        }
        VariableArgsU32(args).encode(extended)
    }

    #[inline(always)]
    fn decode(instruction: u64, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        (one[0].into(), Vec::from(rest))
    }
}
