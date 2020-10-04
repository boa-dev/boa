# CHANGELOG

# 0.10.0 (2020-09-29) - New Lexer & Test 262 Harness

Feature Enhancements:

- [FEATURE #524](https://github.com/boa-dev/boa/pull/525):
  Implement remaining `Math` methods (@mr-rodgers)
- [FEATURE #562](https://github.com/boa-dev/boa/pull/562):
  Implement remaining `Number` methods (@joshwd36)
- [FEATURE #536](https://github.com/boa-dev/boa/pull/536):
  Implement `SyntaxError` (@HalidOdat)
- [FEATURE #543](https://github.com/boa-dev/boa/pull/543):
  Implements `Object.create` builtin method (@croraf)
- [FEATURE #492](https://github.com/boa-dev/boa/pull/492):
  Switch to [rustyline](https://github.com/kkawakam/rustyline) for the CLI (@IovoslavIovchev & @Razican)
- [FEATURE #595](https://github.com/boa-dev/boa/pull/595):
  Added syntax highlighting for strings in REPL (@HalidOdat)
- [FEATURE #586](https://github.com/boa-dev/boa/pull/586):
  Better error formatting and cli color (@HalidOdat)
- [FEATURE #590](https://github.com/boa-dev/boa/pull/590):
  Added keyword and operator colors and matching bracket validator to REPL (@HalidOdat)
- [FEATURE #555](https://github.com/boa-dev/boa/pull/555):
  Implement Array.prototype.reduce (@benjaminflin)
- [FEATURE #550](https://github.com/boa-dev/boa/pull/550):
  Initial implementation of Map() (@joshwd36 & @HalidOdat)
- [FEATURE #579](https://github.com/boa-dev/boa/pull/579):
  Implement Array.prototype.reduceRight (@benjaminflin)
- [FEATURE #585](https://github.com/boa-dev/boa/pull/587):
  Implement Well-Known Symbols (@joshwd36)
- [FEATURE #589](https://github.com/boa-dev/boa/pull/589):
  Implement the comma operator (@KashParty)
- [FEATURE #341](https://github.com/boa-dev/boa/pull/590):
  Ability to create multiline blocks in boa shell (@HalidOdat)
- [FEATURE #252](https://github.com/boa-dev/boa/pull/596):
  Implement `Date` (@jcdickinson)
- [FEATURE #711](https://github.com/boa-dev/boa/pull/711):
  Add support for >>>= (@arpit-saxena)
- [FEATURE #549](https://github.com/boa-dev/boa/pull/549):
  Implement label statements (@jasonwilliams)
- [FEATURE #373](https://github.com/boa-dev/boa/pull/373):
  Introduce PropertyKey for field acces (@RageKnify)
- [FEATURE #627](https://github.com/boa-dev/boa/pull/627):
  Feature native class objects (`NativeObject` and `Class` traits) (@HalidOdat)
- [FEATURE #694](https://github.com/boa-dev/boa/pull/694):
  Feature `gc` module (@HalidOdat)
- [FEATURE #656](https://github.com/boa-dev/boa/pull/656):
  Feature `Context` (@HalidOdat)
- [FEATURE #673](https://github.com/boa-dev/boa/pull/673):
  Add `#[track_caller]` to `GcObject` methods that can panic (@HalidOdat)
- [FEATURE #661](https://github.com/boa-dev/boa/pull/661):
  Add documentation to `GcObject` methods (@HalidOdat)
- [FEATURE #662](https://github.com/boa-dev/boa/pull/662):
  Implement `std::error::Error` for `GcObject` borrow errors (@HalidOdat)
- [FEATURE #660](https://github.com/boa-dev/boa/pull/660):
  Make `GcObject::contruct` not take 'this' (@HalidOdat)
- [FEATURE #654](https://github.com/boa-dev/boa/pull/654):
  Move `require_object_coercible` to `Value` (@HalidOdat)
- [FEATURE #603](https://github.com/boa-dev/boa/pull/603):
  Index `PropertyKey`, `Object` iterators and symbol support (@HalidOdat)
- [FEATURE #637](https://github.com/boa-dev/boa/pull/637):
  Feature `boa::Result<T>` (@HalidOdat)
- [FEATURE #625](https://github.com/boa-dev/boa/pull/625):
  Moved value operations from `Interpreter` to `Value` (@HalidOdat)
- [FEATURE #638](https://github.com/boa-dev/boa/pull/638):
  Changed to `Value::to_*int32` => `Value::to_*32` (@HalidOdat)

Bug Fixes:

- [BUG #405](https://github.com/boa-dev/boa/issues/405):
  Fix json.stringify symbol handling (@n14little)
- [BUG #520](https://github.com/boa-dev/boa/pull/520):
  Fix all `Value` operations and add unsigned shift right (@HalidOdat)
- [BUG #529](https://github.com/boa-dev/boa/pull/529):
  Refactor exec/expression into exec/call and exec/new (@croraf)
- [BUG #510](https://github.com/boa-dev/boa/issues/510):
  [[Call]] calling an undefined method does not throw (@joshwd36)
- [BUG #493](https://github.com/boa-dev/boa/pull/493):
  Use correct exponential representation for rational values (@Tropid)
- [BUG #572](https://github.com/boa-dev/boa/pull/572):
  Spec Compliant `Number.prototype.toString()`, better `Number` object formating and `-0` (@HalidOdat)
- [BUG #599](https://github.com/boa-dev/boa/pull/599):
  Fixed `String.prototype.indexOf()` bug, when the search string is empty (@HalidOdat)
- [BUG #615](https://github.com/boa-dev/boa/issues/615):
  Fix abstract relational comparison operators (@HalidOdat)
- [BUG #608](https://github.com/boa-dev/boa/issues/608):
  `Debug::fmt` Causes Causes a Stack Overflow (@jcdickinson)
- [BUG #532](https://github.com/boa-dev/boa/issues/532)
  [builtins - Object] Object.getPrototypeOf returning incorrectly (@54k1)
- [BUG #533](https://github.com/boa-dev/boa/issues/533)
  [exec - function] function.prototype doesn't have own constructor property pointing to this function (@54k1)
- [BUG #641](https://github.com/boa-dev/boa/issues/641)
  Test new_instance_should_point_to_prototype is not checked correctly (@54k1)
- [BUG #644](https://github.com/boa-dev/boa/pull/645)
  `undefined` constants panic on execution (@jcdickinson)
- [BUG #631](https://github.com/boa-dev/boa/pull/645):
  Unexpected result when applying typeof to undefined value (@jcdickinson)
- [BUG #667](https://github.com/boa-dev/boa/pull/667):
  Fix panic when calling function that mutates itself (@dvtkrlbs)
- [BUG #668](https://github.com/boa-dev/boa/pull/668):
  Fix clippy on Nightly (@dvtkrlbs)
- [BUG #582](https://github.com/boa-dev/boa/pull/582):
  Make `String.prototype.repeat()` ECMAScript specification compliant (@HalidOdat)
- [BUG #541](https://github.com/boa-dev/boa/pull/541):
  Made all `Math` methods spec compliant (@HalidOdat)
- [BUG #597](https://github.com/boa-dev/boa/pull/597):
  Made `String.prototype.indexOf` spec compliant. (@HalidOdat)
- [BUG #598](https://github.com/boa-dev/boa/pull/598):
  Made `String.prototype.lastIndexOf()` spec compliant (@HalidOdat)
- [BUG #583](https://github.com/boa-dev/boa/pull/583):
  Fix string prototype `trim` methods (@HalidOdat)
- [BUG #728](https://github.com/boa-dev/boa/pull/728):
  Fix bug when setting the length on String objects (@jasonwilliams)
- [BUG #710](https://github.com/boa-dev/boa/pull/710):
  Fix panic when a self mutating function is constructing an object (@HalidOdat)
- [BUG #699](https://github.com/boa-dev/boa/pull/699):
  Fix `Value::to_json` order of items in array (@sele9)
- [BUG #610](https://github.com/boa-dev/boa/pull/610):
  Fix: `String.prototype.replace` substitutions (@RageKnify)
- [BUG #645](https://github.com/boa-dev/boa/pull/645):
  Fix undefined constant expression evaluation (@jcdickinson)
- [BUG #643](https://github.com/boa-dev/boa/pull/643):
  Change default return type from null to undefined (@54k1)
- [BUG #642](https://github.com/boa-dev/boa/pull/642):
  Missing `constructor` field in ordinary functions (@54k1)
- [BUG #604](https://github.com/boa-dev/boa/pull/604):
  Missing `__proto__` field in functions instances (@54k1)
- [BUG #561](https://github.com/boa-dev/boa/pull/561):
  Throw a `TypeError` when a non-object is called (@joshwd36)
- [BUG #748](https://github.com/boa-dev/boa/pull/748):
  Fix parse error throwing a `TypeError`, instead of `SyntaxError` (@iamsaquib8)
- [BUG #737](https://github.com/boa-dev/boa/pull/737):
  Make `Object.toString()` spec compliant (@RageKnify)

Internal Improvements:

- [INTERNAL #567](https://github.com/boa-dev/boa/pull/567):
  Add ECMAScript test suite (test262) (@Razican)
- [INTERNAL #559](https://github.com/boa-dev/boa/pull/559):
  New Lexer (@Lan2u @HalidOdat @Razican)
- [INTERNAL #712](https://github.com/boa-dev/boa/pull/712):
  Refactor: `Value::to_object` to return `GcObject` (@RageKnify)
- [INTERNAL #544](https://github.com/boa-dev/boa/pull/544):
  Removed `console`s dependency of `InternalState` (@HalidOdat)
- [INTERNAL #556](https://github.com/boa-dev/boa/pull/556):
  Added benchmark for goal symbol switching (@Razican)
- [INTERNAL #578](https://github.com/boa-dev/boa/pull/580):
  Extract `prototype` from internal slots (@HalidOdat)
- [INTERNAL #553](https://github.com/boa-dev/boa/pull/553):
  Refactor Property Descriptor flags (@HalidOdat)
- [INTERNAL #592](https://github.com/boa-dev/boa/pull/592):
  `RegExp` specialization (@HalidOdat)
- [INTERNAL #626](https://github.com/boa-dev/boa/pull/626):
  Refactor `Function` (@HalidOdat @Razican)
- [INTERNAL #564](https://github.com/boa-dev/boa/pull/581):
  Add benchmarks for "uglified" JS (@neeldug)
- [INTERNAL #706](https://github.com/boa-dev/boa/pull/706):
  Cache well known symbols (@HalidOdat)
- [INTERNAL #723](https://github.com/boa-dev/boa/pull/723):
  Add fast path for string concatenation (@RageKnify)
- [INTERNAL #689](https://github.com/boa-dev/boa/pull/689):
  Move `object` module to root (@HalidOdat)
- [INTERNAL #684](https://github.com/boa-dev/boa/pull/684):
  Move `property` module to root (@HalidOdat)
- [INTERNAL #674](https://github.com/boa-dev/boa/pull/674):
  Move `value` module to root (@HalidOdat)
- [INTERNAL #693](https://github.com/boa-dev/boa/pull/693):
  Rename `Object::prototype()` and `Object::set_prototype()` (@RageKnify)
- [INTERNAL #665](https://github.com/boa-dev/boa/pull/665):
  `approx_eq!` macro for `expm1` tests. (@neeldung)
- [INTERNAL #581](https://github.com/boa-dev/boa/pull/581):
  Added CLEAN_JS and MINI_JS benches (@neeldung)
- [INTERNAL #640](https://github.com/boa-dev/boa/pull/640):
  Benchmark refactor (@neeldung)
- [INTERNAL #635](https://github.com/boa-dev/boa/pull/635):
  Add missing ops to exec module (@jarredholman)
- [INTERNAL #616](https://github.com/boa-dev/boa/pull/616):
  Remove `Value::as_num_to_power()` (@HalidOdat)
- [INTERNAL #601](https://github.com/boa-dev/boa/pull/601):
  Removed internal_slots from object (@HalidOdat)
- [INTERNAL #560](https://github.com/boa-dev/boa/pull/560):
  Added benchmarks for full program execution (@Razican)
- [INTERNAL #547](https://github.com/boa-dev/boa/pull/547):
  Merged `create` into `init` for builtins (@HalidOdat)
- [INTERNAL #538](https://github.com/boa-dev/boa/pull/538):
  Cleanup and added test for `String.prototype.concat` (@HalidOdat)
- [INTERNAL #739](https://github.com/boa-dev/boa/pull/739):
  Add release action (@jasonwilliams)
- [INTERNAL #744](https://github.com/boa-dev/boa/pull/744):
  Add MacOS check and test to CI (@neeldug)

# [# 0.9.0 (2020-07-03) - Move to Organisation, 78% faster execution time](https://github.com/boa-dev/boa/compare/v0.8.0...v0.9.0)

Feature Enhancements:

- [FEATURE #414](https://github.com/boa-dev/boa/issues/414):
  Implement `Number` object constants (@Lan2u) (@HalidOdat)
- [FEATURE #345](https://github.com/boa-dev/boa/issues/345):
  Implement the optional `replacer` parameter in `JSON.stringify( value[, replacer [, space] ] )` (@n14little)
- [FEATURE #480](https://github.com/boa-dev/boa/issues/480):
  Implement global `Infinity` property (@AnirudhKonduru)
- [FEATURE #410](https://github.com/boa-dev/boa/pull/410):
  Add support for the reviver function to JSON.parse (@abhijeetbhagat)
- [FEATURE #425](https://github.com/boa-dev/boa/pull/425):
  Specification compliant `ToString` (`to_string`) (@HalidOdat)
- [FEATURE #442](https://github.com/boa-dev/boa/pull/442):
  Added `TypeError` implementation (@HalidOdat)
- [FEATURE #450](https://github.com/boa-dev/boa/pull/450):
  Specification compliant `ToBigInt` (`to_bigint`) (@HalidOdat)
- [FEATURE #455](https://github.com/boa-dev/boa/pull/455):
  TemplateLiteral Basic lexer implementation (@croraf)
- [FEATURE #447](https://github.com/boa-dev/boa/issues/447):
  parseInt, parseFloat implementation (@Lan2u)
- [FEATURE #468](https://github.com/boa-dev/boa/pull/468):
  Add BigInt.asIntN() and BigInt.asUintN() functions (@Tropid)
- [FEATURE #428](https://github.com/boa-dev/boa/issues/428):
  [Feature Request] - Create benchmark for Array manipulation (@abhijeetbhagat)
- [FEATURE #439](https://github.com/boa-dev/boa/issues/439):
  Implement break handling in switch statements (@Lan2u)
- [FEATURE #301](https://github.com/boa-dev/boa/issues/301):
  Implementing the switch statement in the new parser (@Lan2u)
- [FEATURE #120](https://github.com/boa-dev/boa/issues/120):
  Implement `globalThis` (@zanayr)
- [FEATURE #513](https://github.com/boa-dev/boa/issues/513):
  Implement `Object.is()` method (@tylermorten)
- [FEATURE #481](https://github.com/boa-dev/boa/issues/481):
  Implement global `undefined` property (@croraf)

Bug Fixes:

- [BUG #412](https://github.com/boa-dev/boa/pull/412):
  Fixed parsing if statement without else block preceded by a newline (@HalidOdat)
- [BUG #409](https://github.com/boa-dev/boa/pull/409):
  Fix function object constructable/callable (@HalidOdat)
- [BUG #403](https://github.com/boa-dev/boa/issues/403)
  `Value::to_json()` does not handle `undefined` correctly (@n14little)
- [BUG #443](https://github.com/boa-dev/boa/issues/443):
  HasOwnProperty should call GetOwnProperty and not GetProperty (@n14little)
- [BUG #210](https://github.com/boa-dev/boa/issues/210):
  builtinfun.length undefined (@Croraf)
- [BUG #466](https://github.com/boa-dev/boa/issues/466):
  Change `ToPrimitive()` (`to_primitive()`) hint to be an enum, instead of string (@HalidOdat)
- [BUG #421](https://github.com/boa-dev/boa/issues/421):
  `NaN` is lexed as a number, not as an identifier (@croraf)
- [BUG #454](https://github.com/boa-dev/boa/issues/454):
  Function declaration returns the function, it should return `undefined` (@croraf)
- [BUG #482](https://github.com/boa-dev/boa/issues/482):
  Field access should propagate the exception (`Err(_)`) (@neeldug)
- [BUG #463](https://github.com/boa-dev/boa/issues/463):
  Use of undefined variable should throw an error (@croraf)
- [BUG #502](https://github.com/boa-dev/boa/pull/502):
  Fixed global objects initialization order (@HalidOdat)
- [BUG #509](https://github.com/boa-dev/boa/issues/509):
  JSON.stringify(undefined) panics (@n14little)
- [BUG #514](https://github.com/boa-dev/boa/issues/514):
  Clean up `Math` Methods (@n14little)
- [BUG #511](https://github.com/boa-dev/boa/issues/511):
  [Call] Usage of "this" in methods is not supported (@jasonwilliams)

Internal Improvements

- [INTERNAL #435](https://github.com/boa-dev/boa/issues/435):
  Optimize type comparisons (@Lan2u)
- [INTERNAL #296](https://github.com/boa-dev/boa/issues/296):
  using measureme for profiling the interpreter (@jasonwilliams)
- [INTERNAL #419](https://github.com/boa-dev/boa/pull/419):
  Object specialization (fast paths for many objects) (@HalidOdat)
- [INTERNAL #392](https://github.com/boa-dev/boa/pull/392):
  Execution and Node modulization (@Razican)
- [INTERNAL #465](https://github.com/boa-dev/boa/issues/465):
  Refactoring Value (decouple `Gc` from `Value`) (@HalidOdat)
- [INTERNAL #416](https://github.com/boa-dev/boa/pull/416) & [INTERNAL #423](https://github.com/boa-dev/boa/commit/c8218dd91ef3181e048e7a2659a4fbf8d53c7174):
  Update links to boa-dev (@pedropaulosuzuki)
- [INTERNAL #378](https://github.com/boa-dev/boa/issues/378):
  Code Coverage! (@Lan2u)
- [INTERNAL #431](https://github.com/boa-dev/boa/pull/431):
  Updates to PR Benchmarks (@Razican)
- [INTERNAL #427 #429 #430](https://github.com/boa-dev/boa/commit/64dbf13afd15f12f958daa87a3d236dc9af1a9aa):
  Added new benchmarks (@Razican)

# [# 0.8.0 (2020-05-23) - BigInt, Modularized Parser, Faster Hashing](https://github.com/boa-dev/boa/compare/v0.7.0...v0.8.0)

`v0.8.0` brings more language implementations, such as do..while, function objects and also more recent EcmaScript additions, like BigInt.
We have now moved the Web Assembly build into the `wasm` package, plus added a code of conduct for those contributing.

The parser has been even more modularized in this release making it easier to add new parsing rules.

Boa has migrated it's object implemention to FXHash which brings much improved results over the built-in Rust hashmaps (at the cost of less DOS Protection).

Feature Enhancements:

- [FEATURE #121](https://github.com/boa-dev/boa/issues/121):
  `BigInt` Implemented (@HalidOdat)
- [FEATURE #293](https://github.com/boa-dev/boa/pull/293):
  Improved documentation of all modules (@HalidOdat)
- [FEATURE #302](https://github.com/boa-dev/boa/issues/302):
  Implement do..while loop (@ptasz3k)
- [FEATURE #318](https://github.com/boa-dev/boa/pull/318):
  Added continous integration for windows (@HalidOdat)
- [FEATURE #290](https://github.com/boa-dev/boa/pull/290):
  Added more build profiles (@Razican)
- [FEATURE #323](https://github.com/boa-dev/boa/pull/323):
  Aded more benchmarks (@Razican)
- [FEATURE #326](https://github.com/boa-dev/boa/pull/326):
  Rename Boa CLI (@sphinxc0re)
- [FEATURE #312](https://github.com/boa-dev/boa/pull/312):
  Added jemallocator for linux targets (@Razican)
- [FEATURE #339](https://github.com/boa-dev/boa/pull/339):
  Improved Method parsing (@muskuloes)
- [FEATURE #352](https://github.com/boa-dev/boa/pull/352):
  create boa-wasm package (@muskuloes)
- [FEATURE #304](https://github.com/boa-dev/boa/pull/304):
  Modularized parser
- [FEATURE #141](https://github.com/boa-dev/boa/issues/141):
  Implement function objects (@jasonwilliams)
- [FEATURE #365](https://github.com/boa-dev/boa/issues/365):
  Implement for loop execution (@Razican)
- [FEATURE #356](https://github.com/boa-dev/boa/issues/356):
  Use Fx Hash to speed up hash maps in the compiler (@Razican)
- [FEATURE #321](https://github.com/boa-dev/boa/issues/321):
  Implement unary operator execution (@akryvomaz)
- [FEATURE #379](https://github.com/boa-dev/boa/issues/379):
  Automatic auditing of Boa (@n14little)
- [FEATURE #264](https://github.com/boa-dev/boa/issues/264):
  Implement `this` (@jasonwilliams)
- [FEATURE #395](https://github.com/boa-dev/boa/pull/395):
  impl abstract-equality-comparison (@hello2dj)
- [FEATURE #359](https://github.com/boa-dev/boa/issues/359):
  impl typeof (@RestitutorOrbis)
- [FEATURE #390](https://github.com/boa-dev/boa/pull/390):
  Modularize try statement parsing (@abhijeetbhagat)

Bug fixes:

- [BUG #308](https://github.com/boa-dev/boa/issues/308):
  Assignment operator not working in tests (a = a +1) (@ptasz3k)
- [BUG #322](https://github.com/boa-dev/boa/issues/322):
  Benchmarks are failing in master (@Razican)
- [BUG #325](https://github.com/boa-dev/boa/pull/325):
  Put JSON functions on the object, not the prototype (@coolreader18)
- [BUG #331](https://github.com/boa-dev/boa/issues/331):
  We only get `Const::Num`, never `Const::Int` (@HalidOdat)
- [BUG #209](https://github.com/boa-dev/boa/issues/209):
  Calling `new Array` with 1 argument doesn't work properly (@HalidOdat)
- [BUG #266](https://github.com/boa-dev/boa/issues/266):
  Panic assigning named function to variable (@Razican)
- [BUG #397](https://github.com/boa-dev/boa/pull/397):
  fix `NaN` is lexed as identifier, not as a number (@attliaLin)
- [BUG #362](https://github.com/boa-dev/boa/pull/362):
  Remove Monaco Editor Webpack Plugin and Manually Vendor Editor Workers (@subhankar-panda)
- [BUG #406](https://github.com/boa-dev/boa/pull/406):
  Dependency Upgrade (@Razican)
- [BUG #407](https://github.com/boa-dev/boa/pull/407):
  `String()` wasn't defaulting to empty string on call (@jasonwilliams)
- [BUG #404](https://github.com/boa-dev/boa/pull/404):
  Fix for 0 length new String(@tylermorten)

Code Of Conduct:

- [COC #384](https://github.com/boa-dev/boa/pull/384):
  Code of conduct added (@Razican)

Security:

- [SEC #391](https://github.com/boa-dev/boa/pull/391):
  run security audit daily at midnight. (@n14little)

# [# 0.7.0 (2020-04-13) - New Parser is 67% faster](https://github.com/boa-dev/boa/compare/v0.6.0...v0.7.0)

`v0.7.0` brings a REPL, Improved parser messages and a new parser!
This is now the default behaviour of Boa, so running Boa without a file argument will bring you into a javascript shell.
Tests have also been moved to their own files, we had a lot of tests in some modules so it was time to separate.

## New Parser

Most of the work in this release has been on rewriting the parser. A big task taken on by [HalidOdat](https://github.com/HalidOdat), [Razican](https://github.com/Razican) and [myself](https://github.com/jasonwilliams).

The majority of the old parser was 1 big function (called [`parse`](https://github.com/boa-dev/boa/blob/019033eff066e8c6ba9456139690eb214a0bf61d/boa/src/syntax/parser.rs#L353)) which had some pattern matching on each token coming in.\
The easy branches could generate expressions (which were basically AST Nodes), the more involved branches would recursively call into the same function, until eventually you had an expression generated.

This only worked so far, eventually debugging parsing problems were difficult, also more bugs were being raised against the parser which couldn't be fixed.

We decided to break the parser into more of a state-machine. The initial decision for this was inspired by [Fedor Indutny](https://github.com/indutny) who did a talk at (the last) JSConf EU about how he broke up the old node-parser to make it more maintanable. He goes into more detail here https://www.youtube.com/watch?v=x3k_5Mi66sY&feature=youtu.be&t=530

The new parser has functions to match the states of parsing in the spec. For example https://tc39.es/ecma262/#prod-VariableDeclaration has a matching function `read_variable_declaration`. This not only makes it better to maintain but easier for new contributors to get involed, as following the parsing logic of the spec is easier than before.

Once finished some optimisations were added by [HalidOdat](https://github.com/HalidOdat) to use references to the tokens instead of cloning them each time we take them from the lexer.\
This works because the tokens live just as long as the parser operations do, so we don't need to copy the tokens.\
What this brings is a huge performance boost, the parser is 67% faster than before!

![Parser Improvement](./docs/img/parser-graph.png)

Feature enhancements:

- [FEATURE #281](https://github.com/boa-dev/boa/pull/281):
  Rebuild the parser (@jasonwilliams, @Razican, @HalidOdat)
- [FEATURE #278](https://github.com/boa-dev/boa/pull/278):
  Added the ability to dump the token stream or ast in bin. (@HalidOdat)
- [FEATURE #253](https://github.com/boa-dev/boa/pull/253):
  Implement Array.isArray (@cisen)
- [FEATURE](https://github.com/boa-dev/boa/commit/edab5ca6cc10d13265f82fa4bc05d6b432a362fc)
  Switch to normal output instead of debugged output (stdout/stdout) (@jasonwilliams)
- [FEATURE #258](https://github.com/boa-dev/boa/pull/258):
  Moved test modules to their own files (@Razican)
- [FEATURE #267](https://github.com/boa-dev/boa/pull/267):
  Add print & REPL functionality to CLI (@JohnDoneth)
- [FEATURE #268](https://github.com/boa-dev/boa/pull/268):
  Addition of forEach() (@jasonwilliams) (@xSke)
- [FEATURE #262](https://github.com/boa-dev/boa/pull/262):
  Implement Array.prototype.filter (@Nickforall)
- [FEATURE #261](https://github.com/boa-dev/boa/pull/261):
  Improved parser error messages (@Razican)
- [FEATURE #277](https://github.com/boa-dev/boa/pull/277):
  Add a logo to the project (@HalidOdat)
- [FEATURE #260](https://github.com/boa-dev/boa/pull/260):
  Add methods with f64 std equivelant to Math object (@Nickforall)

Bug fixes:

- [BUG #249](https://github.com/boa-dev/boa/pull/249):
  fix(parser): handle trailing comma in object literals (@gomesalexandre)
- [BUG #244](https://github.com/boa-dev/boa/pull/244):
  Fixed more Lexer Panics (@adumbidiot)
- [BUG #256](https://github.com/boa-dev/boa/pull/256):
  Fixed comments lexing (@Razican)
- [BUG #251](https://github.com/boa-dev/boa/issues/251):
  Fixed empty returns (@Razican)
- [BUG #272](https://github.com/boa-dev/boa/pull/272):
  Fix parsing of floats that start with a zero (@Nickforall)
- [BUG #240](https://github.com/boa-dev/boa/issues/240):
  Fix parser panic
- [BUG #273](https://github.com/boa-dev/boa/issues/273):
  new Class().method() has incorrect precedence

Documentation Updates:

- [DOC #297](https://github.com/boa-dev/boa/pull/297):
  Better user contributed documentation

# [# 0.6.0 (2020-02-14) - Migration to Workspace Architecture + lexer/parser improvements](https://github.com/boa-dev/boa/compare/v0.5.1...v0.6.0)

The lexer has had several fixes in this release, including how it parses numbers, scientific notation should be improved.
On top of that the lexer no longer panics on errors including Syntax Errors (thanks @adumbidiot), instead you get some output on where the error happened.

## Moving to a workspace architecture

Boa offers both a CLI and a library, initially these were all in the same binary. The downside is
those who want to embed boa as-is end up with all of the command-line dependencies.
So the time has come to separate out the two, this is normal procedure, this should be analogous to ripgrep
and the regex crate.
Cargo has great support for workspaces, so this shouldn't be an issue.

## Benchmarks

We now have [benchmarks which run against master](https://boa-dev.github.io/boa/dev/bench)!
Thanks to Github Actions these will run automatically a commit is merged.

Feature enhancements:

- [FEATURE #218](https://github.com/boa-dev/boa/pull/218):
  Implement Array.prototype.toString (@cisen)
- [FEATURE #216](https://github.com/boa-dev/boa/commit/85e9a3526105a600358bd53811e2b022987c6fc8):
  Keep accepting new array elements after spread.
- [FEATURE #220](https://github.com/boa-dev/boa/pull/220):
  Documentation updates. (@croraf)
- [FEATURE #226](https://github.com/boa-dev/boa/pull/226):
  add parser benchmark for expressions. (@jasonwilliams)
- [FEATURE #217](https://github.com/boa-dev/boa/pull/217):
  String.prototype.replace() implemented
- [FEATURE #247](https://github.com/boa-dev/boa/pull/247):
  Moved to a workspace architecture (@Razican)

Bug fixes:

- [BUG #222](https://github.com/boa-dev/boa/pull/222):
  Fixed clippy errors (@IovoslavIovchev)
- [BUG #228](https://github.com/boa-dev/boa/pull/228):
  [lexer: single-line-comment] Fix bug when single line comment is last line of file (@croraf)
- [BUG #229](https://github.com/boa-dev/boa/pull/229):
  Replace error throwing with panic in "Lexer::next()" (@croraf)
- [BUG #232/BUG #238](https://github.com/boa-dev/boa/pull/232):
  Clippy checking has been scaled right back to just Perf and Style (@jasonwilliams)
- [BUG #227](https://github.com/boa-dev/boa/pull/227):
  Array.prototype.toString should be called by ES value (@cisen)
- [BUG #242](https://github.com/boa-dev/boa/pull/242):
  Fixed some panics in the lexer (@adumbidiot)
- [BUG #235](https://github.com/boa-dev/boa/pull/235):
  Fixed arithmetic operations with no space (@gomesalexandre)
- [BUG #245](https://github.com/boa-dev/boa/pull/245):
  Fixed parsing of floats with scientific notation (@adumbidiot)

# [# 0.5.1 (2019-12-02) - Rest / Spread (almost)](https://github.com/boa-dev/boa/compare/v0.5.0...v0.5.1)

Feature enhancements:

- [FEATURE #151](https://github.com/boa-dev/boa/issues/151):
  Implement the Rest/Spread operator (functions and arrays).
- [FEATURE #193](https://github.com/boa-dev/boa/issues/193):
  Implement macro for setting builtin functions
- [FEATURE #211](https://github.com/boa-dev/boa/pull/211):
  Better Display support for all Objects (pretty printing)

# [# 0.5.0 (2019-11-06) - Hacktoberfest Release](https://github.com/boa-dev/boa/compare/v0.4.0...v0.5.1)

Feature enhancements:

- [FEATURE #119](https://github.com/boa-dev/boa/issues/119):
  Introduce realm struct to hold realm context and global object.
- [FEATURE #89](https://github.com/boa-dev/boa/issues/89):
  Implement exponentiation operator. Thanks @arbroween
- [FEATURE #47](https://github.com/boa-dev/boa/issues/47):
  Add tests for comments in source code. Thanks @Emanon42
- [FEATURE #137](https://github.com/boa-dev/boa/issues/137):
  Use Monaco theme for the demo page
- [FEATURE #114](https://github.com/boa-dev/boa/issues/114):
  String.match(regExp) is implemented (@muskuloes)
- [FEATURE #115](https://github.com/boa-dev/boa/issues/115):
  String.matchAll(regExp) is implemented (@bojan88)
- [FEATURE #163](https://github.com/boa-dev/boa/issues/163):
  Implement Array.prototype.every() (@letmutx)
- [FEATURE #165](https://github.com/boa-dev/boa/issues/165):
  Implement Array.prototype.find() (@letmutx)
- [FEATURE #166](https://github.com/boa-dev/boa/issues/166):
  Implement Array.prototype.findIndex() (@felipe-fg)
- [FEATURE #39](https://github.com/boa-dev/boa/issues/39):
  Implement block scoped variable declarations (@barskern)
- [FEATURE #161](https://github.com/boa-dev/boa/pull/161):
  Enable obj[key] = value syntax.
- [FEATURE #179](https://github.com/boa-dev/boa/issues/179):
  Implement the Tilde operator (@letmutx)
- [FEATURE #189](https://github.com/boa-dev/boa/pull/189):
  Implement Array.prototype.includes (incl tests) (@simonbrahan)
- [FEATURE #180](https://github.com/boa-dev/boa/pull/180):
  Implement Array.prototype.slice (@muskuloes @letmutx)
- [FEATURE #152](https://github.com/boa-dev/boa/issues/152):
  Short Function syntax (no arguments)
- [FEATURE #164](https://github.com/boa-dev/boa/issues/164):
  Implement Array.prototype.fill() (@bojan88)
- Array tests: Tests implemented for shift, unshift and reverse, pop and push (@muskuloes)
- Demo page has been improved, new font plus change on input. Thanks @WofWca
- [FEATURE #182](https://github.com/boa-dev/boa/pull/182):
  Implement some Number prototype methods (incl tests) (@pop)
- [FEATURE #34](https://github.com/boa-dev/boa/issues/34):
  Number object and Constructore are implemented (including methods) (@pop)
- [FEATURE #194](https://github.com/boa-dev/boa/pull/194):
  Array.prototype.map (@IovoslavIovchev)
- [FEATURE #90](https://github.com/boa-dev/boa/issues/90):
  Symbol Implementation (@jasonwilliams)

Bug fixes:

- [BUG #113](https://github.com/boa-dev/boa/issues/113):
  Unassigned variables have default of undefined (@pop)
- [BUG #61](https://github.com/boa-dev/boa/issues/61):
  Clippy warnings/errors fixed (@korpen)
- [BUG #147](https://github.com/boa-dev/boa/pull/147):
  Updated object global
- [BUG #154](https://github.com/boa-dev/boa/issues/154):
  Correctly handle all whitespaces within the lexer
- Tidy up Globals being added to Global Object. Thanks @DomParfitt

# 0.4.0 (2019-09-25)

v0.4.0 brings quite a big release. The biggest feature to land is the support of regular expressions.
Functions now have the arguments object supported and we have a [`debugging`](docs/debugging.md) section in the docs.

Feature enhancements:

- [FEATURE #6](https://github.com/boa-dev/boa/issues/6):
  Support for regex literals. (Big thanks @999eagle)
- [FEATURE #13](https://github.com/boa-dev/boa/issues/13):
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

- [BUG #57](https://github.com/boa-dev/boa/issues/57):
  Fixed issue with stackoverflow by implementing early returns.
- Allow to re-assign value to an existing binding. (Thanks @oll3)

# 0.3.0 (2019-07-26)

- UnexpectedKeyword(Else) bug fixed https://github.com/boa-dev/boa/issues/38
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
