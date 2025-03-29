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
    FourArgU8,
    FourArgU16,
    FourArgU32,
    VariableArgsU32,
    Jump,
    JumpOneArgU8,
    JumpOneArgU16,
    JumpOneArgU32,
    JumpTwoArgU8,
    JumpTwoArgU16,
    JumpTwoArgU32,
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
            10 => Self::FourArgU8,
            11 => Self::FourArgU16,
            12 => Self::FourArgU32,
            13 => Self::VariableArgsU32,
            14 => Self::Jump,
            15 => Self::JumpOneArgU8,
            16 => Self::JumpOneArgU16,
            17 => Self::JumpOneArgU32,
            18 => Self::JumpTwoArgU8,
            19 => Self::JumpTwoArgU16,
            20 => Self::JumpTwoArgU32,
            _ => Self::Reserved,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Arguments {
    OpcodeOnly,
    OneArgU8(OneArgU8),
    OneArgU16(OneArgU16),
    OneArgU32(OneArgU32),
    TwoArgU8(TwoArgU8),
    TwoArgU16(TwoArgU16),
    TwoArgU32(TwoArgU32),
    ThreeArgU8(ThreeArgU8),
    ThreeArgU16(ThreeArgU16),
    ThreeArgU32(ThreeArgU32),
    FourArgU8(FourArgU8),
    FourArgU16(FourArgU16),
    FourArgU32(FourArgU32),
    VariableArgsU32(VariableArgsU32),
    Jump(JumpArg),
    JumpOneArgU8(JumpOneArgU8),
    JumpOneArgU16(JumpOneArgU16),
    JumpOneArgU32(JumpOneArgU32),
    JumpTwoArgU8(JumpTwoArgU8),
    JumpTwoArgU16(JumpTwoArgU16),
    JumpTwoArgU32(JumpTwoArgU32),
    Reserved,
}

impl Arguments {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        match self {
            Self::OneArgU8(arg) => arg.encode(),
            Self::OneArgU16(arg) => arg.encode(),
            Self::OneArgU32(arg) => arg.encode(),
            Self::TwoArgU8(args) => args.encode(),
            Self::TwoArgU16(args) => args.encode(),
            Self::TwoArgU32(args) => args.encode(extended),
            Self::ThreeArgU8(args) => args.encode(),
            Self::ThreeArgU16(args) => args.encode(),
            Self::ThreeArgU32(args) => args.encode(extended),
            Self::FourArgU8(args) => args.encode(),
            Self::FourArgU16(args) => args.encode(extended),
            Self::FourArgU32(args) => args.encode(extended),
            Self::VariableArgsU32(args) => args.encode(extended),
            Self::Jump(args) => args.encode(),
            Self::JumpOneArgU8(args) => args.encode(),
            Self::JumpOneArgU16(args) => args.encode(),
            Self::JumpOneArgU32(args) => args.encode(extended),
            Self::JumpTwoArgU8(args) => args.encode(),
            Self::JumpTwoArgU16(args) => args.encode(extended),
            Self::JumpTwoArgU32(args) => args.encode(extended),
            Self::OpcodeOnly | Self::Reserved => 0,
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
struct FourArgU8(u8, u8, u8, u8);

impl FourArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::FourArgU8.encode()
            | ((self.0 as u64) << 40)
            | ((self.1 as u64) << 32)
            | ((self.2 as u64) << 24)
            | ((self.3 as u64) << 16)
    }

