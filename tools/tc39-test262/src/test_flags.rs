#![allow(missing_docs)]
use bitflags::bitflags;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};


/// Individual test flag.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    #[serde(rename = "CanBlockIsFalse")]
    CanBlockIsFalse,
    #[serde(rename = "CanBlockIsTrue")]
    CanBlockIsTrue,
    #[serde(rename = "non-deterministic")]
    NonDeterministic,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub  struct TestFlags: u16 {
        const STRICT = 0b0_0000_0001;
        const NO_STRICT = 0b0_0000_0010;
        const MODULE = 0b0_0000_0100;
        const RAW = 0b0_0000_1000;
        const ASYNC = 0b0_0001_0000;
        const GENERATED = 0b0_0010_0000;
        const CAN_BLOCK_IS_FALSE = 0b0_0100_0000;
        const CAN_BLOCK_IS_TRUE = 0b0_1000_0000;
        const NON_DETERMINISTIC = 0b1_0000_0000;
    }
}

impl Default for TestFlags {
    fn default() -> Self {
        Self::STRICT | Self::NO_STRICT
    }
}

impl From<TestFlag> for TestFlags {
    fn from(flag: TestFlag) -> Self {
        match flag {
            TestFlag::OnlyStrict => Self::STRICT,
            TestFlag::NoStrict => Self::NO_STRICT,
            TestFlag::Module => Self::MODULE,
            TestFlag::Raw => Self::RAW,
            TestFlag::Async => Self::ASYNC,
            TestFlag::Generated => Self::GENERATED,
            TestFlag::CanBlockIsFalse => Self::CAN_BLOCK_IS_FALSE,
            TestFlag::CanBlockIsTrue => Self::CAN_BLOCK_IS_TRUE,
            TestFlag::NonDeterministic => Self::NON_DETERMINISTIC,
        }
    }
}

impl<T> From<T> for TestFlags
where
    T: AsRef<[TestFlag]>,
{
    fn from(flags: T) -> Self {
        let flags = flags.as_ref();
        if flags.is_empty() {
            Self::default()
        } else {
            let mut result = Self::empty();
            for flag in flags {
                result |= Self::from(*flag);
            }

            if !result.intersects(Self::default()) {
                result |= Self::default();
            }

            result
        }
    }
}

impl<'de> Deserialize<'de> for TestFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlagsVisitor;

        impl<'de> Visitor<'de> for FlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of flags")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut flags = TestFlags::empty();
                while let Some(elem) = seq.next_element::<TestFlag>()? {
                    flags |= elem.into();
                }
                Ok(flags)
            }
        }

        struct RawFlagsVisitor;

        impl Visitor<'_> for RawFlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a flags number")
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                TestFlags::from_bits(v).ok_or_else(|| {
                    E::invalid_value(Unexpected::Unsigned(v.into()), &"a valid flag number")
                })
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_seq(FlagsVisitor)
        } else {
            deserializer.deserialize_u16(RawFlagsVisitor)
        }
    }
}
