# Debugging

There are multiple ways to debug what Boa is doing. Or maybe you just want to know how it works under the hood. Or even test some JavaScript.

One way to do so is to create a file in the root of the repository. For example `test.js`. Then execute `cargo run -- test.js` to run the file with boa.

You can also run boa interactively by simply calling `cargo run` without any arguments to start a shell to execute JS.

These are added in order of how the code is read:

## Tokens

The first thing boa will do is generate tokens from source code.  
If the token generation is wrong the rest of the operation will be wrong, this is usually a good starting place.

Navigate to `parser_expr` in [lib.rs](../src/lib/lib.rs#L48) and add `dbg!(&tokens);` just below tokens to see the array of token output. You code should look like this:

```rust
    let mut lexer = Lexer::new(src);
    lexer.lex().expect("lexing failed");
    let tokens = lexer.tokens;
    dbg!(&tokens);
    ...
```

Seeing the order of tokens can be a big help to understanding what the parser is working with.

## Expressions

Assuming the tokens looks fine, the next step is to see the AST.
You can output the expressions in [forward](../src/lib/lib.rs#L57), add `dbg!(&expr);`
This will print out the entire parse tree.

## Execution

Once the tree has been generated [exec](../src/lib/exec.rs#L66) will begin to run through each expression. If the tokens and tree looks fine, you can start looking here.  
I usually just add `dbg!()` in the relevent places to see what the output is at the time.

## Debugger

### VS Code Debugger

The quickest way to get debugging is to re-open the workspace in the container (using the Dockerfile provided). This is using the [Remote Containers plugin](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers). Once inside make sure you have the CodeLLDB extension installed and add breakpoints.

### LLDB Manually

You can also use rust-lldb.  
The `Dockerfile` already has this enabled, you should be able to use that environment to run your code.  
`rust-lldb ./target/debug/boa [arguments]`
