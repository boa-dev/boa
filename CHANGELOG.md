# CHANGELOG

# [# 0.7.0 (Master)](https://github.com/jasonwilliams/boa/compare/v0.6.0...HEAD)

`v0.7.0` brings a REPL, thanks @JohnDoneth!  
This is now the default behaviour of Boa, so running Boa without a file argument will bring you into a javascript shell.  
Tests have also been moved to their own files, we had a lot of tests in some modules so it was time to separate.

Feature enhancements:

- [FEATURE #253](https://github.com/jasonwilliams/boa/pull/253):
  Implement Array.isArray (@cisen)
- [FEATURE](https://github.com/jasonwilliams/boa/commit/edab5ca6cc10d13265f82fa4bc05d6b432a362fc)
  Switch to normal output instead of debugged output (stdout/stdout) (@jasonwilliams)
- [FEATURE #258](https://github.com/jasonwilliams/boa/pull/258):
  Moved test modules to their own files (@Razican)
- [FEATURE #267](https://github.com/jasonwilliams/boa/pull/267):
  Add print & REPL functionality to CLI (@JohnDoneth)
- [FEATURE #268](https://github.com/jasonwilliams/boa/pull/268):
  Addition of forEach() (@jasonwilliams) (@xSke)
- [FEATURE #262](https://github.com/jasonwilliams/boa/pull/262):
  Implement Array.prototype.filter (@Nickforall)

Bug fixes:

- [BUG #249](https://github.com/jasonwilliams/boa/pull/249):
  fix(parser): handle trailing comma in object literals (@gomesalexandre)
- [BUG #244](https://github.com/jasonwilliams/boa/pull/244):
  Fixed more Lexer Panics (@adumbidiot)
- [BUG #256](https://github.com/jasonwilliams/boa/pull/256):
  Fixed comments lexing (@Razican)
- [BUG #251](https://github.com/jasonwilliams/boa/issues/251):
  Fixed empty returns (@Razican)

# [# 0.6.0 (2020-02-14) - Migration to Workspace Architecture + lexer/parser improvements](https://github.com/jasonwilliams/boa/compare/v0.5.1...v0.6.0)

The lexer has had several fixes in this release, including how it parses numbers, scientific notation should be improved.  
On top of that the lexer no longer panics on errors including Syntax Errors (thanks @adumbidiot), instead you get some output on where the error happened.

## Moving to a workspace architecture

Boa offers both a CLI and a library, initially these were all in the same binary. The downside is
those who want to embed boa as-is end up with all of the command-line dependencies.  
So the time has come to separate out the two, this is normal procedure, this should be analogous to ripgrep
and the regex crate.  
Cargo has great support for workspaces, so this shouldn't be an issue.

## Benchmarks

We now have [benchmarks which run against master](https://jasonwilliams.github.io/boa/dev/bench)!  
Thanks to Github Actions these will run automatically a commit is merged.

Feature enhancements:

- [FEATURE #218](https://github.com/jasonwilliams/boa/pull/218):
  Implement Array.prototype.toString (@cisen)
- [FEATURE #216](https://github.com/jasonwilliams/boa/commit/85e9a3526105a600358bd53811e2b022987c6fc8):
  Keep accepting new array elements after spread.
- [FEATURE #220](https://github.com/jasonwilliams/boa/pull/220):
  Documentation updates. (@croraf)
- [FEATURE #226](https://github.com/jasonwilliams/boa/pull/226):
  add parser benchmark for expressions. (@jasonwilliams)
- [FEATURE #217](https://github.com/jasonwilliams/boa/pull/217):
  String.prototype.replace() implemented
- [FEATURE #247](https://github.com/jasonwilliams/boa/pull/247):
  Moved to a workspace architecture (@Razican)

Bug fixes:

- [BUG #222](https://github.com/jasonwilliams/boa/pull/222):
  Fixed clippy errors (@IovoslavIovchev)
- [BUG #228](https://github.com/jasonwilliams/boa/pull/228):
  [lexer: single-line-comment] Fix bug when single line comment is last line of file (@croraf)
- [BUG #229](https://github.com/jasonwilliams/boa/pull/229):
  Replace error throwing with panic in "Lexer::next()" (@croraf)
- [BUG #232/BUG #238](https://github.com/jasonwilliams/boa/pull/232):
  Clippy checking has been scaled right back to just Perf and Style (@jasonwilliams)
- [BUG #227](https://github.com/jasonwilliams/boa/pull/227):
  Array.prototype.toString should be called by ES value (@cisen)
- [BUG #242](https://github.com/jasonwilliams/boa/pull/242):
  Fixed some panics in the lexer (@adumbidiot)
- [BUG #235](https://github.com/jasonwilliams/boa/pull/235):
  Fixed arithmetic operations with no space (@gomesalexandre)
- [BUG #245](https://github.com/jasonwilliams/boa/pull/245):
  Fixed parsing of floats with scientific notation (@adumbidiot)

# [# 0.5.1 (2019-12-02) - Rest / Spread (almost)](https://github.com/jasonwilliams/boa/compare/v0.5.0...v0.5.1)

Feature enhancements:

- [FEATURE #151](https://github.com/jasonwilliams/boa/issues/151):
  Implement the Rest/Spread operator (functions and arrays).
- [FEATURE #193](https://github.com/jasonwilliams/boa/issues/193):
  Implement macro for setting builtin functions
- [FEATURE #211](https://github.com/jasonwilliams/boa/pull/211):
  Better Display support for all Objects (pretty printing)

# [# 0.5.0 (2019-11-06) - Hacktoberfest Release](https://github.com/jasonwilliams/boa/compare/v0.4.0...v0.5.1)

Feature enhancements:

- [FEATURE #119](https://github.com/jasonwilliams/boa/issues/119):
  Introduce realm struct to hold realm context and global object.
- [FEATURE #89](https://github.com/jasonwilliams/boa/issues/89):
  Implement exponentiation operator. Thanks @arbroween
- [FEATURE #47](https://github.com/jasonwilliams/boa/issues/47):
  Add tests for comments in source code. Thanks @Emanon42
- [FEATURE #137](https://github.com/jasonwilliams/boa/issues/137):
  Use Monaco theme for the demo page
- [FEATURE #114](https://github.com/jasonwilliams/boa/issues/114):
  String.match(regExp) is implemented (@muskuloes)
- [FEATURE #115](https://github.com/jasonwilliams/boa/issues/115):
  String.matchAll(regExp) is implemented (@bojan88)
- [FEATURE #163](https://github.com/jasonwilliams/boa/issues/163):
  Implement Array.prototype.every() (@letmutx)
- [FEATURE #165](https://github.com/jasonwilliams/boa/issues/165):
  Implement Array.prototype.find() (@letmutx)
- [FEATURE #166](https://github.com/jasonwilliams/boa/issues/166):
  Implement Array.prototype.findIndex() (@felipe-fg)
- [FEATURE #39](https://github.com/jasonwilliams/boa/issues/39):
  Implement block scoped variable declarations (@barskern)
- [FEATURE #161](https://github.com/jasonwilliams/boa/pull/161):
  Enable obj[key] = value syntax.
- [FEATURE #179](https://github.com/jasonwilliams/boa/issues/179):
  Implement the Tilde operator (@letmutx)
- [FEATURE #189](https://github.com/jasonwilliams/boa/pull/189):
  Implement Array.prototype.includes (incl tests) (@simonbrahan)
- [FEATURE #180](https://github.com/jasonwilliams/boa/pull/180):
  Implement Array.prototype.slice (@muskuloes @letmutx)
- [FEATURE #152](https://github.com/jasonwilliams/boa/issues/152):
  Short Function syntax (no arguments)
- [FEATURE #164](https://github.com/jasonwilliams/boa/issues/164):
  Implement Array.prototype.fill() (@bojan88)
- Array tests: Tests implemented for shift, unshift and reverse, pop and push (@muskuloes)
- Demo page has been improved, new font plus change on input. Thanks @WofWca
- [FEATURE #182](https://github.com/jasonwilliams/boa/pull/182):
  Implement some Number prototype methods (incl tests) (@pop)
- [FEATURE #34](https://github.com/jasonwilliams/boa/issues/34):
  Number object and Constructore are implemented (including methods) (@pop)
- [FEATURE #194](https://github.com/jasonwilliams/boa/pull/194):
  Array.prototype.map (@IovoslavIovchev)
- [FEATURE #90](https://github.com/jasonwilliams/boa/issues/90):
  Symbol Implementation (@jasonwilliams)

Bug fixes:

- [BUG #113](https://github.com/jasonwilliams/boa/issues/113):
  Unassigned variables have default of undefined (@pop)
- [BUG #61](https://github.com/jasonwilliams/boa/issues/61):
  Clippy warnings/errors fixed (@korpen)
- [BUG #147](https://github.com/jasonwilliams/boa/pull/147):
  Updated object global
- [BUG #154](https://github.com/jasonwilliams/boa/issues/154):
  Correctly handle all whitespaces within the lexer
- Tidy up Globals being added to Global Object. Thanks @DomParfitt

# 0.4.0 (2019-09-25)

v0.4.0 brings quite a big release. The biggest feature to land is the support of regular expressions.  
Functions now have the arguments object supported and we have a [`debugging`](docs/debugging.md) section in the docs.

Feature enhancements:

- [FEATURE #6](https://github.com/jasonwilliams/boa/issues/6):
  Support for regex literals. (Big thanks @999eagle)
- [FEATURE #13](https://github.com/jasonwilliams/boa/issues/13):
  toLowerCase, toUpperCase, substring, substr and valueOf implemented (thanks @arbroween)
- Support for `arguments` object within functions
- `StringData` instead of `PrimitieData` to match spec
- Native function signatures changed, operations added to match spec
- Primitives can now be boxed/unboxed when methods are ran on them
- Spelling edits (thanks @someguynamedmatt)
- Ability to set global values before interpreter starts (thanks @999eagle)
- Assign operators implemented (thanks @oll3)
-

Bug fixes:

- [BUG #57](https://github.com/jasonwilliams/boa/issues/57):
  Fixed issue with stackoverflow by implementing early returns.
- Allow to re-assign value to an existing binding. (Thanks @oll3)

# 0.3.0 (2019-07-26)

- UnexpectedKeyword(Else) bug fixed https://github.com/jasonwilliams/boa/issues/38
- Contributing guide added
- Ability to specify file - Thanks @callumquick
- Travis fixes
- Parser Tests - Thanks @Razican
- Migrate to dyn traits - Thanks @Atul9
- Added implementations for Array.prototype: concat(), push(), pop() and join() - Thanks @callumquick
- Some clippy Issues fixed - Thanks @Razican
- Objects have been refactored to use structs which are more closely aligned with the specification
- Benchmarks have been added
- String and Array specific console.log formats - Thanks @callumquick
- isPropertyKey implementation added - Thanks @KrisChambers
- Unit Tests for Array and Strings - Thanks @GalAster
- typo fix - Thanks @palerdot
- dist cleanup, thanks @zgotsch

# 0.2.1 (2019-06-30)

Some String prototype methods are implemented.  
Thanks to @lennartbuit we have
trim/trimStart/trimEnd added to the string prototype

Feature enhancements:

- [String.prototype.concat ( ...args )](https://tc39.es/ecma262/#sec-string.prototype.slice)
- [String.prototype.endsWith ( searchString [ , endPosition ] )](https://tc39.es/ecma262/#sec-string.prototype.endswith)
- [String.prototype.includes ( searchString [ , position ] )](https://tc39.es/ecma262/#sec-string.prototype.includes)
- [String.prototype.indexOf ( searchString [ , position ] )](https://tc39.es/ecma262/#sec-string.prototype.indexof)
- [String.prototype.lastIndexOf ( searchString [ , position ] )](https://tc39.es/ecma262/#sec-string.prototype.lastindexof)
- [String.prototype.repeat ( count )](https://tc39.es/ecma262/#sec-string.prototype.repeat)
- [String.prototype.slice ( start, end )](https://tc39.es/ecma262/#sec-string.prototype.slice)
- [String.prototype.startsWith ( searchString [ , position ] )](https://tc39.es/ecma262/#sec-string.prototype.startswith)

Bug fixes:

- Plenty

# 0.2.0 (2019-06-10)

Working state reached

- Tests on the lexer, conforms with puncturators and keywords from TC39 specification
- wasm-bindgen added with working demo in Web Assembly
- snapshot of boa in a working state for the first time
