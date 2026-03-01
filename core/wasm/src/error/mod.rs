//! Error and result implementation for the parser.

use std::fmt;

/// Result of a parsing operation.
pub type ParseResult<T> = Result<T, Error>;

/// Errors encountered during parsing a `WebAssembly` binary module.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// Unexpected end of input.
    UnexpectedEof {
        /// Context.
        context: &'static str,
    },
    /// Invalid magic number.
    InvalidMagic {
        /// Found bytes.
        found: [u8; 4],
    },
    /// Unsupported binary format version.
    UnsupportedVersion {
        /// Found version.
        found: u32,
    },
    /// LEB128 integer overflow.
    Leb128Overflow {
        /// Context.
        context: &'static str,
    },
    /// Unknown section ID.
    InvalidSectionId {
        /// Section ID.
        id: u8,
    },
    /// Unknown value type.
    InvalidValueType {
        /// Byte value.
        byte: u8,
    },
    /// Invalid function type tag.
    InvalidFuncTypeTag {
        /// Byte value.
        byte: u8,
    },
    /// Unknown descriptor tag.
    InvalidDescriptorTag {
        /// Byte value.
        byte: u8,
    },
    /// Invalid mutability flag.
    InvalidMutability {
        /// Byte value.
        byte: u8,
    },
    /// Invalid reference type.
    InvalidRefType {
        /// Byte value.
        byte: u8,
    },
    /// Invalid UTF-8 in name.
    InvalidUtf8,
    /// Trailing bytes after a section.
    TrailingBytes {
        /// Section name.
        section: &'static str,
        /// Remaining bytes.
        remaining: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof { context } => {
                write!(f, "unexpected end of input while parsing {context}")
            }
            Self::InvalidMagic { found } => {
                write!(
                    f,
                    "invalid magic number: expected [0, 97, 115, 109], found {found:?}"
                )
            }
            Self::UnsupportedVersion { found } => write!(f, "unsupported wasm version: {found}"),
            Self::Leb128Overflow { context } => {
                write!(f, "LEB128 overflow while parsing {context}")
            }
            Self::InvalidSectionId { id } => write!(f, "invalid section id: {id}"),
            Self::InvalidValueType { byte } => write!(f, "invalid value type: 0x{byte:02x}"),
            Self::InvalidFuncTypeTag { byte } => {
                write!(f, "expected function type tag 0x60, found 0x{byte:02x}")
            }
            Self::InvalidDescriptorTag { byte } => {
                write!(f, "invalid descriptor tag: 0x{byte:02x}")
            }
            Self::InvalidMutability { byte } => {
                write!(f, "invalid mutability flag: 0x{byte:02x}")
            }
            Self::InvalidRefType { byte } => write!(f, "invalid reference type: 0x{byte:02x}"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in name"),
            Self::TrailingBytes { section, remaining } => {
                write!(f, "{remaining} trailing bytes after {section} section")
            }
        }
    }
}

impl std::error::Error for Error {}