    const fn decode(instruction: u64) -> Self {
        Self(
            (instruction >> 40) as u8,
            (instruction >> 32) as u8,
            (instruction >> 24) as u8,
            (instruction >> 16) as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FourArgU16(u16, u16, u16, u16);

impl FourArgU16 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0 as u32);
        extended.push(self.1 as u32);
        extended.push(self.2 as u32);
        extended.push(self.3 as u32);
        ArgumentsFormat::FourArgU16.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(
            extended[index] as u16,
            extended[index + 1] as u16,
            extended[index + 2] as u16,
            extended[index + 3] as u16,
        )
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
        ArgumentsFormat::VariableArgsU32.encode() | ((index as u64) << 16) | len as u16 as u64
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpArg(u32);

impl JumpArg {
    const fn encode(self) -> u64 {
        ArgumentsFormat::Jump.encode() | ((self.0 as u64) << 16)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 16) as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpOneArgU8(u32, u8);

impl JumpOneArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::JumpOneArgU8.encode() | ((self.0 as u64) << 16) | ((self.1 as u64) << 8)
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 16) as u32, (instruction >> 8) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpOneArgU16(u32, u16);

impl JumpOneArgU16 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::JumpOneArgU16.encode() | ((self.0 as u64) << 16) | self.1 as u64
    }

