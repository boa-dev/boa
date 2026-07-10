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

Each major AST category — such as `expression`, `statement`, and `declaration` - has corresponding compilation logic that emits one or more bytecode instructions.

The compiler operates in a **single pass** over the AST, generating instructions as it traverses nodes.

## Execution Model

Boa uses a register-based bytecode model. Instructions operate on virtual registers instead of stack-based operations.

If you see in here the struct `ByteCompiler` contains the register allocation field.

```rust
pub(crate) register_allocator: RegisterAllocator,
```

Registers are allocated during the compilation process via `RegisterAllocator`.

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

## Identifier and Symbol Handling

The compiler relies on a string interner to store and resolve identifiers efficiently:

```rust
pub(crate) interner: &'ctx mut Interner
```

Identifiers are interned and referenced by symbols (`Sym`) rather than raw strings. This avoids repeated heap allocations for the same identifier and allows O(1) comparisons by symbol index.

## Scope and Binding Resolution

The compiler maintains two separate scopes during AST traversal:

```rust
/// The current variable scope.
pub(crate) variable_scope: Scope,

/// The current lexical scope.
pub(crate) lexical_scope: Scope,
```

- `variable_scope` tracks `var` declared bindings, which are function-scoped.
- `lexical_scope` tracks `let` and `const` declared bindings, which are block-scoped.

Bindings are resolved using `BindingLocator` structures, and scope information is embedded into the resulting `CodeBlock` for runtime environment resolution.

## Constants and Literal Collections

During compilation, the bytecompiler collects values that cannot be embedded directly into instructions - such as strings, numbers, and scope objects - into a `constants` vector in the `CodeBlock`.

Instead of embedding these values inline, the compiler stores them in `constants` and emits the index as the instruction operand. The VM then looks up the value by index at runtime.

For example, when pushing a new scope, the compiler stores it in `constants` and emits the index:

```rust
let index = self.constants.len() as u32;
self.constants.push(Constant::Scope(scope.clone()));
```

This keeps instructions compact and uniform in size, while still allowing the VM to access arbitrarily complex values.

## Jump Patching and Control Flow

When compiling control flow constructs like `if`, `while`, or `for`, the compiler needs to emit jump instructions before the jump target is known - because the target code hasn't been compiled yet.

To handle this, the compiler emits the jump with a placeholder address, then comes back and fills in the real address once the target location is known. This process is called jump patching.

The `jump_control.rs` module manages jump targets, labels, and the bookkeeping required to patch jumps correctly. This includes:

- Forward jumps (e.g., jumping past an if block)
- Loop back-edges (e.g., jumping back to the top of a while)
- Break and continue targets

## Async and Generator Functions

Async functions and generators are **not** lowered to explicit state machines at the AST level. Instead, the compiler emits handler metadata and suspend/resume points that allow the VM to pause and continue execution across `await` expressions and `yield` statements.

The presence of an active async handler is tracked via:

```rust
pub(crate) async_handler: Option,
```

When set, this indicates the index of the handler responsible for managing suspension. The VM uses this information at runtime to correctly propagate values into and out of suspended frames.

## Relationship to the Virtual Machine

The bytecompiler produces `CodeBlock` structures which contain the instructions and metadata required for execution.

The VM interprets these instructions at runtime. For execution details, see the VM documentation (`vm.md`).
