# VM

## Architecture

![image](img/boa_architecture.drawio.png)

## Understanding the trace output

Once set up you can try some simple javascript in your test file. For example:

```js
let a = 1;
let b = 2;
```

Outputs:

```text
----------------------Compiled Output: '<main>'-----------------------
Location  Count   Opcode                     Operands

000001    0000    PushOne
000006    0001    DefInitLet                 0000: 'a'
000008    0002    PushInt8                   2
000013    0003    DefInitLet                 0001: 'b'

Literals:
    <empty>

Bindings:
    0000: a
    0001: b

Functions:
    <empty>


------------------------------------------ VM Start ------------------------------------------
Time          Opcode                     Operands                   Top Of Stack

386μs         PushOne                                               1
6μs           DefInitLet                 0000: 'a'                  <empty>
1μs           PushInt8                   2                          2
2μs           DefInitLet                 0001: 'b'                  <empty>

Stack:
    <empty>


undefined
```

The above output contains the following information:

- The bytecode and properties of the function that will be executed
  - `Compiled Output`: The bytecode.
    - `Location`: Location of the instruction (instructions are not the same size).
    - `Count`: Instruction count.
    - `Opcode`: Opcode name.
    - `Operands`: The operands of the opcode.
  - `Literals`: The literals used by the bytecode (like strings).
  - `Bindings`: Binding names used by the bytecode.
  - `Functions`: Function names use by the bytecode.
- The code being executed (marked by `Vm Start` or `Call Frame`).
  - `Time`: The amount of time that instruction took to execute.
  - `Opcode`: Opcode name.
  - `Operands`: The operands of the opcode.
  - `Top Of Stack`: The top element of the stack **after** execution of instruction.
- `Stack`: The trace of the stack after execution ends.
- The result of the execution (The top element of the stack, if the stack is empty then `undefined` is returned).

### Comparing ByteCode output

If you wanted another engine's bytecode output for the same JS, SpiderMonkey's bytecode output is the best to use. You can follow the setup [here](https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Introduction_to_the_JavaScript_shell). You will need to build from source because the pre-built binarys don't include the debugging utilities which we need.

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
