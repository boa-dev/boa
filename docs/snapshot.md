# Snapshot File format

This document describes the binary file format of the boa snapshot files.

## Header

The header composes the first part of the snapshot.

| Field                 | Description                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| signature `: [u8; 4]` | This is used to quickly check if this file is a snapshot file (`.boa`)                                       |
| guid                  | Guid generated in compile time and backed into the binary, that is used to check if snapshot is compatibile. |
| checksum              | Checksum that is used to check that the snapshot is not corrupted.                                           |

## Internal Reference Map


## JsValue Encoding

type `: u8` (JsValue discriminant, Boolean, Null, etc) followed by the value if it applied for the type (`Null` and `Undefined` does not have a value part).
If following the `JsValue` is an `JsString`, `JsSymbol`, `JsBigInt`, `JsObject` then the
following value will be an index into the appropriate tables.
