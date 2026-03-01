# VM

## Architecture

![image](img/boa_architecture.png)

When Boa runs JavaScript, the source code goes through a pipeline:

```text
Source Code → Parser → AST → ByteCompiler → CodeBlock → VM → Result
```

The parser produces an AST, the `ByteCompiler` compiles it into bytecode stored in a `CodeBlock`,
and then the VM executes that bytecode. Let's dig into how each piece works.

## CodeBlock

Every function (or script/module) the `ByteCompiler` processes gets its own `CodeBlock`. Think
of it as the compiled form of a function — it has the bytecode, a pool of constants, info about
bindings, exception handlers, and some metadata.

Here's a simplified view of what's inside:

```rust
struct CodeBlock {
    bytecode: ByteCode,                     // the actual instruction bytes
    constants: ThinVec<Constant>,           // strings, nested functions, bigints, scopes
    bindings: Box<[BindingLocator]>,        // variable binding locators
    handlers: ThinVec<Handler>,             // try/catch/finally handler ranges
    ic: Box<[InlineCache]>,                 // inline caches for fast property access
    register_count: u32,                    // how many local registers this function uses
    length: u32,                            // the .length property of the function
    parameter_length: u32,                  // number of formal parameters
    this_mode: ThisMode,                    // Global, Strict, or Lexical
    flags: Cell<CodeBlockFlags>,            // strict mode, async, generator, etc.
    source_info: SourceInfo,                // source maps and function name
}
```

Constants are things the bytecode references by index:

```rust
enum Constant {
    String(JsString),        // property names, string literals
    Function(Gc<CodeBlock>), // nested function declarations/expressions
    BigInt(JsBigInt),        // bigint literals
    Scope(Scope),            // declarative or function scopes
}
```

The `flags` field uses bitflags to track things like whether the function is `strict`, `async`,
a `generator`, a class constructor, a derived constructor, whether it has a prototype property,
and so on. If you've built with the `trace` feature, there's also a `TRACEABLE` flag per
function.

## Bytecode and Opcodes

Instructions live in a `ByteCode` struct, which is just a `Box<[u8]>`. Each instruction starts
with a one-byte opcode, followed by its operands:

```text
┌────────┬──────────┬──────────┬───┐
│ opcode │ operand1 │ operand2 │...│
│ (1 B)  │ (varies) │ (varies) │   │
└────────┴──────────┴──────────┴───┘
```

Most operands use `VaryingOperand`, which picks the smallest encoding that fits the value — `u8`
if it's ≤ 255, `u16` if ≤ 65535, otherwise `u32`. This keeps bytecode compact in the common case.

All opcodes are defined in a single `generate_opcodes!` macro invocation. The macro generates
quite a lot from one definition: the `Opcode` enum, the `Instruction` enum (decoded form with
typed fields), a dispatch table of handler functions, and emit methods on `ByteCodeEmitter`.

Each opcode's behavior is implemented via the `Operation` trait:

```rust
trait Operation {
    const NAME: &'static str;
    const INSTRUCTION: &'static str;
    const COST: u8;  // used for budget-based async execution
}
```

There are over 100 opcodes, grouped roughly into these categories:

- **push/pop** — push constants, pop values (`PushZero`, `PushInt8`, `PushLiteral`, `Pop`, etc.)
- **binary ops** — arithmetic and comparison (`Add`, `Sub`, `Mul`, `Eq`, `LessThan`, etc.)
- **unary ops** — `Neg`, `Pos`, `BitNot`, `LogicalNot`, `TypeOf`, `Inc`, `Dec`
- **control flow** — `Jump`, `JumpIfTrue`, `JumpIfFalse`, `Return`
- **call/new** — `Call`, `CallEval`, `New`, `SuperCall`
- **get/set** — `GetName`, `GetPropertyByName`, `SetName`, `SetPropertyByName`
- **define/delete** — `DefVar`, `DefineOwnPropertyByName`, `DeletePropertyByValue`
- **environment** — `PushScope`, `PopScope`, `PushObjectEnvironment`
- **generator/async** — `Generator`, `GeneratorYield`, `Await`
- **iteration** — `GetIterator`, `IteratorNext`, `IteratorDone`
- **copy** — `Move`, `SetRegisterFromAccumulator`

