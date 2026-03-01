//! Section parsers for the `WebAssembly` binary format.

use super::cursor::Cursor;
use crate::{
    Error,
    error::ParseResult,
    types::{
        CustomSection, Data, DataMode, ElemInit, ElemMode, Element, Export, ExportDesc, FuncBody,
        FuncType, Global, GlobalType, Import, ImportDesc, Limits, Local, MemType, Mut, RefType,
        TableType, ValType,
    },
};

fn read_val_type(r: &mut Cursor<'_>) -> ParseResult<ValType> {
    let byte = r.read_u8("value type")?;
    match byte {
        0x7F => Ok(ValType::I32),
        0x7E => Ok(ValType::I64),
        0x7D => Ok(ValType::F32),
        0x7C => Ok(ValType::F64),
        0x7B => Ok(ValType::V128),
        0x70 => Ok(ValType::FuncRef),
        0x6F => Ok(ValType::ExternRef),
        _ => Err(Error::InvalidValueType { byte }),
    }
}

fn read_val_type_vec(r: &mut Cursor<'_>) -> ParseResult<Vec<ValType>> {
    let count = r.read_u32("vec count")? as usize;
    (0..count).map(|_| read_val_type(r)).collect()
}

fn read_func_type(r: &mut Cursor<'_>) -> ParseResult<FuncType> {
    let tag = r.read_u8("func type tag")?;
    if tag != 0x60 {
        return Err(Error::InvalidFuncTypeTag { byte: tag });
    }
    Ok(FuncType {
        params: read_val_type_vec(r)?,
        results: read_val_type_vec(r)?,
    })
}

fn read_limits(r: &mut Cursor<'_>) -> ParseResult<Limits> {
    let flag = r.read_u8("limits flag")?;
    let min = r.read_u32("limits min")?;
    let max = if flag == 0x01 {
        Some(r.read_u32("limits max")?)
    } else {
        None
    };
    Ok(Limits { min, max })
}

fn read_table_type(r: &mut Cursor<'_>) -> ParseResult<TableType> {
    let byte = r.read_u8("table element type")?;
    let element = match byte {
        0x70 => RefType::FuncRef,
        0x6F => RefType::ExternRef,
        _ => return Err(Error::InvalidRefType { byte }),
    };
    let limits = read_limits(r)?;
    Ok(TableType { element, limits })
}

fn read_mem_type(r: &mut Cursor<'_>) -> ParseResult<MemType> {
    Ok(MemType {
        limits: read_limits(r)?,
    })
}

fn read_global_type(r: &mut Cursor<'_>) -> ParseResult<GlobalType> {
    let val_type = read_val_type(r)?;
    let byte = r.read_u8("mutability")?;
    let mutability = match byte {
        0x00 => Mut::Const,
        0x01 => Mut::Var,
        _ => return Err(Error::InvalidMutability { byte }),
    };
    Ok(GlobalType {
        val_type,
        mutability,
    })
}

pub(super) fn parse_custom_section(r: &mut Cursor<'_>) -> ParseResult<CustomSection> {
    let name = r.read_name()?;
    let data = r.read_bytes(r.remaining(), "custom section data")?.to_vec();
    Ok(CustomSection { name, data })
}

pub(super) fn parse_type_section(r: &mut Cursor<'_>) -> ParseResult<Vec<FuncType>> {
    let count = r.read_u32("type section count")? as usize;
    (0..count).map(|_| read_func_type(r)).collect()
}

pub(super) fn parse_import_section(r: &mut Cursor<'_>) -> ParseResult<Vec<Import>> {
    let count = r.read_u32("import count")? as usize;
    let mut imports = Vec::with_capacity(count);
    for _ in 0..count {
        let module = r.read_name()?;
        let name = r.read_name()?;
        let tag = r.read_u8("import desc tag")?;
        let desc = match tag {
            0x00 => ImportDesc::Func(r.read_u32("import func type idx")?),
            0x01 => ImportDesc::Table(read_table_type(r)?),
            0x02 => ImportDesc::Memory(read_mem_type(r)?),
            0x03 => ImportDesc::Global(read_global_type(r)?),
            _ => return Err(Error::InvalidDescriptorTag { byte: tag }),
        };
        imports.push(Import { module, name, desc });
    }
    Ok(imports)
}

pub(super) fn parse_function_section(r: &mut Cursor<'_>) -> ParseResult<Vec<u32>> {
    let count = r.read_u32("function count")? as usize;
    (0..count)
        .map(|_| r.read_u32("function type index"))
        .collect()
}

pub(super) fn parse_table_section(r: &mut Cursor<'_>) -> ParseResult<Vec<TableType>> {
    let count = r.read_u32("table count")? as usize;
    (0..count).map(|_| read_table_type(r)).collect()
}

pub(super) fn parse_memory_section(r: &mut Cursor<'_>) -> ParseResult<Vec<MemType>> {
    let count = r.read_u32("memory count")? as usize;
    (0..count).map(|_| read_mem_type(r)).collect()
}

