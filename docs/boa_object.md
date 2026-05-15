# Boa Debug Object

The `$boa` object exposes internal engine utilities for debugging, profiling, and inspecting JavaScript execution at runtime.

## Enabling the Debug Object

The `$boa` object is **not** available by default. Start Boa with the `--debug-object` flag:

```bash
boa --debug-object script.js
# or for the REPL:
boa --debug-object
```

Once enabled, `$boa` is available as a global variable with the following sub-modules:

| Module | Purpose |
|---|---|
| [`$boa.function`](#module-boafunction) | Bytecode inspection, execution tracing, flow graphs |
| [`$boa.object`](#module-boaobject) | Internal object representation & storage introspection |
| [`$boa.shape`](#module-boashape) | Shape (hidden class) inspection |
| [`$boa.optimizer`](#module-boaoptimizer) | Enable/disable compiler optimizations at runtime |
| [`$boa.gc`](#module-boagc) | Manual garbage collection control |
| [`$boa.realm`](#module-boarealm) | Cross-realm testing utilities |
| [`$boa.limits`](#module-boalimits) | Runtime limit getters/setters |
| [`$boa.string`](#module-boastring) | Internal string representation details |

---

## Module `$boa.gc`

This module contains functions that are related the garbage collector. It currently has the `.collect()` method.

```JavaScript
$boa.gc.collect()
```

This force triggers the GC to scan the heap and collect garbage.

---

## Module `$boa.function`

Utility functions for inspecting and debugging function execution.

### Function `$boa.function.bytecode(func)`

Prints the compiled bytecode of a function to stdout as a formatted dump. Shows opcodes, operands, constants, bindings, and exception handler tables. Returns `undefined`.

```JavaScript
>> function add(x, y) {
  return x + y
}
>> $boa.function.bytecode(add)
"
------------------------Compiled Output: 'add'------------------------
Location  Count    Handler    Opcode                     Operands

000000    0000      none      CreateMappedArgumentsObject
000001    0001      none      PutLexicalValue                           2: 0
000004    0002      none      GetArgument                           0
000006    0003      none      PutLexicalValue                           2: 1
000009    0004      none      GetArgument                           1
000011    0005      none      PutLexicalValue                           2: 2
000014    0006      none      PushDeclarativeEnvironment                           2
000016    0007      none      GetName                           0000: 'x'
000018    0008      none      GetName                           0001: 'y'
000020    0009      none      Add
000021    0010      none      SetAccumulatorFromStack
000022    0011      none      CheckReturn
000023    0012      none      Return
000024    0013      none      CheckReturn
000025    0014      none      Return

Constants:
    0000: [ENVIRONMENT] index: 1, bindings: 1
    0001: [ENVIRONMENT] index: 2, bindings: 3
    0002: [ENVIRONMENT] index: 3, bindings: 0

Bindings:
    0000: x
    0001: y

Handlers: <empty>
"
```

#### Edge Cases

- **Arrow functions** produce bytecode too, but without `CreateMappedArgumentsObject` (they don't have an arguments object).
- **Async functions** and **generators** produce bytecode with additional opcodes for suspend/resume (`Yield`, `GeneratorNext`, `AsyncAwait`).
- Passing a **non-function object** throws: `TypeError: expected an ordinary function object`.
- Passing a **primitive** (number, string, etc.) throws: `TypeError: expected object, got <type>`.

```JavaScript
>> $boa.function.bytecode(42)
Uncaught TypeError: expected object, got number
```

### Function `$boa.function.trace(func, this, ...args)`

Executes a function with instruction-level tracing enabled. Every bytecode instruction is logged to stdout with timing, opcode, operands, and accumulator value. Tracing is scoped to this single call — the function is not permanently marked as traceable.

```JavaScript
>> const add = (a, b) => a + b
>> $boa.function.trace(add, undefined, 1, 2)
5μs           DefInitArg                 0000: 'a'                  2
4μs           DefInitArg                 0001: 'b'                  <empty>
0μs           RestParameterPop                                      <empty>
3μs           GetName                    0000: 'a'                  1
1μs           GetName                    0001: 'b'                  2
2μs           Add                                                   3
1μs           Return                                                3
3
>>
```

The return value of `$boa.function.trace()` is the return value of the traced function itself.

#### Changing `this` and Arguments

```JavaScript
>> function greet() { return `Hello, ${this.name}`; }
>> $boa.function.trace(greet, { name: "Boa" })
// Traces the call with `this` bound to the object
"Hello, Boa"
```

#### Edge Cases

- If the traced function **throws**, the error propagates normally and the trace flag is cleaned up.
- Only **ordinary functions** can be traced; native built-in functions cannot.
- The `this` value and extra arguments are forwarded exactly as they would be in a direct call (see: [`$boa.function.traceable()`](#function-boafunctiontraceablefunc-mode) for persistent tracing).

### Function `$boa.function.traceable(func, mode)`

Marks a function as traceable on all future executions. Unlike `trace()` which handles a single invocation, `traceable()` permanently sets or clears the trace flag on the function's compiled code block.

This is essential for tracing functions that **suspend their execution** — async functions, generators, and async generators — where a single logical call may span multiple discrete execution phases across different microtask ticks.

#### Generator Example

```JavaScript
function* g() {
    yield 1;
    yield 2;
    yield 3;
}
$boa.function.traceable(g, true);
var iter = g();
iter.next();
iter.next();
iter.next();
```

Output:

```bash
1μs           RestParameterPop                                      <empty>
1μs           PushUndefined                                         undefined
2μs           Yield                                                 undefined
4μs           GetName                    0000: 'a'                  1
0μs           Yield                                                 1
1μs           GeneratorNext                                         undefined
1μs           Pop                                                   <empty>
15μs          GetName                    0000: 'a'                  2
1μs           Yield                                                 2
1μs           GeneratorNext                                         undefined
1μs           Pop                                                   <empty>
4μs           GetName                    0000: 'a'                  3
1μs           Yield                                                 3
```

#### Disabling Tracing

Pass `false` as the second argument to disable tracing on a previously-marked function:

```JavaScript
$boa.function.traceable(g, false); // No longer traced on next call
```

### Function `$boa.function.flowgraph(func, options)`

Returns the instruction flow graph of a function. This is equivalent to the `--flowgraph` CLI flag but operates at the function level, allowing inspection without restarting the shell.

#### Format Options

The second argument can be a string (format name) or an options object:

```JavaScript
// Defaults shown below
{
    format: 'mermaid',      // 'mermaid' | 'graphviz'
    direction: 'LeftRight'  // 'LeftRight' | 'RightLeft' | 'TopBottom' | 'BottomTop'
}
```

#### Examples

```JavaScript
// Mermaid format (default), left-to-right
$boa.function.flowgraph(myFunc);

// Graphviz format
$boa.function.flowgraph(myFunc, 'graphviz');

// Custom options
$boa.function.flowgraph(myFunc, { format: 'mermaid', direction: 'TopBottom' });
```

The flowgraph reveals the function's control flow structure — basic blocks, branches, loops, and exception handler entry points.

---

## Module `$boa.object`

Contains utility functions for inspecting the internal representation of objects.

### Function `$boa.object.id(object)`

Returns the memory address of the given object as a hex-formatted string. Useful for identity debugging — checking whether two references point to the same underlying object.

```JavaScript
let o = { x: 10, y: 20 }
$boa.object.id(o)    // '0x7F5B3251B718'

// Getting the address of the $boa object in memory
$boa.object.id($boa) // '0x7F5B3251B5D8'
```

#### Common Uses

- **Alias detection**: Verify that two variables reference the same object after operations like property cloning or prototype manipulation.
- **Object identity after operations**: Check whether built-in methods return the same object or create a new one.
- **Debugging GC issues**: Observe whether an object's address changes after garbage collection (it should not for live objects).

```JavaScript
// Check if two references are the same object
let a = { value: 1 };
let b = a;
$boa.object.id(a) === $boa.object.id(b);  // true

// After Object.assign, a new object is created
let c = Object.assign({}, a);
$boa.object.id(a) === $boa.object.id(c);  // false
```

### Function `$boa.object.indexedStorageType(object)`

Returns the internal indexed storage type of an object. Boa dynamically selects the most efficient storage representation based on the access pattern. This function reveals which representation is currently in use.

| Storage Type | Description |
|---|---|
| `DenseI32` | All elements are 32-bit integers; compact unboxed storage |
| `DenseF64` | Elements include floating-point values; unboxed double storage |
| `DenseElement` | Mixed-type elements including strings/objects; boxed values |
| `SparseElement` | Array has holes (`undefined` gaps); hash-map backed |
| `SparseProperty` | Non-default property descriptors present (non-writable, non-configurable) |

```JavaScript
let a = [1, 2];

$boa.object.indexedStorageType(a); // 'DenseI32'

a.push(0xdeadbeef);
$boa.object.indexedStorageType(a); // 'DenseI32'

a.push(0.5);
$boa.object.indexedStorageType(a); // 'DenseF64'

a.push("Hello");
$boa.object.indexedStorageType(a); // 'DenseElement'

a[100] = 100; // Make a hole
$boa.object.indexedStorageType(a); // 'SparseElement'

// Non-simple property descriptor (e.g., non-writable)
Object.defineProperty(a, 2, { value: 10, writable: false });
$boa.object.indexedStorageType(a); // 'SparseProperty'
```

---

## Module `$boa.optimizer`

This module contains getters and setters for enabling and disabling compiler optimizations at runtime. All properties are accessor properties.

### Getter & Setter `$boa.optimizer.constantFolding`

Enables or disables the constant folding optimization pass.

```JavaScript
$boa.optimizer.constantFolding = true
$boa.optimizer.constantFolding // true
```

### Getter & Setter `$boa.optimizer.statistics`

Enables or disables optimization statistics output to stdout.

```JavaScript
>> $boa.optimizer.constantFolding = true
>> $boa.optimizer.statistics = true
>> 1 + 1
Optimizer {
    constant folding: 1 run(s), 2 pass(es) (1 mutating, 1 checking)
}

2
>>
```

---

## Module `$boa.realm`

Realm utilities for testing cross-realm behavior. A realm represents a distinct JavaScript execution environment with its own global object and built-ins.

### `$boa.realm.create`

Creates a new ECMAScript realm and returns its global object. The new realm has its own set of built-ins, distinct from the calling realm's.

```javascript
let global = $boa.realm.create();

Object != global.Object; // true
```

This is useful for testing:
- Cross-realm property access
- `===` identity semantics across realms
- Built-in object brand checking

---

## Module `$boa.shape`

This module contains functions for inspecting the "shape" (hidden class) of an object. Boa uses shapes to optimize property access — objects with the same property layout share a shape, enabling fast inline caching.

### Function `$boa.shape.id(object)`

Returns the pointer of the object's shape in memory as a hex-formatted string.

```JavaScript
$boa.shape.id(Number) // '0x7FC35A073868'
$boa.shape.id({})     // '0x7FC35A046258'
```

If two objects share a shape, their shape IDs will be identical:

```JavaScript
let a = { x: 1 };
let b = { x: 2 };
$boa.shape.id(a) === $boa.shape.id(b); // true — same shape
b.y = 3;
$boa.shape.id(a) === $boa.shape.id(b); // false — different shapes now
```

### Function `$boa.shape.type(object)`

Returns the object's shape type, indicating whether the shape is shared or unique to the object.

```JavaScript
$boa.shape.type({x: 3}) // 'shared'
$boa.shape.type(Number) // 'unique'
```

A `'shared'` shape means the object's property layout is shared with other objects of the same structure. A `'unique'` shape means this object has a property layout not shared by any other object.

### Function `$boa.shape.same(o1, o2)`

Returns `true` if both objects have the same shape (identical property layout).

```JavaScript
// The values of the properties are not important!
let o1 = { x: 10 }
let o2 = {}
$boa.shape.same(o1, o2) // false

o2.x = 20
$boa.shape.same(o1, o2) // true

o2.y = 200
$boa.shape.same(o1, o2) // false
```

---

## Module `$boa.limits`

This module contains getters and setters for controlling runtime execution limits. These are useful for sandboxing untrusted code or debugging infinite loops.

### Getter & Setter `$boa.limits.loop`

Gets or sets the loop iteration limit. Throws `RuntimeLimit` when exceeded.

```javascript
$boa.limits.loop = 10;

while (true) {} // RuntimeLimit: Maximum loop iteration limit 10 exceeded
```

### Getter & Setter `$boa.limits.stack`

Gets or sets the value stack size limit. Throws `RuntimeLimit` when exceeded.

```javascript
$boa.limits.stack = 10;

function x() {
  return;
}
x(1, 2, 3, 4, 5, 6, 7, 8, 9, 10); // RuntimeLimit: exceeded maximum call stack length
```

### Getter & Setter `$boa.limits.recursion`

Gets or sets the recursion limit. Throws `RuntimeLimit` when exceeded.

```javascript
$boa.limits.recursion = 100;

function x() {
  return x();
}
x(); // RuntimeLimit: Maximum recursion limit 100 exceeded
```

### Getter & Setter `$boa.limits.backtrace`

Gets or sets the backtrace limit for error stack traces. Controls how many stack frames appear in thrown error backtraces.

```javascript
$boa.limits.backtrace = 100;

function x() {
  function y() {
    function z() {
      throw "Hello";
    }
    z();
  }
  y();
}
x();

// Uncaught "Hello"
//     at z (test.js:6:13)
//     at y (test.js:8:6)
//     at x (test.js:10:4)
//     at <main> (test.js:12:2)
```

---

## Module `$boa.string`

This module contains functions for inspecting the internal representation of strings. Boa strings can have different storage types and encodings depending on their content.

### Function `$boa.string.storage(str)`

Returns the string's internal storage type: `"static"` for well-known strings stored in the engine's `STATIC_STRINGS` array, or `"heap"` for dynamically allocated strings.

```JavaScript
$boa.string.storage("push")             // "static"
$boa.string.storage("specialFunction")  // "heap"
```

#### Performance Note

Accessing a `"static"` string has zero allocation cost — it references pre-compiled constant data. `"heap"` strings carry a memory allocation cost proportional to their length.

### Function `$boa.string.encoding(str)`

Returns the string's internal encoding. Boa uses `"latin1"` (one byte per character) for strings containing only Latin-1 characters, and `"utf16"` (two bytes per character) otherwise.

```JavaScript
$boa.string.encoding("Greeting") // "latin1"
$boa.string.encoding("挨拶")      // "utf16"
```

#### Performance Note

`"latin1"` strings are more memory-efficient (1 byte per char vs 2) and have faster iteration. Property keys using Latin-1 identifiers benefit from this optimization automatically.

### Function `$boa.string.summary(str)`

Returns an object with both storage and encoding properties for a quick overview:

```JavaScript
$boa.string.summary("Greeting") // { storage: "heap", encoding: "latin1" }
$boa.string.summary("push")     // { storage: "static", encoding: "latin1" }
```

---

## Common Use Cases

### 1. Diagnosing Performance

1. Enable optimizer statistics to verify optimization passes run:
   ```javascript
   $boa.optimizer.statistics = true;
   $boa.optimizer.constantFolding = true;
   ```
2. Use `$boa.function.bytecode()` on hot functions to inspect generated code quality.
3. Use `$boa.function.flowgraph()` to understand control flow complexity.
4. Compare `indexedStorageType()` of arrays at different points to ensure dense storage.

### 2. Debugging Object Identity

When objects behave unexpectedly, check if they are actually the same reference:

```javascript
// After cloning
let original = { data: [1, 2, 3] };
let shallow = Object.assign({}, original);
$boa.object.id(original) === $boa.object.id(shallow); // false (different objects)

// Did a method mutate in-place or return a new object?
let arr = [3, 2, 1];
let sorted = arr.toSorted();
$boa.object.id(arr) === $boa.object.id(sorted); // false (new array)
```

### 3. Sandboxing Untrusted Code

Set runtime limits before evaluating external scripts:

```javascript
$boa.limits.loop = 100000;
$boa.limits.recursion = 50;
$boa.limits.stack = 10000;

eval(untrustedCode); // guarded by limits
```

### 4. Understanding Shape Transitions

Track how shapes evolve as properties are added:

```javascript
let obj = {};
let shapeBefore = $boa.shape.id(obj);  // initial empty shape
obj.x = 1;
let shapeAfter = $boa.shape.id(obj);   // transitioned shape

// Objects with same property order share shapes
let obj2 = { x: 2 };
$boa.shape.id(obj) === $boa.shape.id(obj2); // true (same shape chain)
```

---

## Error Handling

All `$boa.*` functions validate their arguments and throw descriptive `TypeError` messages when the input is incorrect.

### Common error patterns

| Pattern | Example | Error |
|---|---|---|
| Missing argument | `$boa.function.bytecode()` | `TypeError: expected function argument` |
| Wrong type | `$boa.function.bytecode(42)` | `TypeError: expected object, got number` |
| Wrong object kind | `$boa.function.trace({})` | `TypeError: expected callable object` |
| Invalid format | `$boa.function.flowgraph(fn, 'svg')` | `TypeError: Unknown format type 'svg'` |

### Catching errors

```javascript
try {
    $boa.function.bytecode(42);
} catch (e) {
    console.log(e.message);
    // "expected object, got number"
}
```

---

## Troubleshooting

### `ReferenceError: $boa is not defined`

The `$boa` object is only available when the `--debug-object` CLI flag is passed:

```bash
boa --debug-object
```

Make sure you started Boa with this flag. It's **not** available in the default execution mode.

### No output from `$boa.function.bytecode()`

The bytecode is printed to **stdout**, not returned as a value. If you're running a script and seeing no output, ensure stdout is not being redirected. The function returns `undefined` — the bytecode text is a side-effect of the call.

### `flowgraph` returns empty output

Arrow functions and native built-in functions may not produce meaningful flowgraphs. Ensure the argument is a user-defined ordinary function (not a class constructor or native function).

### `trace()` produces no output after upgrading

The trace output format has changed across Boa versions. If running a newer version, the column layout and timing format may differ from the examples above. This is expected — the trace format is not stable across releases.

### Runtime limits seemingly ignored

Runtime limits apply per-context. If you create a new realm with `$boa.realm.create()`, that realm has its **own** default limits. Set limits after creating the realm:

```javascript
let global = $boa.realm.create();
// Limits must be set on the new realm via code evaluated in that realm
```

---

## Performance Considerations

1. **`bytecode()`** is cheap — it formats already-compiled data without re-parsing or re-compiling.
2. **`trace()`** and **`traceable()`** significantly slow down execution — each bytecode instruction incurs formatting and I/O overhead. Do not use in production or with hot loops.
3. **`flowgraph()`** allocates a control flow graph structure — acceptable for ad-hoc inspection but not for frequent calls.
4. **`id()`** is a simple pointer cast — O(1) and negligible overhead.
5. **Runtime limits** (`loop`, `recursion`, `stack`) are checked on every iteration/recursion step and add measurable overhead in tight loops. Set them generously in production and only tighten during debugging.