    const fn decode(instruction: u64) -> Self {
        Self((instruction >> 16) as u32, instruction as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpOneArgU32(u32, u32);

impl JumpOneArgU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1);
        ArgumentsFormat::JumpOneArgU32.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(extended[index], extended[index + 1])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpTwoArgU8(u32, u8, u8);

impl JumpTwoArgU8 {
    const fn encode(self) -> u64 {
        ArgumentsFormat::JumpTwoArgU8.encode()
            | ((self.0 as u64) << 16)
            | ((self.1 as u64) << 8)
            | self.2 as u64
    }

    const fn decode(instruction: u64) -> Self {
        Self(
            (instruction >> 16) as u32,
            (instruction >> 8) as u8,
            instruction as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpTwoArgU16(u32, u16, u16);

impl JumpTwoArgU16 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1 as u32);
        extended.push(self.2 as u32);
        ArgumentsFormat::JumpTwoArgU16.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(
            extended[index],
            extended[index + 1] as u16,
            extended[index + 2] as u16,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpTwoArgU32(u32, u32, u32);

impl JumpTwoArgU32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let index = extended.len();
        extended.push(self.0);
        extended.push(self.1);
        extended.push(self.2);
        ArgumentsFormat::JumpTwoArgU32.encode() | ((index as u64) << 16)
    }

    const fn decode(instruction: u64, extended: &[u32]) -> Self {
        let index = (instruction >> 16) as u32 as usize;
        Self(extended[index], extended[index + 1], extended[index + 2])
    }
}

pub(crate) trait DecodeAndDispatch: Sized+ std::fmt::Debug {
    fn encode(self, extended: &mut Vec<u32>) -> u64;

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, extended: &[u32]) -> Self;
}

impl DecodeAndDispatch for () {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        Arguments::OpcodeOnly.encode(extended)
    }

    fn decode_and_dispatch(_: u64, _: ArgumentsFormat, _: &[u32]) -> Self {
        return ();
    }
}

impl DecodeAndDispatch for VaryingOperand {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = match self.value() {
            VaryingOperandValue::U8(value) => Arguments::OneArgU8(OneArgU8(value)),
            VaryingOperandValue::U16(value) => Arguments::OneArgU16(OneArgU16(value)),
            VaryingOperandValue::U32(value) => Arguments::OneArgU32(OneArgU32(value)),
        };
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, _: &[u32]) -> Self {
        let args = match format {
            ArgumentsFormat::OneArgU8 => OneArgU8::decode(instruction).0.into(),
            ArgumentsFormat::OneArgU16 => OneArgU16::decode(instruction).0.into(),
            ArgumentsFormat::OneArgU32 => OneArgU32::decode(instruction).0.into(),
            _ => unreachable!(),
        };
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, i8) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = match self.0.value() {
            VaryingOperandValue::U8(value) => Arguments::TwoArgU8(TwoArgU8(value, self.1 as u8)),
            VaryingOperandValue::U16(value) => {
                Arguments::TwoArgU16(TwoArgU16(value, self.1 as u16))
            }
            VaryingOperandValue::U32(value) => {
                Arguments::TwoArgU32(TwoArgU32(value, self.1 as u32))
            }
        };
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = match format {
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
        };
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, i16) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = match self.0.value() {
            VaryingOperandValue::U8(value) => {
                Arguments::TwoArgU16(TwoArgU16(value.into(), self.1 as u16))
            }
            VaryingOperandValue::U16(value) => {
                Arguments::TwoArgU16(TwoArgU16(value, self.1 as u16))
            }
            VaryingOperandValue::U32(value) => {
                Arguments::TwoArgU32(TwoArgU32(value, self.1 as u32))
            }
        };
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = match format {
            ArgumentsFormat::TwoArgU16 => {
                let args = TwoArgU16::decode(instruction);
                (args.0.into(), args.1 as i16)
            }
            ArgumentsFormat::TwoArgU32 => {
                let args = TwoArgU32::decode(instruction, extended);
                (args.0.into(), args.1 as i16)
            }
            _ => unreachable!(),
        };
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, i32) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::TwoArgU32(TwoArgU32(self.0.value, self.1 as u32));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        let args = (args.0.into(), args.1 as i32);
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, f32) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::TwoArgU32(TwoArgU32(self.0.value, self.1.to_bits()));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        let args = (args.0.into(), f32::from_bits(args.1));
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, f64) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let float = self.1.to_bits();
        let low = (float & 0xFFFFFFFF) as u32;
        let high = (float >> 32) as u32;
        let args = Arguments::ThreeArgU32(ThreeArgU32(self.0.value, low, high));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = ThreeArgU32::decode(instruction, extended);
        let low = args.1 as u64;
        let high = args.2 as u64;
        let float = (u64::from(high) << 32) | u64::from(low);
        let args = (args.0.into(), f64::from_bits(float));
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = match (self.0.value(), self.1.value()) {
            (VaryingOperandValue::U8(lhs), VaryingOperandValue::U8(rhs)) => {
                Arguments::TwoArgU8(TwoArgU8(lhs, rhs))
            }
            (VaryingOperandValue::U8(lhs), VaryingOperandValue::U16(rhs)) => {
                Arguments::TwoArgU16(TwoArgU16(lhs.into(), rhs))
            }
            (VaryingOperandValue::U16(lhs), VaryingOperandValue::U8(rhs)) => {
                Arguments::TwoArgU16(TwoArgU16(lhs, rhs.into()))
            }
            (VaryingOperandValue::U16(lhs), VaryingOperandValue::U16(rhs)) => {
                Arguments::TwoArgU16(TwoArgU16(lhs, rhs))
            }
            _ => Arguments::TwoArgU32(TwoArgU32(self.0.value, self.1.value)),
        };
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = match format {
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
        };
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = match (self.0.value(), self.1.value(), self.2.value()) {
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U8(rhs),
            ) => Arguments::ThreeArgU8(ThreeArgU8(lhs, mid, rhs)),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U16(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs.into(), mid.into(), rhs)),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U8(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs.into(), mid, rhs.into())),
            (
                VaryingOperandValue::U8(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U16(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs.into(), mid, rhs)),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U8(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs, mid.into(), rhs.into())),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U8(mid),
                VaryingOperandValue::U16(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs, mid.into(), rhs)),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U8(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs, mid, rhs.into())),
            (
                VaryingOperandValue::U16(lhs),
                VaryingOperandValue::U16(mid),
                VaryingOperandValue::U16(rhs),
            ) => Arguments::ThreeArgU16(ThreeArgU16(lhs, mid, rhs)),
            _ => Arguments::ThreeArgU32(ThreeArgU32(self.0.value, self.1.value, self.2.value)),
        };
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, format: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = match format {
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
        };
        return args;
    }
}