## The Stack

The VM uses a single `Vec<JsValue>` as its value stack, shared across all call frames. Let's
look at how it's organized.

When a function gets called, its portion of the stack looks like this:

```text
                     Setup by the caller
  ┌─────────────────────────────────────────────────────────┐ ┌───── register pointer (rp)
  ▼                                                         ▼ ▼
| -(2+N): this | -(1+N): func | -N: arg1 | ... | -1: argN | 0: reg1 | ... | K: regK |
  ▲                              ▲   ▲                      ▲   ▲                    ▲
  └──────────────────────────────┘   └──────────────────────┘   └────────────────────┘
        function prologue                   arguments             Setup by the callee
  ▲
  └─ Frame pointer (fp)
```

The first two slots are always `this` and the function object — that's the *prologue*. Then come
the arguments. After that, the callee allocates `register_count` slots for its local registers.
The register pointer (`rp`) sits right at the boundary, so registers are addressed as simple
offsets from `rp`.

Let's see a concrete example. Given:

```javascript
function x(a) {
}
function y(b, c) {
    return x(b + c)
}

y(1, 2)
```

During the call to `x`, the stack looks like:

```text
    caller prologue    caller arguments   callee prologue   callee arguments
  ┌─────────────────┐   ┌─────────┐   ┌─────────────────┐  ┌──────┐
  ▼                 ▼   ▼         ▼   │                 ▼  ▼      ▼
| 0: undefined | 1: y | 2: 1 | 3: 2 | 4: undefined | 5: x | 6:  3 |
▲                                   ▲                             ▲
│       caller register pointer ────┤                             │
│                                   │                 callee register pointer
│                             callee frame pointer
│
└─────  caller frame pointer
```

The calling convention works like this:

1. The caller pushes `this` and the function object (prologue), then pushes the arguments.
2. The caller creates a `CallFrame` and calls `push_frame()`.
3. `push_frame()` sets `rp` to the current stack top and extends the stack by `register_count`
   slots, all initialized to `undefined`.
4. When the function returns, the stack gets truncated back to the caller's frame pointer.

## Call Frames

A `CallFrame` holds all the execution state for a single function invocation:

```rust
struct CallFrame {
    code_block: Gc<CodeBlock>,     // the function's compiled bytecode
    pc: u32,                       // program counter (offset into bytecode)
    rp: u32,                       // register pointer (start of registers in the stack)
    argument_count: u32,           // how many arguments were passed
    env_fp: u32,                   // environment frame pointer (for cleanup on exception)
    environments: EnvironmentStack, // lexical environment chain
    realm: Realm,                  // the realm this function runs in
    iterators: ThinVec<IteratorRecord>,  // open iterators (need closing on abrupt completion)
    binding_stack: Vec<BindingLocator>,  // bindings being updated
    loop_iteration_count: u64,     // tracks loop iterations for runtime limits
    active_runnable: Option<ActiveRunnable>,  // owning Script or Module
    flags: CallFrameFlags,         // EXIT_EARLY, CONSTRUCT, etc.
}
```

The `CallFrame` can figure out where everything lives on the stack using just `rp` and
`argument_count`:

- `frame_pointer() = rp - argument_count - 2` (start of prologue)
- `this_index() = rp - argument_count - 2`
- `function_index() = rp - argument_count - 1`
- `arguments_range() = (rp - argument_count)..rp`

There are a few flags worth knowing about:

- **`EXIT_EARLY`** — when we return from this frame, stop the VM entirely and return to the
  Rust caller, instead of continuing with the parent frame.
- **`CONSTRUCT`** — this frame was created via `[[Construct]]` (the `new` keyword).
- **`REGISTERS_ALREADY_PUSHED`** — used when resuming a generator. The register area is
  already populated from the previous execution, so `push_frame()` skips allocating registers.
