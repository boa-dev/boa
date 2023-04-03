# Boa Debug Object

The `$boa` object that contains useful utilities that can be used to debug JavaScript in JavaScript.

It becomes available with the `--debug-object` command-line flag.
It's injected into the context as global variable, the object is separated into modules.

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
Location  Count   Opcode                     Operands

000000    0000    DefInitArg                 0000: 'a'
000005    0001    DefInitArg                 0001: 'b'
000010    0002    RestParameterPop
000011    0003    GetName                    0000: 'a'
000016    0004    GetName                    0001: 'b'
000021    0005    Add
000022    0006    Return
000023    0007    PushUndefined
000024    0008    Return

Literals:
    <empty>

Bindings:
    0000: a
    0001: b

Functions:
    <empty>
"
>>
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

```JavaScript
$boa.function.trace(func, this, ...args)
```

## Function `$boa.function.flowgraph(func, options)`

It can be used to get the instruction flowgraph, like the cli flag. This works on the
function level, allows getting the flow graph without quiting the boa shell and
adding the specified flags.

Besides the function it also takes an argument that, can be a string or an object.
If it is a string it represets the flowgraph format, otherwire if it's an object

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