pub(super) fn parse_global_section(r: &mut Cursor<'_>) -> ParseResult<Vec<Global>> {
    let count = r.read_u32("global count")? as usize;
    let mut globals = Vec::with_capacity(count);
    for _ in 0..count {
        let global_type = read_global_type(r)?;
        let init_expr = r.read_const_expr()?;
        globals.push(Global {
            global_type,
            init_expr,
        });
    }
    Ok(globals)
}

pub(super) fn parse_export_section(r: &mut Cursor<'_>) -> ParseResult<Vec<Export>> {
    let count = r.read_u32("export count")? as usize;
    let mut exports = Vec::with_capacity(count);
    for _ in 0..count {
        let name = r.read_name()?;
        let tag = r.read_u8("export desc tag")?;
        let idx = r.read_u32("export desc index")?;
        let desc = match tag {
            0x00 => ExportDesc::Func(idx),
            0x01 => ExportDesc::Table(idx),
            0x02 => ExportDesc::Memory(idx),
            0x03 => ExportDesc::Global(idx),
            _ => return Err(Error::InvalidDescriptorTag { byte: tag }),
        };
        exports.push(Export { name, desc });
    }
    Ok(exports)
}

pub(super) fn parse_start_section(r: &mut Cursor<'_>) -> ParseResult<u32> {
    r.read_u32("start function index")
}

pub(super) fn parse_element_section(r: &mut Cursor<'_>) -> ParseResult<Vec<Element>> {
    let count = r.read_u32("element count")? as usize;
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        let flags = r.read_u32("element flags")?;
        elements.push(parse_element_entry(r, flags)?);
    }
    Ok(elements)
}

fn parse_element_entry(r: &mut Cursor<'_>, flags: u32) -> ParseResult<Element> {
    let has_table_or_active = (flags & 0x01) == 0;
    let has_explicit_info = (flags & 0x02) != 0;
    let uses_exprs = (flags & 0x04) != 0;

    let mode = if has_table_or_active {
        let table_idx = if has_explicit_info {
            r.read_u32("element table index")?
        } else {
            0
        };
        let offset_expr = r.read_const_expr()?;
        ElemMode::Active {
            table_idx,
            offset_expr,
        }
    } else if has_explicit_info {
        ElemMode::Declarative
    } else {
        ElemMode::Passive
    };

    let ref_type = if has_explicit_info {
        if uses_exprs {
            let byte = r.read_u8("element ref type")?;
            match byte {
                0x70 => RefType::FuncRef,
                0x6F => RefType::ExternRef,
                _ => return Err(Error::InvalidRefType { byte }),
            }
        } else {
            let _ = r.read_u8("element kind")?;
            RefType::FuncRef
        }
    } else {
        RefType::FuncRef
    };

    let init = if uses_exprs {
        let n = r.read_u32("element expr count")? as usize;
        let mut exprs = Vec::with_capacity(n);
        for _ in 0..n {
            exprs.push(r.read_const_expr()?);
        }
        ElemInit::Expressions(exprs)
    } else {
        let n = r.read_u32("element func index count")? as usize;
        (0..n)
            .map(|_| r.read_u32("element func index"))
            .collect::<ParseResult<Vec<_>>>()
            .map(ElemInit::FuncIndices)?
    };

    Ok(Element {
        ref_type,
        mode,
        init,
    })
}

pub(super) fn parse_code_section(r: &mut Cursor<'_>) -> ParseResult<Vec<FuncBody>> {
    let count = r.read_u32("code count")? as usize;
    let mut bodies = Vec::with_capacity(count);
    for _ in 0..count {
        let body_size = r.read_u32("code body size")? as usize;
        let mut body_cursor = r.sub_cursor(body_size, "code body")?;
        let local_count = body_cursor.read_u32("local decl count")? as usize;
        let mut locals = Vec::with_capacity(local_count);
        for _ in 0..local_count {
            let count = body_cursor.read_u32("local count")?;
            let val_type = read_val_type(&mut body_cursor)?;
            locals.push(Local { count, val_type });
        }
        let code = body_cursor
            .read_bytes(body_cursor.remaining(), "code instructions")?
            .to_vec();
        bodies.push(FuncBody { locals, code });
    }
    Ok(bodies)
}

pub(super) fn parse_data_section(r: &mut Cursor<'_>) -> ParseResult<Vec<Data>> {
    let count = r.read_u32("data count")? as usize;
    let mut segments = Vec::with_capacity(count);
    for _ in 0..count {
        let flags = r.read_u32("data flags")?;
        let mode = match flags {
            0 => DataMode::Active {
                mem_idx: 0,
                offset_expr: r.read_const_expr()?,
            },
            1 => DataMode::Passive,
            2 => {
                let mem_idx = r.read_u32("data memory index")?;
                DataMode::Active {
                    mem_idx,
                    offset_expr: r.read_const_expr()?,
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            _ => return Err(Error::InvalidDescriptorTag { byte: flags as u8 }),
        };
        let len = r.read_u32("data byte length")? as usize;
        let data = r.read_bytes(len, "data bytes")?.to_vec();
        segments.push(Data { mode, data });
    }
    Ok(segments)
}

pub(super) fn parse_data_count_section(r: &mut Cursor<'_>) -> ParseResult<u32> {
    r.read_u32("data count")
}
