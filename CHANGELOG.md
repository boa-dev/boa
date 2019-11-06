# TBD

TODO

Features:

- [FEATURE #74](https://github.com/jasonwilliams/boa/issues/74):
  Enables Boa to run within the Test 262 framework.  
  This will help us see what is implemented or not within the spec

# [# 0.5.0 (2019-11-06)](https://github.com/jasonwilliams/boa/compare/v0.4.0...HEAD)

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
