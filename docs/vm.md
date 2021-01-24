# VM (Beta)

## State Of Play

By default Boa does not use the VM branch; execution is done via walking the AST. This allows us to work on the VM branch whilst not interrupting any progress made on AST execution.

You can interpret bytecode by passing the "vm" flag (see below). The diagram below should illustrate how things work today (Jan 2021).

![image](img/boa_architecture.svg)

## Enabling ByteCode interpretation

You need to enable this via a feature flag. If using VSCode you can run `Cargo Run (VM)`. If using the command line you can pass `cargo run --features vm ../tests/js/test.js` from within the boa_cli folder.

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
