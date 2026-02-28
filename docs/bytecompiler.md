# Byte Compiler

## Architecture

The bytecompiler is responsible for lowering ECMAScript AST nodes (from `boa_ast`) into executable bytecode for the virtual machine.

It traverses the parsed AST and emits instructions into a `CodeBlock` using the `ByteCodeEmitter`. The resulting bytecode is later executed by the VM.

During compilation, the bytecompiler performs several tasks:

- Instruction emission
- Register allocation
- Lexical and variable scope management
- Binding resolution
- Control-flow generation (jumps and exception handlers)
- Collection of constants and literals

The produced `CodeBlock` contains the instructions, literal tables, bindings, and handler metadata required for execution.

The flow of the compilation process is as follows:

```bash
AST → ByteCompiler → CodeBlock → VM
```

## Compilation Model

The bytecompiler performs a lowering step from high-level ECMAScript AST nodes into a lower-level bytecode instruction set understood by the VM.

Each major AST category - such as `expression`, `statement`, and `declaration` — has corresponding compilation logic that emits one or more bytecode instructions.

The compiler operates in a single pass over the AST, generating instructions as it traverses nodes.

## Module Organization

The bytecompiler module is structured according to ECMAScript syntactic categories:

- `expression/` - compilation of expression nodes (e.g., `Binary`, `Unary`)
- `statement/` - compilation of statements and control flow (e.g., `If`, `For`, `While`)
- `declaration/` - handling of variable and function declarations (including patterns)

Supporting modules include:

- `register.rs` - register allocation logic
- `jump_control.rs` - management of jump targets and control flow
- `function.rs` - function compilation
- `class.rs` - class compilation
- `module.rs` - ECMAScript module compilation
- `env/` - management of environments and scope-related data

The `mod.rs` file coordinates the overall compilation process.

## Relationship to the Virtual Machine

The bytecompiler produces `CodeBlock` structures which contain the instructions and metadata required for execution.

The VM interprets these instructions at runtime. For execution details, see the VM documentation (`vm.md`).
