# Snapshot File format

This docoment describes the binary file format of the boa snapshot files.

## Header

The header composes the first part of the snapshot.

| Field                 | Description                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| signature `: [u8; 4]` | This is used to quickly check if this file is a snapshot file (`.boa`)                                       |
| guid                  | Guid generated in compile time and backed into the binary, that is used to check if snapshot is compatibile. |
| checksum              | Checksum that is used to check that the snapshot is not corrupted.                                           |

## JsString Table

After the `Header` the table containing `JsString`s each entry contains

| static? `: u8` | length: `: usize` | `JsString` elements `: [u16]` |
| -------------- | ----------------- | ----------------------------- |
| 0              | 5                 | `'H', 'e', 'l', 'l', 'o'`     |
| 1              | -                 | 3                             |
| ...            | ...               | ...                           |

If it's a static string then it's elements comprise the index into the `STATIC_STRING`s.

## JsSymbol Table

| `JsSymbol` hash `: u64` | Description (index into `JsString` table) `: usize` |
| ----------------------- | --------------------------------------------------- |
| 200                     | 0                                                   |
| ...                     | ...                                                 |

## JsBigInt Table

| Length in bytes `: u64` | Content |
| ----------------------- | ------- |
| 32                      | ...     |

## Shapes (Hidden classes) Table

### Unique Shapes

| `[[prototype]]` `: u32` (index into `JsObject` table) | property count `: u32` | key-value pairs |
| ----------------------------------------------------- | ---------------------- | --------------- |
|                                                       | 0                      |                 |
|                                                       | ...                    |                 |

### Shared Shapes

| previous `: u32` | flags | transition |
| ---------------- | ----- | ---------- |
| `MAX` (root)     | ...   | `x`        |

## JsObject Table

| length | Content |
| ------ | ------- |
| 200    | ...     |
| ...    | ...     |

## JsValue Encoding

type `: u8` (JsValue discriminant, Boolean, Null, etc) followed by the value if it exits.
If following the `JsValue` is an `JsString`, `JsSymbol`, `JsBigInt`, `JsObject` then the
following value will be an index into the appropriate tables.
