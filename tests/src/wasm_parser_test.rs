#[test]
fn test_basic_wasm_header() {
    // WebAssembly magic number
    let wasm_header = [0x00, 0x61, 0x73, 0x6D];

    assert_eq!(wasm_header, [0x00, 0x61, 0x73, 0x6D]);
}