- **`THIS_VALUE_CACHED`** — the `this` value has been resolved and cached.

### How frames get pushed and popped

When `push_frame()` is called, the current frame gets swapped out and pushed onto a `frames`
vector. The new frame becomes `vm.frame`. On `pop_frame()`, the reverse happens — the last
frame on the vector gets swapped back in.

For async/generator functions, the first few registers are reserved:

- Registers 0, 1, 2: promise capability (promise object, resolve fn, reject fn)
- Register 3: async generator object (when applicable)

## Execution Loop

The core loop lives in `Context::run()`. It's a straightforward fetch-decode-execute loop:

```rust
fn run(&mut self) -> CompletionRecord {
    while let Some(byte) = bytecode.get(frame.pc) {
        let opcode = Opcode::decode(*byte);

        match self.execute_one(Self::execute_bytecode_instruction, opcode) {
            ControlFlow::Continue(()) => {}
            ControlFlow::Break(value) => return value,
        }
    }
}
```

Dispatch uses a static handler table — `OPCODE_HANDLERS` is an array of 256 function pointers,
one per possible opcode byte. Each handler decodes the operands from the bytecode, advances `pc`
past them, runs the operation, and returns a `ControlFlow`.

For async contexts (like module evaluation), there's `run_async_with_budget()`. Each opcode has
a `COST`, and the budget gets decremented on every instruction. When it hits zero, the function
yields back to the async executor with `yield_now().await`, preventing a long-running script from
starving other tasks.

### Tracing

If you build with the `trace` feature, the VM can print each instruction as it runs. You can
enable it globally (`vm.trace = true`) or per-function (`code_block.set_traceable(true)`). The
output looks like:

```text
Time          Opcode                     Operands                   Top Of Stack
6μs           PushOne                    dst:0                      1
7μs           PutLexicalValue            src:0, binding_index:0     <empty>
```

## Exception Handling

Exception handlers are compiled as `Handler` structs in the `CodeBlock`:

```rust
struct Handler {
    start: u32,              // start of the protected range
    end: u32,                // handler address (where to jump on catch)
    environment_count: u32,  // environments to preserve when unwinding
}
```

So for a try/catch like:

```javascript
try {
    // bytecode at pc 10..50
    riskyOperation();
} catch (e) {
    // handler at pc 50
    handleError(e);
}
```

...we'd get `Handler { start: 10, end: 50, environment_count: N }`.

When an exception is thrown, here's what happens:

1. The VM captures a backtrace from the shadow stack.
2. It checks if the error is catchable. Non-catchable errors (like exceeding runtime limits) skip
   all handler logic and immediately unwind everything.
3. For catchable errors, `find_handler(pc)` searches the handlers in reverse order (innermost
   first) for one whose `[start, end)` range contains the current `pc`.
4. If a handler is found in the current frame: set `pc = handler.end`, truncate environments
   to `env_fp + handler.environment_count`, store the exception in `vm.pending_exception`,
   and continue. Bytecode can retrieve it later with the `Exception` opcode.
5. If no handler in the current frame: check if the frame has `EXIT_EARLY` set (if so, return
   the error to the Rust caller). Otherwise, pop the frame and try the parent frame's handlers.
   Keep unwinding until we find a handler or run out of frames.

## Generators and Async Functions

Generator functions use `GeneratorResumeKind` to track how they're being resumed:

```rust
enum GeneratorResumeKind {
    Normal = 0,  // .next(value)
    Throw = 1,   // .throw(error)
    Return = 2,  // .return(value)
}
```

When a generator yields (via the `GeneratorYield` opcode), the current frame gets popped and
its stack portion is saved. When `.next()` is called again, the saved `CallFrame` — with the
`REGISTERS_ALREADY_PUSHED` flag set — gets pushed back. Since the registers are already there
from the previous run, `push_frame()` skips the allocation step and execution picks up right
where it left off.

Async functions work similarly but use reserved register slots for their promise machinery
(registers 0-2 hold the promise, resolve, and reject). The `Await` opcode suspends execution
like `Yield`, but resumption is driven by promise settlement instead of an explicit `.next()`
call. Async generators combine both: registers 0-2 for the promise, register 3 for the async
generator object.

