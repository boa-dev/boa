# VM (Beta)

## State Of Play

By default Boa does not use the VM branch; execution is done via walking the AST. This allows us to work on the VM branch whilst not interrupting any progress made on AST execution.

You can interpret bytecode by passing the "vm" flag (see below). The diagram below should illustrate how things work today (Jan 2021).

![image](img/boa_architecture.svg)

## Enabling ByteCode interpretation

You need to enable this via a feature flag. If using VSCode you can run `Cargo Run (VM)`. If using the command line you can pass `cargo run --features vm ../tests/js/test.js` from within the boa_cli folder.

## Understanding the output

Once set up you should you can try some very simple javascript in your test file. For example:

```js
let a = 1;
let b = 2;
```

Should output:

```
VM start up time: 0μs
Time       Instr                Top Of Stack

27μs       DefLet(0)             <empty>
3μs        One                   <empty>
35μs       InitLexical(0)        1      0x7f727f41d0c0
18μs       DefLet(1)             1      0x7f727f41d0c0
4μs        Int32(2)              1      0x7f727f41d0c0
19μs       InitLexical(1)        2      0x7f727f41d0d8

Pool
0          "a" 0x7f727f41d120
1          "b" 0x7f727f41d138

Stack
0          1 0x7f727f41d0c0
1          2 0x7f727f41d0d8

2
```

The above will output 3 sections: Instructions, pool and Stack. We can go through each one in detail:

### Instruction

This shows each instruction being executed and how long it took. This is useful for us to see if a particular instruction is taking too long.
Then you have the instruction itself and its operand. Last you have what is on the top of the stack **before** the instruction is executed, followed by the memory address of that same value. We show the memory address to identify if 2 values are the same or different.

### Pool

JSValues can live on the pool, which acts as our heap. Instructions often have an index of where on the pool it refers to a value.
You can use these values to match up with the instructions above. For e.g (using the above output) `DefLet(0)` means take the value off the pool at index `0`, which is `a` and define it in the current scope.

### Stack

The stack view shows what the stack looks like for the JS executed.
Using the above output as an exmaple, after `One` has been executed the next instruction (`InitLexical(0)`) has a `1` on the top of the stack. This is because `One` puts `1` on the stack.

### Comparing ByteCode output

If you wanted another engine's bytecode output for the same JS, SpiderMonkey's bytecode output is the best to use. You can follow the setup [here](https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Introduction_to_the_JavaScript_shell). You will need to build from source because the pre-built binarys don't include the debugging utilities which we need.

I named the binary `js_shell` as `js` conflicts with NodeJS. Once up and running you should be able to use `js_shell -f tests/js/test.js`. You will get no output to begin with, this is because you need to run `dis()` or `dis([func])` in the code. Once you've done that you should get some output like so:

```
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
