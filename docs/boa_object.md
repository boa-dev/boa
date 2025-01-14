# Boa Debug Object

The `$boa` object contains useful utilities that can be used to debug JavaScript in JavaScript.

It's injected into the context as global variable with the `--debug-object` command-line flag,
the object is separated into modules.

## Module `$boa.gc`

This module contains functions that are related the garbage collector. It currently has the `.collect()` method.

```JavaScript
$boa.gc.collect()
```

This force triggers the GC to scan the heap and collect garbage.

## Module `$boa.function`

In this module are untility functions related to execution and debugging function.

### Function `$boa.function.bytecode(func)`

This function returns the compiled bytecode of a function as a string,

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

### Function `$boa.function.trace(func, this, ...args)`

It only traces the specified function. If the specified function calls other functions,
their instructions aren't traced.

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

The `this` value can be changed as well as the arguments that are passed to the function.

### Function `$boa.function.traceable(func, mode)`

Marks a single function as traceable on all future executions of the function. Both useful to mark
several functions as traceable and to trace functions that suspend their execution (async functions,
generators, async generators).

#### Input

```Javascript
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

#### Output

```bash
1μs           RestParameterPop                                      <empty>
1μs           PushUndefined                                         undefined
2μs           Yield                                                 undefined
4μs           GetName                    0000: 'a'                  1
0μs           Yield                                                 1
1μs           GeneratorNext                                         undefined
1μs           Pop                                                   <empty>
15μs          GetName                    0001: 'b'                  2
1μs           Yield                                                 2
1μs           GeneratorNext                                         undefined
1μs           Pop                                                   <empty>
4μs           GetName                    0002: 'c'                  3
1μs           Yield                                                 3
```

## Function `$boa.function.flowgraph(func, options)`

It can be used to get the instruction flowgraph, like the command-line flag.
This works on the function level, allows getting the flow graph without
quiting the boa shell and adding the specified flags.

Besides the function it also takes an argument that, can be a string or an object.
If it is a string it represets the flowgraph format, otherwire if it's an object:

```JavaScript
// These are the defaults, if not specified.
{
    format: 'mermaid'
    direction: 'LeftRight' // or 'LR' shorthand.
}
```

Example:

```JavaScript
$boa.function.flowgraph(func, 'graphviz')
$boa.function.flowgraph(func, { format: 'mermaid', direction: 'TopBottom' })
```

## Module `$boa.object`

Contains utility functions for getting internal information about an object.

## Function `$boa.object.id(object)`

This function returns memory address of the given object, as a string.

Example:

```JavaScript
let o = { x: 10, y: 20 }
$boa.object.id(o)    // '0x7F5B3251B718'

// Geting the address of the $boa object in memory
$boa.object.id($boa) // '0x7F5B3251B5D8'
```

## Function `$boa.object.indexedStorageType(object)`

This function returns indexed storage type.

Example:

```JavaScript
let a = [1, 2]

$boa.object.indexedStorageType(a) // 'DenseI32'

a.push(0xdeadbeef)
$boa.object.indexedStorageType(a) // 'DenseI32'

a.push(0.5)
$boa.object.indexedStorageType(a) // 'DenseF64'

a.push("Hello")
$boa.object.indexedStorageType(a) // 'DenseElement'

a[100] = 100 // Make a hole
$boa.object.indexedStorageType(a) // 'SparseElement'
```

## Module `$boa.optimizer`

This modules contains getters and setters for enabling and disabling optimizations.

### Getter & Setter `$boa.optimizer.constantFolding`

This is and accessor property on the module, its getter returns `true` if enabled or `false` otherwise.
Its setter can be used to enable/disable the constant folding optimization.

```JavaScript
$boa.optimizer.constantFolding = true
$boa.optimizer.constantFolding // true
```

### Getter & Setter `$boa.optimizer.statistics`

This is an accessor property on the module, its getter returns `true` if enabled or `false` otherwise.
Its setter can be used to enable/disable optimization statistics, which are printed to `stdout`.

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

## Module `$boa.realm`

This module contains realm utilities to test cross-realm behaviour.

### `$boa.realm.create`

Creates a new realm with a new set of builtins and returns its global object.

```javascript
let global = $boa.realm.create();

Object != global.Object; // true
```

## Module `$boa.shape`

This module contains helpful functions for getting information about a shape of an object.

### Function `$boa.shape.id(object)`

Returns the pointer of the object's shape in memory as a string encoded in hexadecimal format.

```JavaScript
$boa.shape.id(Number) // '0x7FC35A073868'
$boa.shape.id({}) // '0x7FC35A046258'
```

### Function `$boa.shape.type(object)`

Returns the object's shape type.

```JavaScript
$boa.shape.type({x: 3}) // 'shared'
$boa.shape.type(Number) // 'unique'
```

### Function `$boa.shape.same(o1, o2)`

Returns `true` if both objects have the same shape.

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

## Module `$boa.limits`

This module contains utilities for changing runtime limits.

### Getter & Setter `$boa.limits.loop`

This is an accessor property on the module, its getter returns the loop iteration limit before an error is thrown.
Its setter can be used to set the loop iteration limit.

```javascript
$boa.limits.loop = 10;

while (true) {} // RuntimeLimit: Maximum loop iteration limit 10 exceeded
```

### Getter & Setter `$boa.limits.stack`

This is an accessor property on the module, its getter returns the value stack limit before an error is thrown.
Its setter can be used to set the recursion limit.

```javascript
$boa.limits.stack = 10;

function x() {
  return;
}
x(1, 2, 3, 4, 5, 6, 7, 8, 9, 10); // RuntimeLimit: exceeded maximum call stack length
```

### Getter & Setter `$boa.limits.recursion`

This is an accessor property on the module, its getter returns the recursion limit before an error is thrown.
Its setter can be used to set the recursion limit.

```javascript
$boa.limits.recursion = 100;

function x() {
  return x();
}
x(); // RuntimeLimit: Maximum recursion limit 100 exceeded
```

## Module `$boa.string`

This module contains helpful functions for getting information about a strings.

### Function `$boa.string.storage(str)`

Returns the string's inner storage type, if it's a well known string that is stored in the `STATIC_STRINGS` array in boa,
then `"static"` is returned, `"heap"` otherwise.

```JavaScript
$boa.string.storage("push")             // "static"
$boa.string.storage("specialFunction")  // "heap"
```

### Function `$boa.string.encoding(str)`

Returns the string's inner encoding of the string.

```JavaScript
$boa.string.encoding("Greeting") // "latin1"
$boa.string.encoding("挨拶")      // "utf16"
```

### Function `$boa.string.summary(str)`

Returns an object with a short summary of the of the given string.

```JavaScript
$boa.string.summary("Greeting") // { storage: "heap", encoding: "latin1" }
```
