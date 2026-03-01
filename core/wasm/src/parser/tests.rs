use super::{Parser, WASM_MAGIC, WASM_VERSION};
use crate::types::{ExportDesc, ImportDesc, Limits, ValType};

fn wasm_header() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&WASM_MAGIC);
    bytes.extend_from_slice(&WASM_VERSION.to_le_bytes());
    bytes
}

#[allow(clippy::cast_possible_truncation)]
fn append_section(bytes: &mut Vec<u8>, id: u8, payload: &[u8]) {
    bytes.push(id);
    let mut len = payload.len();
    loop {
        let mut byte = (len & 0x7F) as u8;
        len >>= 7;
        if len > 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if len == 0 {
            break;
        }
    }
    bytes.extend_from_slice(payload);
}

#[test]
fn parse_empty_module() {
    let module = Parser::new(&wasm_header()).parse().unwrap();
    assert!(module.types.is_empty());
    assert!(module.exports.is_empty());
    assert!(module.start.is_none());
}

#[test]
fn parse_invalid_magic() {
    let bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
    assert!(
        Parser::new(&bytes)
            .parse()
            .unwrap_err()
            .to_string()
            .contains("invalid magic")
    );
}

#[test]
fn parse_unsupported_version() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&WASM_MAGIC);
    bytes.extend_from_slice(&99u32.to_le_bytes());
    assert!(
        Parser::new(&bytes)
            .parse()
            .unwrap_err()
            .to_string()
            .contains("unsupported")
    );
}

#[test]
fn parse_type_section() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 1, &[0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F]);
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(module.types.len(), 1);
    assert_eq!(module.types[0].params, vec![ValType::I32, ValType::I32]);
    assert_eq!(module.types[0].results, vec![ValType::I32]);
}

#[test]
fn parse_import_section() {
    let mut bytes = wasm_header();
    append_section(
        &mut bytes,
        2,
        &[
            0x01, 0x03, b'e', b'n', b'v', 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x01,
        ],
    );
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "env");
    assert_eq!(module.imports[0].name, "memory");
    assert!(
        matches!(module.imports[0].desc, ImportDesc::Memory(m) if m.limits == Limits { min: 1, max: None })
    );
}

#[test]
fn parse_export_section() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 7, &[0x01, 0x03, b'a', b'd', b'd', 0x00, 0x00]);
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(module.exports.len(), 1);
    assert_eq!(module.exports[0].name, "add");
    assert_eq!(module.exports[0].desc, ExportDesc::Func(0));
}

#[test]
fn parse_memory_section() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 5, &[0x01, 0x01, 0x01, 0x0A]);
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(
        module.memories[0].limits,
        Limits {
            min: 1,
            max: Some(10)
        }
    );
}

#[test]
fn parse_start_section() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 8, &[0x02]);
    assert_eq!(Parser::new(&bytes).parse().unwrap().start, Some(2));
}

#[test]
fn parse_function_and_code_sections() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 1, &[0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F]);
    append_section(&mut bytes, 3, &[0x01, 0x00]);
    let code_body: &[u8] = &[0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B];
    #[allow(clippy::cast_possible_truncation)]
    let mut code_payload = vec![0x01, code_body.len() as u8];
    code_payload.extend_from_slice(code_body);
    append_section(&mut bytes, 10, &code_payload);
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(module.functions, vec![0]);
    assert_eq!(module.code.len(), 1);
    assert!(module.code[0].locals.is_empty());
    assert_eq!(
        module.code[0].code,
        vec![0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B]
    );
}

#[test]
fn parse_custom_section() {
    let mut bytes = wasm_header();
    append_section(&mut bytes, 0, &[0x04, b't', b'e', b's', b't', 0xDE, 0xAD]);
    let module = Parser::new(&bytes).parse().unwrap();
    assert_eq!(module.custom_sections[0].name, "test");
    assert_eq!(module.custom_sections[0].data, vec![0xDE, 0xAD]);
}

#[test]
fn truncated_input_errors() {
    assert!(Parser::new(&WASM_MAGIC).parse().is_err());
}
