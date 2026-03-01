//! Boa `WebAssembly` binary parser implementation.

mod cursor;
mod section;

#[cfg(test)]
mod tests;

use self::cursor::Cursor;
use crate::{
    Error,
    error::ParseResult,
    types::{
        CustomSection, Data, Element, Export, FuncBody, FuncType, Global, Import, MemType,
        TableType,
    },
};

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
const WASM_VERSION: u32 = 1;

/// A decoded `WebAssembly` module.
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Custom sections.
    pub custom_sections: Vec<CustomSection>,
    /// Function signatures.
    pub types: Vec<FuncType>,
    /// Imports.
    pub imports: Vec<Import>,
    /// Type indices for each function.
    pub functions: Vec<u32>,
    /// Tables.
    pub tables: Vec<TableType>,
    /// Memories.
    pub memories: Vec<MemType>,
    /// Globals.
    pub globals: Vec<Global>,
    /// Exports.
    pub exports: Vec<Export>,
    /// Start function index.
    pub start: Option<u32>,
    /// Element segments.
    pub elements: Vec<Element>,
    /// Function bodies.
    pub code: Vec<FuncBody>,
    /// Data segments.
    pub data: Vec<Data>,
    /// Data count.
    pub data_count: Option<u32>,
}

/// Parser for the `WebAssembly` binary format.
///
/// # Examples
///
/// ```
/// use boa_wasm_parser::Parser;
///
/// let wasm = b"\0asm\x01\x00\x00\x00";
/// let module = Parser::new(wasm).parse().expect("valid wasm");
/// assert!(module.types.is_empty());
/// ```
#[derive(Debug)]
pub struct Parser<'a> {
    /// Cursor of the parser, pointing to the binary input.
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` with the given byte slice as input.
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
        }
    }

    /// Parse the full input as a `WebAssembly` module.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid magic numbers,
    /// unsupported versions, malformed sections, or unexpected end of input.
    pub fn parse(mut self) -> ParseResult<Module> {
        let magic = self.cursor.read_bytes(4, "magic number")?;
        if magic != WASM_MAGIC {
            let mut found = [0u8; 4];
            found.copy_from_slice(magic);
            return Err(Error::InvalidMagic { found });
        }

        let version_bytes = self.cursor.read_bytes(4, "version")?;
        let version = u32::from_le_bytes([
            version_bytes[0],
            version_bytes[1],
            version_bytes[2],
            version_bytes[3],
        ]);
        if version != WASM_VERSION {
            return Err(Error::UnsupportedVersion { found: version });
        }

        let mut module = Module::default();
        while !self.cursor.is_empty() {
            let section_id = self.cursor.read_u8("section id")?;
            let section_size = self.cursor.read_u32("section size")? as usize;
            let mut section_cursor = self.cursor.sub_cursor(section_size, "section payload")?;

            match section_id {
                0 => module
                    .custom_sections
                    .push(section::parse_custom_section(&mut section_cursor)?),
                1 => module.types = section::parse_type_section(&mut section_cursor)?,
                2 => module.imports = section::parse_import_section(&mut section_cursor)?,
                3 => module.functions = section::parse_function_section(&mut section_cursor)?,
                4 => module.tables = section::parse_table_section(&mut section_cursor)?,
                5 => module.memories = section::parse_memory_section(&mut section_cursor)?,
                6 => module.globals = section::parse_global_section(&mut section_cursor)?,
                7 => module.exports = section::parse_export_section(&mut section_cursor)?,
                8 => module.start = Some(section::parse_start_section(&mut section_cursor)?),
                9 => module.elements = section::parse_element_section(&mut section_cursor)?,
                10 => module.code = section::parse_code_section(&mut section_cursor)?,
                11 => module.data = section::parse_data_section(&mut section_cursor)?,
                12 => {
                    module.data_count =
                        Some(section::parse_data_count_section(&mut section_cursor)?);
                }
                _ => return Err(Error::InvalidSectionId { id: section_id }),
            }

            if !section_cursor.is_empty() {
                let section_name = match section_id {
                    0 => "custom",
                    1 => "type",
                    2 => "import",
                    3 => "function",
                    4 => "table",
                    5 => "memory",
                    6 => "global",
                    7 => "export",
                    8 => "start",
                    9 => "element",
                    10 => "code",
                    11 => "data",
                    12 => "data count",
                    _ => "unknown",
                };
                return Err(Error::TrailingBytes {
                    section: section_name,
                    remaining: section_cursor.remaining(),
                });
            }
        }
        Ok(module)
    }
}