impl DecodeAndDispatch
    for (
        VaryingOperand,
        VaryingOperand,
        VaryingOperand,
        VaryingOperand,
    )
{
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::FourArgU32(FourArgU32(
            self.0.value,
            self.1.value,
            self.2.value,
            self.3.value,
        ));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = FourArgU32::decode(instruction, extended);
        let args = (args.0.into(), args.1.into(), args.2.into(), args.3.into());
        return args;
    }
}

impl DecodeAndDispatch for u32 {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::OneArgU32(OneArgU32(self));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, _: &[u32]) -> Self {
        let args = OneArgU32::decode(instruction).0.into();
        return args;
    }
}

impl DecodeAndDispatch for (u32, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::TwoArgU32(TwoArgU32(self.0, self.1.value));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = TwoArgU32::decode(instruction, extended);
        let args = (args.0, args.1.into());
        return args;
    }
}

impl DecodeAndDispatch for (u32, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let args = Arguments::ThreeArgU32(ThreeArgU32(self.0, self.1.value, self.2.value));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = ThreeArgU32::decode(instruction, extended);
        let args = (args.0.into(), args.1.into(), args.2.into());
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, Vec<VaryingOperand>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0.value);
        for arg in self.1 {
            args.push(arg.value);
        }
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        let rest = rest.iter().map(|v| (*v).into()).collect();
        let args = (one[0].into(), rest);
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, VaryingOperand, Vec<VaryingOperand>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.2.len() + 2);
        args.push(self.0.value);
        args.push(self.1.value);
        for arg in self.2 {
            args.push(arg.value);
        }
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(2);
        let rest = rest.iter().map(|v| (*v).into()).collect();
        let args = (one[0].into(), one[1].into(), rest);
        return args;
    }
}

impl DecodeAndDispatch for (u32, u64, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(4);
        args.push(self.0);
        args.push((self.1 & 0xFFFFFFFF) as u32);
        args.push((self.1 >> 32) as u32);
        args.push(self.2.value);
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let low = args.0[1];
        let high = args.0[2];
        let two = (u64::from(high) << 32) | u64::from(low);
        let args = (args.0[0], two, args.0[3].into());
        return args;
    }
}

impl DecodeAndDispatch for (u32, u32, VaryingOperand, VaryingOperand, VaryingOperand) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(5);
        args.push(self.0);
        args.push(self.1);
        args.push(self.2.value);
        args.push(self.3.value);
        args.push(self.4.value);
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let args = (
            args.0[0],
            args.0[1],
            args.0[2].into(),
            args.0[3].into(),
            args.0[4].into(),
        );
        return args;
    }
}

impl DecodeAndDispatch for (u32, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0);
        for arg in self.1 {
            args.push(arg);
        }
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        let args = (one[0].into(), Vec::from(rest));
        return args;
    }
}

impl DecodeAndDispatch for (u64, VaryingOperand, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity((self.2.len()) + 3);
        args.push((self.0 & 0xFFFFFFFF) as u32);
        args.push((self.0 >> 32) as u32);
        args.push(self.1.value);
        for arg in self.2 {
            args.push(arg);
        }
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(3);
        let low = args.0[0];
        let high = args.0[1];
        let two = (u64::from(high) << 32) | u64::from(low);
        let args = (two, one[2].into(), Vec::from(rest));
        return args;
    }
}

impl DecodeAndDispatch for (VaryingOperand, Vec<u32>) {
    fn encode(self, extended: &mut Vec<u32>) -> u64 {
        let mut args = Vec::with_capacity(self.1.len() + 1);
        args.push(self.0.value);
        for arg in self.1 {
            args.push(arg);
        }
        let args = Arguments::VariableArgsU32(VariableArgsU32(args));
        args.encode(extended)
    }

    fn decode_and_dispatch(instruction: u64, _: ArgumentsFormat, extended: &[u32]) -> Self {
        let args = VariableArgsU32::decode(instruction, extended);
        let (one, rest) = args.0.split_at(1);
        let args = (one[0].into(), Vec::from(rest));
        return args;
    }
}