## Inline Caching

Property access can be expensive — the VM has to walk the shape chain each time. To speed things
up, each `GetPropertyByName` and `SetPropertyByName` instruction references an inline cache
entry (via `ic_index`). The cache stores the property name and the last-seen object shape. If
the object's shape matches the cached one, we already know the property's slot index and can
skip the full lookup. On a miss, we do the lookup and update the cache.

## Runtime Limits

Embedders can constrain the VM through `RuntimeLimits`. By default we allow 512 levels of
recursion, a stack size of 1024, and practically unlimited loop iterations. These get checked
at call boundaries and inside loops. Exceeding any limit throws a non-catchable
`RuntimeLimitError` that bypasses all exception handlers.

## Understanding the trace output

Once set up you can try some simple javascript in your test file. For example:

```js
let a = 1;
let b = 2;
```

Outputs:

```text
----------------------Compiled Output: '<main>'-----------------------
Location  Count    Handler    Opcode                     Operands

000000    0000      none      PushOne
000001    0001      none      PutLexicalValue            0000: 'a'
000006    0002      none      PushInt8                   2
000008    0003      none      PutLexicalValue            0001: 'b'
000013    0004      none      Return

Literals:
    <empty>

Bindings:
    0000: a
    0001: b

Functions:
    <empty>

Handlers:
    <empty>


----------------------------------------- Call Frame -----------------------------------------
Time          Opcode                     Operands                   Top Of Stack

6μs           PushOne                                               1
7μs           PutLexicalValue            0000: 'a'                  <empty>
0μs           PushInt8                   2                          2
1μs           PutLexicalValue            0001: 'b'                  <empty>
0μs           Return                                                <empty>

Stack:
    <empty>


undefined
```

The above output contains the following information:

- The bytecode and properties of the function that will be executed
  - `Compiled Output`: The bytecode.
    - `Location`: Location of the instruction (instructions are not the same size).
    - `Count`: Instruction count.
    - `Handler`: Exception handler, if the instruction throws an exception, which handler is responsible for that instruction and where it would jump. Additionally `>` denotes the beginning of a handler and `<` the end.
    - `Opcode`: Opcode name.
    - `Operands`: The operands of the opcode.
  - `Literals`: The literals used by the bytecode (like strings).
  - `Bindings`: Binding names used by the bytecode.
  - `Functions`: Function names use by the bytecode.
  - `Handlers`: Exception handlers use by the bytecode, it contains how many values should be on the stack and environments (relative to `CallFrame`'s frame pointers).
- The code being executed (marked by `Vm Start` or `Call Frame`).
  - `Time`: The amount of time that instruction took to execute.
  - `Opcode`: Opcode name.
  - `Operands`: The operands of the opcode.
  - `Top Of Stack`: The top element of the stack **after** execution of instruction.
- `Stack`: The trace of the stack after execution ends.
- The result of the execution (The top element of the stack, if the stack is empty then `undefined` is returned).

### Comparing ByteCode output

If you wanted another engine's bytecode output for the same JS, SpiderMonkey's bytecode output is the best to use. You can follow the setup [here](https://udn.realityripple.com/docs/Mozilla/Projects/SpiderMonkey/Introduction_to_the_JavaScript_shell). You will need to build from source because the pre-built binarys don't include the debugging utilities which we need.

I named the binary `js_shell` as `js` conflicts with NodeJS. Once up and running you should be able to use `js_shell -f tests/js/test.js`. You will get no output to begin with, this is because you need to run `dis()` or `dis([func])` in the code. Once you've done that you should get some output like so:

```text
loc     op
-----   --
00000:  GlobalOrEvalDeclInstantiation 0 #
main:
00005:  One                             # 1
00006:  InitGLexical "a"                # 1
00011:  Pop                             #
00012:  Int8 2                          # 2
00014:  InitGLexical "b"                # 2
00019:  Pop                             #
00020:  GetGName "dis"                  # dis
```
