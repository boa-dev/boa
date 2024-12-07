# CHANGELOG

## What's Changed

# [0.20.0 (2024-12-5)](https://github.com/boa-dev/boa/compare/v0.19.1...v0.20.0)

### Feature Enhancements

- Add a js_error! macro to create opaque errors by @hansl in https://github.com/boa-dev/boa/pull/3920
- Update `Instant` for new Temporal functionality by @nekevss in https://github.com/boa-dev/boa/pull/3928
- Add a way to add setters/getters in js_class! by @hansl in https://github.com/boa-dev/boa/pull/3911
- Fix lints from rustc 1.80.0 by @jedel1043 in https://github.com/boa-dev/boa/pull/3936
- Add a JsError::from_rust constructor to create native errors from Rust by @hansl in https://github.com/boa-dev/boa/pull/3921
- add some temporal methods by @jasonwilliams in https://github.com/boa-dev/boa/pull/3856
- Allow a custom Logger to be used as the backend for boa_runtime::Console by @hansl in https://github.com/boa-dev/boa/pull/3943
- Add more utility functions around modules and exports by @hansl in https://github.com/boa-dev/boa/pull/3937
- Allow trailing commas in js_class functions by @hansl in https://github.com/boa-dev/boa/pull/3964
- Implement `Atomics.pause` by @jedel1043 in https://github.com/boa-dev/boa/pull/3956
- Add a clone_inner method to allow cloning of inner data by @hansl in https://github.com/boa-dev/boa/pull/3968
- fix: ignore `debugger` statement by @shurizzle in https://github.com/boa-dev/boa/pull/3976
- Add support for boa(rename = "") in TryFromJs derive by @hansl in https://github.com/boa-dev/boa/pull/3980
- Add an "iter()" method to Js\*Array for convenience by @hansl in https://github.com/boa-dev/boa/pull/3986
- A simple module loader from a function by @hansl in https://github.com/boa-dev/boa/pull/3932
- Add a way for js_error! macro to create native errors with message by @hansl in https://github.com/boa-dev/boa/pull/3971
- Limit actions runs to 1 per branch and fix macos release by @jedel1043 in https://github.com/boa-dev/boa/pull/3996
- Add TextEncoder, TextDecoder implementations to boa_runtime by @hansl in https://github.com/boa-dev/boa/pull/3994
- Add TryFromJs for TypedJsFunction and more tests by @hansl in https://github.com/boa-dev/boa/pull/3981
- Add context to the console `Logger` trait by @hansl in https://github.com/boa-dev/boa/pull/4005
- Add a URL class to boa_runtime by @hansl in https://github.com/boa-dev/boa/pull/4004
- Add a display_lossy() to write a JsString lossily by @hansl in https://github.com/boa-dev/boa/pull/4023
- `TryIntoJs` trait and derive macro for it by @Nikita-str in https://github.com/boa-dev/boa/pull/3999
- console.debug() should use a debug Logger method by @hansl in https://github.com/boa-dev/boa/pull/4019
- `TryFromJs` from `JsMap` for `HashMap` & `BtreeMap` by @Nikita-str in https://github.com/boa-dev/boa/pull/3998
- Add string builder to build `JsString` by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/3915

### Bug Fixes

- Implement `Math.pow` function according to ECMAScript specification by @magic-akari in https://github.com/boa-dev/boa/pull/3916
- Fix temporal builtin properties by @nekevss in https://github.com/boa-dev/boa/pull/3930
- Fix wrong `neg` operation by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/3926
- Fix destructuring assignment evaluation order by @raskad in https://github.com/boa-dev/boa/pull/3934
- Fix various parser idempotency issues and parsing errors by @raskad in https://github.com/boa-dev/boa/pull/3917
- Implement new spec changes for `AsyncGenerator` by @jedel1043 in https://github.com/boa-dev/boa/pull/3950
- Refactor ast function types by @raskad in https://github.com/boa-dev/boa/pull/3931
- Fix `js_str` macro to correctly handle latin1 strings by @jedel1043 in https://github.com/boa-dev/boa/pull/3959
- Allow dead code for code that is newly detected as unused by @hansl in https://github.com/boa-dev/boa/pull/3984
- Allow warnings when running CI on release branches by @jedel1043 in https://github.com/boa-dev/boa/pull/3990
- docs: Fix link to examples by @it-a-me in https://github.com/boa-dev/boa/pull/4007
- `IntegerOrInfinity` `eq` bug fix by @Nikita-str in https://github.com/boa-dev/boa/pull/4010

### Internal Improvements

- Refactor `RawJsString`'s representation to make `JsString`s construction from string literal heap-allocation free by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/3935
- Split default icu data into lazily deserialized parts by @jedel1043 in https://github.com/boa-dev/boa/pull/3948
- Add clippy for denying print and eprints by @hansl in https://github.com/boa-dev/boa/pull/3967
- Refactor iterator APIs to be on parity with the latest spec by @jedel1043 in https://github.com/boa-dev/boa/pull/3962
- Add support for Trace, Finalize and JsData for Convert<> by @hansl in https://github.com/boa-dev/boa/pull/3970
- use with_capacity to reduce re-allocations fixes #3896 by @jasonwilliams in https://github.com/boa-dev/boa/pull/3961
- add nightly build by @jasonwilliams in https://github.com/boa-dev/boa/pull/4026
- Patch the indentation in nightly_build.yml by @nekevss in https://github.com/boa-dev/boa/pull/4028
- Update night build's rename binary step by @nekevss in https://github.com/boa-dev/boa/pull/4032
- Use upload-rust-binary-action for nightly release by @nekevss in https://github.com/boa-dev/boa/pull/4040
- Fix `ref` value in nightly and add target to nightly release by @nekevss in https://github.com/boa-dev/boa/pull/4042
- Reduce environment allocations by @raskad in https://github.com/boa-dev/boa/pull/4002

### Other Changes

- Implement more Temporal functionality by @nekevss in https://github.com/boa-dev/boa/pull/3924
- Add a Source::with_path method to set the path on a Source by @hansl in https://github.com/boa-dev/boa/pull/3941
- Add spec edition 15 to the tester by @jedel1043 in https://github.com/boa-dev/boa/pull/3957
- Rename as_promise to as_promise_object and add as_promise -> JsPromise by @hansl in https://github.com/boa-dev/boa/pull/3965
- Build out partial record functionality, property bag construction, and `with` methods by @nekevss in https://github.com/boa-dev/boa/pull/3955
- Enable CI for release branches by @jedel1043 in https://github.com/boa-dev/boa/pull/3987
- Add a display type for JsString to allow formatting without allocations by @hansl in https://github.com/boa-dev/boa/pull/3951
- Add TryIntoJsResult for vectors by @hansl in https://github.com/boa-dev/boa/pull/3993
- Add tests from WPT and fix them in the Console by @hansl in https://github.com/boa-dev/boa/pull/3979
- Update changelog for v0.19.1 by @jedel1043 in https://github.com/boa-dev/boa/pull/3995
- Implement register allocation by @HalidOdat in https://github.com/boa-dev/boa/pull/3942
- Implement scope analysis and local variables by @raskad in https://github.com/boa-dev/boa/pull/3988
- `JsValue::to_json` fix integer property keys by @Nikita-str in https://github.com/boa-dev/boa/pull/4011
- Some optimizations on `Error` by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/4020
- Option::None should try into Undefined, not Null by @hansl in https://github.com/boa-dev/boa/pull/4029
- Some string optimizations by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/4030
- Add a JsPromise::from_result for convenience by @hansl in https://github.com/boa-dev/boa/pull/4039
- Fix misspelled permissions in nightly build action by @nekevss in https://github.com/boa-dev/boa/pull/4041
- Remove dockerfile from documentation by @4yman-0 in https://github.com/boa-dev/boa/pull/4046
- Bump dependencies with breaking changes by @jedel1043 in https://github.com/boa-dev/boa/pull/4050
- Migrate to fast-float2 by @jedel1043 in https://github.com/boa-dev/boa/pull/4052

## New Contributors

- @magic-akari made their first contribution in https://github.com/boa-dev/boa/pull/3916
- @shurizzle made their first contribution in https://github.com/boa-dev/boa/pull/3976
- @it-a-me made their first contribution in https://github.com/boa-dev/boa/pull/4007
- @Nikita-str made their first contribution in https://github.com/boa-dev/boa/pull/4010
- @4yman-0 made their first contribution in https://github.com/boa-dev/boa/pull/4046

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.19...v0.20.0

# [0.19.1 (2024-09-11)](https://github.com/boa-dev/boa/compare/v0.19...v0.19.1)

### Bug Fixes

- Implement new spec changes for `AsyncGenerator` by @jedel1043 in https://github.com/boa-dev/boa/pull/3950
- Allow dead code for code that is newly detected as unused by @hansl in https://github.com/boa-dev/boa/pull/3984
- Allow warnings when running CI on release branches by @jedel1043 in https://github.com/boa-dev/boa/pull/3990

### Internal Improvements

- Add spec edition 15 to the tester by @jedel1043 in https://github.com/boa-dev/boa/pull/3957
- Enable CI for release branches by @jedel1043 in https://github.com/boa-dev/boa/pull/3987

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.19...v0.19.1

# [0.19.0 (2024-07-08)](https://github.com/boa-dev/boa/compare/v0.18...v0.19)

### Feature Enhancements

- Add release binary striping by @Razican in https://github.com/boa-dev/boa/pull/3727
- Added NPM publish workflow by @Razican in https://github.com/boa-dev/boa/pull/3725
- Remove references to dev docs and npm dependencies by @jedel1043 in https://github.com/boa-dev/boa/pull/3787
- Cleanup tester deps and patterns by @jedel1043 in https://github.com/boa-dev/boa/pull/3792
- Build docs.rs docs with all features enabled by @jedel1043 in https://github.com/boa-dev/boa/pull/3794
- Add a new type Convert<> to convert values by @hansl in https://github.com/boa-dev/boa/pull/3786
- Add functions to create modules from a JSON value by @hansl in https://github.com/boa-dev/boa/pull/3804
- Add an embed_module!() macro to boa_interop by @hansl in https://github.com/boa-dev/boa/pull/3784
- Add a ContextData struct to inject host defined types from the context by @hansl in https://github.com/boa-dev/boa/pull/3802
- Implement object keys access by @HalidOdat in https://github.com/boa-dev/boa/pull/3832
- Group dependabot updates by @jedel1043 in https://github.com/boa-dev/boa/pull/3863
- Adding TryFromJs implementations for BTreeMap and HashMap by @hansl in https://github.com/boa-dev/boa/pull/3844
- Adding TryFromJs implementations for tuples by @hansl in https://github.com/boa-dev/boa/pull/3843
- Add a js_class to implement the Class trait without boilerplate by @hansl in https://github.com/boa-dev/boa/pull/3872
- Implement lossless TryFromJs for integers from f64 by @HalidOdat in https://github.com/boa-dev/boa/pull/3907

### Bug Fixes

- Close for-of iterator when the loop body throws by @raskad in https://github.com/boa-dev/boa/pull/3734
- Add default value handling for destructuring property access arrays by @raskad in https://github.com/boa-dev/boa/pull/3738
- Fix invalid syntax errors for allowed `let` as variable names by @raskad in https://github.com/boa-dev/boa/pull/3743
- Fix parsing of `async` in for-of loops by @raskad in https://github.com/boa-dev/boa/pull/3745
- Fix parsing of binding identifier in try catch parameters by @raskad in https://github.com/boa-dev/boa/pull/3752
- Add missing environment creation in initial iteration of for loop by @raskad in https://github.com/boa-dev/boa/pull/3751
- chore: Update README link to reflect new site paths by @NickTomlin in https://github.com/boa-dev/boa/pull/3793
- Fix order of `ToString` call in `Function` constructor by @HalidOdat in https://github.com/boa-dev/boa/pull/3820
- Fix CI for nextest step by @jedel1043 in https://github.com/boa-dev/boa/pull/3862
- Fix base objects in `with` statements by @raskad in https://github.com/boa-dev/boa/pull/3870
- Fix boa cli history by @raskad in https://github.com/boa-dev/boa/pull/3875
- Fix hashbang comments by using proper goal symbols by @raskad in https://github.com/boa-dev/boa/pull/3876
- Fix AsyncGenerator to correctly handle `return` inside `then` by @jedel1043 in https://github.com/boa-dev/boa/pull/3879
- Fix HomeObject for private class methods by @raskad in https://github.com/boa-dev/boa/pull/3897
- Fix evaluation order in destructive property assignments by @raskad in https://github.com/boa-dev/boa/pull/3895

### Internal Improvements

- Apply new clippy lints for rustc 1.77 by @jedel1043 in https://github.com/boa-dev/boa/pull/3759
- Change dependabot interval to weekly by @jedel1043 in https://github.com/boa-dev/boa/pull/3758
- Dense array storage variants for `i32` and `f64` by @HalidOdat in https://github.com/boa-dev/boa/pull/3760
- Optimize number to `PropertyKey` conversion by @HalidOdat in https://github.com/boa-dev/boa/pull/3769
- don't run test262 on push by @jasonwilliams in https://github.com/boa-dev/boa/pull/3774
- Check that `min <= max` in `clamp_finite` by @jedel1043 in https://github.com/boa-dev/boa/pull/3699
- Decouple `Context` from `ByteCompiler` by @HalidOdat in https://github.com/boa-dev/boa/pull/3829
- Implement latin1 encoded `JsString`s by @HalidOdat in https://github.com/boa-dev/boa/pull/3450
- Replace `js_str` with `js_string` in examples by @getong in https://github.com/boa-dev/boa/pull/3836
- Separate `JsString` into its own crate by @HalidOdat in https://github.com/boa-dev/boa/pull/3837
- Bump temporal_rs to latest commit by @jedel1043 in https://github.com/boa-dev/boa/pull/3880
- Remove `FormalParameterList` from `CodeBlock` by @HalidOdat in https://github.com/boa-dev/boa/pull/3882

### Other Changes

- Fix a few Duration code typos by @robot-head in https://github.com/boa-dev/boa/pull/3730
- Add a try_from_js implementation for Vec<T> (accept any Array-like) by @hansl in https://github.com/boa-dev/boa/pull/3755
- Swap to Duration::round from temporal_rs by @robot-head in https://github.com/boa-dev/boa/pull/3731
- Cache `this` value by @HalidOdat in https://github.com/boa-dev/boa/pull/3771
- Allow deserialization of missing objects properties into Option<> by @hansl in https://github.com/boa-dev/boa/pull/3767
- Optimize Regex match check by @HalidOdat in https://github.com/boa-dev/boa/pull/3779
- Add a boa_interop crate by @hansl in https://github.com/boa-dev/boa/pull/3772
- Add a path to Module (and expose it in Referrer) by @hansl in https://github.com/boa-dev/boa/pull/3783
- Properly resolve paths in SimpleModuleLoader and add path to Referrer::Script by @hansl in https://github.com/boa-dev/boa/pull/3791
- Fix SimpleModuleLoader on Windows by @hansl in https://github.com/boa-dev/boa/pull/3795
- Add more utility traits and funtions to boa_interop by @hansl in https://github.com/boa-dev/boa/pull/3773
- Implement Promise.try() by @linusg in https://github.com/boa-dev/boa/pull/3800
- Implement TryFromJs for Either<L, R> by @hansl in https://github.com/boa-dev/boa/pull/3822
- Fix Rust 1.78.0 Clippy lints by @HalidOdat in https://github.com/boa-dev/boa/pull/3838
- Switch from actions-rs/toolchain to dtolnay/rust-toolchain by @raskad in https://github.com/boa-dev/boa/pull/3845
- Replace archived github actions from actions-rs by @raskad in https://github.com/boa-dev/boa/pull/3848
- Add matrix badge and update communication to include matrix by @nekevss in https://github.com/boa-dev/boa/pull/3865
- Add groupCollapsed by @leoflalv in https://github.com/boa-dev/boa/pull/3867
- Bump ICU4X to 1.5 and cleanup Intl by @jedel1043 in https://github.com/boa-dev/boa/pull/3868
- Update regress to v0.10.0 by @raskad in https://github.com/boa-dev/boa/pull/3869
- Combine `HasProperty` and `Get` operations when possible by @raskad in https://github.com/boa-dev/boa/pull/3883
- Remove some environment clones by @raskad in https://github.com/boa-dev/boa/pull/3884
- Refactor call frame access to avoid panic checks by @raskad in https://github.com/boa-dev/boa/pull/3888
- Remove `Temporal.Calendar` and `Temporal.TimeZone` by @jedel1043 in https://github.com/boa-dev/boa/pull/3890
- Update Temporal rounding and implement additional methods by @nekevss in https://github.com/boa-dev/boa/pull/3892
- format code in comments by @jasonwilliams in https://github.com/boa-dev/boa/pull/3902
- Updates to temporal_rs version and temporal methods by @nekevss in https://github.com/boa-dev/boa/pull/3900
- Patch regression from change to to-relative-to-object by @nekevss in https://github.com/boa-dev/boa/pull/3906
- Add `get_unchecked` method to `JsString` and `JsStr` by @CrazyboyQCD in https://github.com/boa-dev/boa/pull/3898
- bump gc threshold by @jasonwilliams in https://github.com/boa-dev/boa/pull/3908
- update versions and ABOUT files by @jasonwilliams in https://github.com/boa-dev/boa/pull/3903
- Cleanup README.md and contributor documentation by @jedel1043 in https://github.com/boa-dev/boa/pull/3909
- Refactor environment stack to remove some panics by @raskad in https://github.com/boa-dev/boa/pull/3893

## New Contributors

- @robot-head made their first contribution in https://github.com/boa-dev/boa/pull/3730
- @hansl made their first contribution in https://github.com/boa-dev/boa/pull/3755
- @NickTomlin made their first contribution in https://github.com/boa-dev/boa/pull/3793
- @linusg made their first contribution in https://github.com/boa-dev/boa/pull/3800
- @getong made their first contribution in https://github.com/boa-dev/boa/pull/3836
- @leoflalv made their first contribution in https://github.com/boa-dev/boa/pull/3867
- @CrazyboyQCD made their first contribution in https://github.com/boa-dev/boa/pull/3898

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.18...v0.19

# [0.18.0 (2024-03-04)](https://github.com/boa-dev/boa/compare/v0.17...v0.18)

### Feature Enhancements

- Format let-else expressions by @jedel1043 in https://github.com/boa-dev/boa/pull/3102
- Add regexp indices (`d` flag) support by @dirkdev98 in https://github.com/boa-dev/boa/pull/3094
- Add missing 'unscopables' to `Array.prototype[@@unscopables]` by @dirkdev98 in https://github.com/boa-dev/boa/pull/3111
- Updated Fuzzer dependencies and added them to Dependabot by @Razican in https://github.com/boa-dev/boa/pull/3124
- Implement `findLast` and `findLastIndex` on TypedArray by @dirkdev98 in https://github.com/boa-dev/boa/pull/3135
- Implement i128/u128 to JsBigInt conversions by @AlvinKuruvilla in https://github.com/boa-dev/boa/pull/3129
- Implement `String.prototype.isWellFormed` and `String.prototype.toWellFormed` by @raskad in https://github.com/boa-dev/boa/pull/3187
- Clarify usage section in `README.md` by @postmeback in https://github.com/boa-dev/boa/pull/3092
- Log traces even without message (boa_runtime) by @kelbazz in https://github.com/boa-dev/boa/pull/3193
- Implement ephemeron-based weak map by @jedel1043 in https://github.com/boa-dev/boa/pull/3052
- Improve bytecompiler bytecode generation. by @HalidOdat in https://github.com/boa-dev/boa/pull/3188
- Add `Instruction` and `InstructionIterator` by @HalidOdat in https://github.com/boa-dev/boa/pull/3201
- Add ECMAScript 14 to `boa_tester` by @jedel1043 in https://github.com/boa-dev/boa/pull/3273
- Bump `rust-version` to 1.71 by @jedel1043 in https://github.com/boa-dev/boa/pull/3290
- Lazily download `test262` repository by @HalidOdat in https://github.com/boa-dev/boa/pull/3214
- Implement `Gc::new_cyclic` by @jedel1043 in https://github.com/boa-dev/boa/pull/3292
- Implement `Intl.PluralRules` by @jedel1043 in https://github.com/boa-dev/boa/pull/3298
- Implement step 5 in `RegExp` constructor by @HalidOdat in https://github.com/boa-dev/boa/pull/3305
- Replace #[deny] with #[warn] by @jedel1043 in https://github.com/boa-dev/boa/pull/3309
- Bump ICU4X to 1.3 by @jedel1043 in https://github.com/boa-dev/boa/pull/3306
- Migrate to workspace deps by @jedel1043 in https://github.com/boa-dev/boa/pull/3313
- Implement `[[HostDefined]]` field on `Realm`s by @HalidOdat in https://github.com/boa-dev/boa/pull/2952
- Introduce experimental features by @jedel1043 in https://github.com/boa-dev/boa/pull/3318
- Introduce a `Class` map by @jedel1043 in https://github.com/boa-dev/boa/pull/3315
- Fix `Function.prototype.toString()` by @HalidOdat in https://github.com/boa-dev/boa/pull/3374
- First portion of the Temporal implementation by @nekevss in https://github.com/boa-dev/boa/pull/3277
- Update feature flags to specific feature flag by @nekevss in https://github.com/boa-dev/boa/pull/3376
- Implement `[[HostDefined]]` for `Module` and `Script` by @arexon in https://github.com/boa-dev/boa/pull/3381
- Implement synthetic modules by @jedel1043 in https://github.com/boa-dev/boa/pull/3294
- Prevent `test262` repository update if not needed by @HalidOdat in https://github.com/boa-dev/boa/pull/3386
- Implement `SharedArrayBuffer` by @jedel1043 in https://github.com/boa-dev/boa/pull/3384
- Add `Context::create_realm` by @johnyob in https://github.com/boa-dev/boa/pull/3369
- Introduce a thread safe version of `JsError` by @jedel1043 in https://github.com/boa-dev/boa/pull/3398
- Implement asynchronous evaluation of scripts by @jedel1043 in https://github.com/boa-dev/boa/pull/3044
- Feature `get/set $boa.limits.stack` by @HalidOdat in https://github.com/boa-dev/boa/pull/3385
- Implement `change-array-by-copy` methods by @jedel1043 in https://github.com/boa-dev/boa/pull/3412
- Implement the `array-grouping` proposal by @jedel1043 in https://github.com/boa-dev/boa/pull/3420
- Implement `Atomics` builtin by @jedel1043 in https://github.com/boa-dev/boa/pull/3394
- Migrate to workspace lints by @jedel1043 in https://github.com/boa-dev/boa/pull/3334
- Bump ICU4X to 1.4 and finish Intl impls with new APIs by @jedel1043 in https://github.com/boa-dev/boa/pull/3469
- Class: Switch `make_data` parameter from `this` to `new_target` by @johnyob in https://github.com/boa-dev/boa/pull/3478
- Add utility methods to the `Class` trait by @jedel1043 in https://github.com/boa-dev/boa/pull/3488
- Simplify `Icu` API by @jedel1043 in https://github.com/boa-dev/boa/pull/3503
- Add UTF-16 input parsing by @raskad in https://github.com/boa-dev/boa/pull/3538
- Remove allocations from `HostDefined::get_many_mut` by @jedel1043 in https://github.com/boa-dev/boa/pull/3606
- Implement getter for `ArrayBuffer` data by @HalidOdat in https://github.com/boa-dev/boa/pull/3610
- Implement non-erased `JsObject`s by @jedel1043 in https://github.com/boa-dev/boa/pull/3618
- Update regress to v0.8.0 and use UTF16 / UCS2 matching by @raskad in https://github.com/boa-dev/boa/pull/3627
- Cleanup 262 tester and stabilize some experimental features by @jedel1043 in https://github.com/boa-dev/boa/pull/3632
- Improve typing of `DataView` and related objects by @jedel1043 in https://github.com/boa-dev/boa/pull/3626
- Close sync iterator when async wrapper yields rejection by @jedel1043 in https://github.com/boa-dev/boa/pull/3633
- Implement resizable buffers by @jedel1043 in https://github.com/boa-dev/boa/pull/3634
- Implement stage 3 feature "arraybuffer-transfer" by @jedel1043 in https://github.com/boa-dev/boa/pull/3649
- Implement prototype of `NumberFormat` by @jedel1043 in https://github.com/boa-dev/boa/pull/3669
- Add example for async module fetches by @jedel1043 in https://github.com/boa-dev/boa/pull/3012
- Js typed array methods by @AngeloChecked in https://github.com/boa-dev/boa/pull/3481
- Create tool to regenerate `ABOUT.md` by @jedel1043 in https://github.com/boa-dev/boa/pull/3692
- Implement RegExp `v` flag by @raskad in https://github.com/boa-dev/boa/pull/3695

### Bug Fixes

- Allow escaped yield and await in labelled statement by @raskad in https://github.com/boa-dev/boa/pull/3117
- `TypedArray.prototype.values()` and `TypedArray.prototype[@@iterator]` should be equal by @HalidOdat in https://github.com/boa-dev/boa/pull/3096
- Fix TypedArrayConstructors tests by @raskad in https://github.com/boa-dev/boa/pull/3171
- Close iterator after generator return call while array destructuring assignment by @HalidOdat in https://github.com/boa-dev/boa/pull/3164
- Fix remaining TypedArray bugs by @raskad in https://github.com/boa-dev/boa/pull/3186
- Add early errors for `LexicalDeclaration` by @raskad in https://github.com/boa-dev/boa/pull/3207
- Fix switch statement `break` and `continue` return values by @raskad in https://github.com/boa-dev/boa/pull/3205
- Fix GitHub coverage workflow by @HalidOdat in https://github.com/boa-dev/boa/pull/3288
- Fix tagged template `this` in strict mode by @HalidOdat in https://github.com/boa-dev/boa/pull/3307
- fix: add 'static lifetime by @mattsse in https://github.com/boa-dev/boa/pull/3297
- Fix class inherit from `null` by @HalidOdat in https://github.com/boa-dev/boa/pull/3312
- Fix anonymous function name in cover assignment by @raskad in https://github.com/boa-dev/boa/pull/3325
- Add `NonMaxU32` as integer variant for `PropertyKey` by @raskad in https://github.com/boa-dev/boa/pull/3321
- Add missing class name binding by @raskad in https://github.com/boa-dev/boa/pull/3328
- Truncate environment stack on non-caught native error by @HalidOdat in https://github.com/boa-dev/boa/pull/3331
- Fix regular expression construction by @HalidOdat in https://github.com/boa-dev/boa/pull/3338
- Fix `super()` construction with default parameters by @HalidOdat in https://github.com/boa-dev/boa/pull/3339
- Fix static class element evaluation order by @raskad in https://github.com/boa-dev/boa/pull/3327
- Fix detection of runtime limits for accessors by @jedel1043 in https://github.com/boa-dev/boa/pull/3335
- Fix `Number.prototype.toFixed()` by @HalidOdat in https://github.com/boa-dev/boa/pull/2898
- Check `eval` realm before call by @HalidOdat in https://github.com/boa-dev/boa/pull/3375
- Evaluate all parts of `class` in strict mode by @HalidOdat in https://github.com/boa-dev/boa/pull/3383
- Fix var declaration for deleted binding locator by @raskad in https://github.com/boa-dev/boa/pull/3387
- Fix await flag in class constructor by @raskad in https://github.com/boa-dev/boa/pull/3388
- Fix compilation for targets without `AtomicU64` by @jedel1043 in https://github.com/boa-dev/boa/pull/3399
- Update `regex.match` spec and code by @raskad in https://github.com/boa-dev/boa/pull/3462
- `Context` independent `CodeBlock`s by @HalidOdat in https://github.com/boa-dev/boa/pull/3424
- Fix a Parser Idempotency Issue by @veera-sivarajan in https://github.com/boa-dev/boa/pull/3172
- Non recursive gc trace by @HalidOdat in https://github.com/boa-dev/boa/pull/3508
- Fix invalid return value when closing an iterator by @raskad in https://github.com/boa-dev/boa/pull/3567
- Implement Date parsing according to the spec by @raskad in https://github.com/boa-dev/boa/pull/3564
- `Date` refactor by @raskad in https://github.com/boa-dev/boa/pull/3595
- Fix regexp `toString` method by @raskad in https://github.com/boa-dev/boa/pull/3608
- Fix escaping in `RegExp.prototype.source` by @raskad in https://github.com/boa-dev/boa/pull/3619
- Fix line terminators in template strings by @raskad in https://github.com/boa-dev/boa/pull/3641
- Consider strict + no-strict tests as a single test by @jedel1043 in https://github.com/boa-dev/boa/pull/3675
- Preserve `.exe` suffix for Windows releases by @HalidOdat in https://github.com/boa-dev/boa/pull/3680

### Internal Improvements

- Move `RefCell` of `CompileTimeEnvironment`s to field `bindings` by @HalidOdat in https://github.com/boa-dev/boa/pull/3108
- Change `name` field type in `CodeBlock` to `JsString` by @HalidOdat in https://github.com/boa-dev/boa/pull/3107
- Refactor `Array.prototype.find*` and TypedArray variants to use `FindViaPredicate` by @dirkdev98 in https://github.com/boa-dev/boa/pull/3134
- Fix 1.71.0 lints by @RageKnify in https://github.com/boa-dev/boa/pull/3140
- Clippy updates: add panics and etc. by @nekevss in https://github.com/boa-dev/boa/pull/3235
- Remove unused class environments by @raskad in https://github.com/boa-dev/boa/pull/3332
- Improve highlighter performance by @jedel1043 in https://github.com/boa-dev/boa/pull/3341
- Cleanup `get_option` and calls to the function by @jedel1043 in https://github.com/boa-dev/boa/pull/3355
- Fix new lints for Rust 1.73 by @jedel1043 in https://github.com/boa-dev/boa/pull/3361
- Refactor compile time environment handling by @raskad in https://github.com/boa-dev/boa/pull/3365
- Update all dependencies by @jedel1043 in https://github.com/boa-dev/boa/pull/3400
- Optimize `shift` for dense arrays by @jedel1043 in https://github.com/boa-dev/boa/pull/3405
- Disallow changing type of already created objects by @jedel1043 in https://github.com/boa-dev/boa/pull/3410
- Merge `CodeBlock` constant pools by @HalidOdat in https://github.com/boa-dev/boa/pull/3413
- Move ordinary function `[[ConstructorKind]]` to `CodeBlock` by @HalidOdat in https://github.com/boa-dev/boa/pull/3439
- Move `FunctionKind` to `CodeBlock` by @HalidOdat in https://github.com/boa-dev/boa/pull/3440
- Unify generator and ordinary function creation by @HalidOdat in https://github.com/boa-dev/boa/pull/3441
- Move `arguments` object creation to bytecode by @HalidOdat in https://github.com/boa-dev/boa/pull/3432
- Move parameter environment creation to bytecode by @HalidOdat in https://github.com/boa-dev/boa/pull/3433
- Prevent `DefVar` opcode emit for global binding by @HalidOdat in https://github.com/boa-dev/boa/pull/3453
- Transition `Intl` types to `NativeObject` API by @jedel1043 in https://github.com/boa-dev/boa/pull/3491
- Reduce `WeakGc<T>` memory usage by @HalidOdat in https://github.com/boa-dev/boa/pull/3492
- Migrate `Temporal` to its own crate. by @nekevss in https://github.com/boa-dev/boa/pull/3461
- Reestructure repo and CI improvements by @jedel1043 in https://github.com/boa-dev/boa/pull/3505
- Move `PromiseCapability` to stack by @HalidOdat in https://github.com/boa-dev/boa/pull/3528
- Fix rust 1.75 lints by @raskad in https://github.com/boa-dev/boa/pull/3540
- Remove double indirection in module types by @jedel1043 in https://github.com/boa-dev/boa/pull/3640
- Fix clippy warnings for rustc 1.76 by @jedel1043 in https://github.com/boa-dev/boa/pull/3668
- Migrate to `temporal_rs` crate by @nekevss in https://github.com/boa-dev/boa/pull/3694

### Other Changes

- Removed time 0.1 dependency, updated dependencies by @Razican in https://github.com/boa-dev/boa/pull/3122
- Add new CLI options to usage in README by @Razican in https://github.com/boa-dev/boa/pull/3123
- Find roots when running GC rather than runtime by @tunz in https://github.com/boa-dev/boa/pull/3109
- Re-enable must_use clippy rule by @tunz in https://github.com/boa-dev/boa/pull/3180
- Refactor environment, exception handling and jumping in VM by @HalidOdat in https://github.com/boa-dev/boa/pull/3059
- Refactor `Context::run()` method by @HalidOdat in https://github.com/boa-dev/boa/pull/3179
- Added examples by @postmeback in https://github.com/boa-dev/boa/pull/3141
- Use main stack for calling ordinary functions by @HalidOdat in https://github.com/boa-dev/boa/pull/3185
- Update license field following SPDX 2.1 license expression standard by @frisoft in https://github.com/boa-dev/boa/pull/3209
- Store active runnable and active function in `CallFrame` by @HalidOdat in https://github.com/boa-dev/boa/pull/3197
- Added MSRV check by @Razican in https://github.com/boa-dev/boa/pull/3291
- Reintroduce publish CI job by @jedel1043 in https://github.com/boa-dev/boa/pull/3308
- Format code snippets in docs by @jedel1043 in https://github.com/boa-dev/boa/pull/3317
- Remove direct conversion from `&str` to `JsValue`/`PropertyKey`. by @jedel1043 in https://github.com/boa-dev/boa/pull/3319
- `icu_properties` default features to true by @nekevss in https://github.com/boa-dev/boa/pull/3326
- Varying length instruction operands by @HalidOdat in https://github.com/boa-dev/boa/pull/3253
- Improve CI testing by @jedel1043 in https://github.com/boa-dev/boa/pull/3333
- Refactor function internal methods by @HalidOdat in https://github.com/boa-dev/boa/pull/3322
- Make environments opcodes use varying operands by @HalidOdat in https://github.com/boa-dev/boa/pull/3340
- Bump test262 by @jedel1043 in https://github.com/boa-dev/boa/pull/3349
- Refactor ordinary VM calling by @HalidOdat in https://github.com/boa-dev/boa/pull/3295
- Fix Array.join when the array contains itself by @ahaoboy in https://github.com/boa-dev/boa/pull/3406
- Rename master workflow to main by @Razican in https://github.com/boa-dev/boa/pull/3409
- Cleaned up a couple of Github action warnings by @Razican in https://github.com/boa-dev/boa/pull/3417
- Temporal duration update and cleanup by @nekevss in https://github.com/boa-dev/boa/pull/3443
- Progress on Duration's round/total method updates by @nekevss in https://github.com/boa-dev/boa/pull/3451
- Simplify all extensions APIs of `Context` by @jedel1043 in https://github.com/boa-dev/boa/pull/3456
- `[[HostDefined]]` Improvements by @johnyob in https://github.com/boa-dev/boa/pull/3460
- Make well_known_symbols functions pub by @tj825 in https://github.com/boa-dev/boa/pull/3465
- Use `Vec<T>` for keeping track of gc objects by @HalidOdat in https://github.com/boa-dev/boa/pull/3493
- Implement `Inline Caching` by @HalidOdat in https://github.com/boa-dev/boa/pull/2767
- Migrate `ISO8601` parsing to `boa_temporal` by @nekevss in https://github.com/boa-dev/boa/pull/3500
- Implement erased objects by @jedel1043 in https://github.com/boa-dev/boa/pull/3494
- Build out ZonedDateTime, TimeZone, and Instant by @nekevss in https://github.com/boa-dev/boa/pull/3497
- `boa_temporal` structure changes and docs update by @nekevss in https://github.com/boa-dev/boa/pull/3504
- Refactor vm calling convention to allow locals by @HalidOdat in https://github.com/boa-dev/boa/pull/3496
- Temporal Parser Cleanup/Fixes by @nekevss in https://github.com/boa-dev/boa/pull/3521
- Refactor Temporal Calendar API for `AnyCalendar` and fields by @nekevss in https://github.com/boa-dev/boa/pull/3522
- Update `boa_temporal` Time Zone design by @nekevss in https://github.com/boa-dev/boa/pull/3543
- Implement `DifferenceInstant` and related refactor by @nekevss in https://github.com/boa-dev/boa/pull/3568
- Run `cargo update` on fuzz crate by @jedel1043 in https://github.com/boa-dev/boa/pull/3607
- Temporal `Instant` migration cont. and related changes by @nekevss in https://github.com/boa-dev/boa/pull/3601
- Temporal: Update `Date` builtin with `boa_temporal` and fixes by @nekevss in https://github.com/boa-dev/boa/pull/3614
- Temporal: Build out `Time` and its methods by @nekevss in https://github.com/boa-dev/boa/pull/3613
- Temporal: Enable temporal tests by @nekevss in https://github.com/boa-dev/boa/pull/3620
- Fix tests results upload by @raskad in https://github.com/boa-dev/boa/pull/3635
- Temporal: `DateTime` and `PlainDateTime` functionality by @nekevss in https://github.com/boa-dev/boa/pull/3628
- Temporal: Initial `PlainTime` build out by @nekevss in https://github.com/boa-dev/boa/pull/3621
- Ignore `Cargo.lock` in fuzzer by @jedel1043 in https://github.com/boa-dev/boa/pull/3636
- Temporal: attribute/property and custom calendar fixes by @nekevss in https://github.com/boa-dev/boa/pull/3639
- Docs: Update boa's main README.md by @nekevss in https://github.com/boa-dev/boa/pull/3650
- Bump time from 0.3.31 to 0.3.33 by @jedel1043 in https://github.com/boa-dev/boa/pull/3652
- Temporal: Refactor Calendar protocol for `JsObject`s by @nekevss in https://github.com/boa-dev/boa/pull/3651
- Simplify Temporal APIs by @jedel1043 in https://github.com/boa-dev/boa/pull/3653
- Implement inline caching tests and cleanup by @HalidOdat in https://github.com/boa-dev/boa/pull/3513
- Docs: Update README.md and add `boa_cli`'s README.md by @nekevss in https://github.com/boa-dev/boa/pull/3659
- Change `HostEnsureCanCompileStrings` to the new spec by @jedel1043 in https://github.com/boa-dev/boa/pull/3690
- Split ICU4X data generation from `boa_icu_provider` by @jedel1043 in https://github.com/boa-dev/boa/pull/3682
- Add a catch all for other categories not labelled by @jasonwilliams in https://github.com/boa-dev/boa/pull/3703
- Fix `temporal_rs` in Cargo.toml by @nekevss in https://github.com/boa-dev/boa/pull/3702

## New Contributors

- @AlvinKuruvilla made their first contribution in https://github.com/boa-dev/boa/pull/3129
- @tunz made their first contribution in https://github.com/boa-dev/boa/pull/3109
- @postmeback made their first contribution in https://github.com/boa-dev/boa/pull/3092
- @kelbazz made their first contribution in https://github.com/boa-dev/boa/pull/3193
- @frisoft made their first contribution in https://github.com/boa-dev/boa/pull/3209
- @mattsse made their first contribution in https://github.com/boa-dev/boa/pull/3297
- @arexon made their first contribution in https://github.com/boa-dev/boa/pull/3381
- @johnyob made their first contribution in https://github.com/boa-dev/boa/pull/3369
- @ahaoboy made their first contribution in https://github.com/boa-dev/boa/pull/3406
- @tj825 made their first contribution in https://github.com/boa-dev/boa/pull/3465
- @AngeloChecked made their first contribution in https://github.com/boa-dev/boa/pull/3481

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.17...v0.18

# [0.17.0 (2023-07-05)](https://github.com/boa-dev/boa/compare/v0.16...v0.17)

### Feature Enhancements

- Implement `new.target` expression by @raskad in [#2299](https://github.com/boa-dev/boa/pull/2299)
- Parse static async private methods in classes by @raskad in [#2315](https://github.com/boa-dev/boa/pull/2315)
- Implement `JsDataView` by @nekevss in [#2308](https://github.com/boa-dev/boa/pull/2308)
- Upgrade clap to 4.0, add value hints for zsh and fish by @Razican in [#2336](https://github.com/boa-dev/boa/pull/2336)
- Implement `JsRegExp` by @nekevss in [#2326](https://github.com/boa-dev/boa/pull/2326)
- Create new lazy Error type by @jedel1043 in [#2283](https://github.com/boa-dev/boa/pull/2283)
- Fixed some documentation and clippy warnings in tests by @Razican in [#2362](https://github.com/boa-dev/boa/pull/2362)
- Removed the "VM Implementation" headline for Test262 by @Razican in [#2364](https://github.com/boa-dev/boa/pull/2364)
- Modified the `loadfile` example to show how Boa can read bytes by @Razican in [#2363](https://github.com/boa-dev/boa/pull/2363)
- Implement `LabelledStatement` by @jedel1043 in [#2349](https://github.com/boa-dev/boa/pull/2349)
- Split vm/opcode into modules by @nekevss in [#2343](https://github.com/boa-dev/boa/pull/2343)
- Removed some duplicate code, added `ToIndentedString` by @Razican in [#2367](https://github.com/boa-dev/boa/pull/2367)
- Document the AST by @jedel1043 in [#2377](https://github.com/boa-dev/boa/pull/2377)
- Implement member accessors in initializer of for loops by @jedel1043 in [#2381](https://github.com/boa-dev/boa/pull/2381)
- Implement `JsGenerator` and wrapper docs clean up by @nekevss in [#2380](https://github.com/boa-dev/boa/pull/2380)
- Add named evaluation of logical assignments by @raskad in [#2389](https://github.com/boa-dev/boa/pull/2389)
- Implement optional chains by @jedel1043 in [#2390](https://github.com/boa-dev/boa/pull/2390)
- Implement delete for references by @jedel1043 in [#2395](https://github.com/boa-dev/boa/pull/2395)
- Implement AST Visitor pattern (attempt #3) by @addisoncrump in [#2392](https://github.com/boa-dev/boa/pull/2392)
- Implement async arrow functions by @raskad in [#2393](https://github.com/boa-dev/boa/pull/2393)
- Pretty print promise objects by @jedel1043 in [#2407](https://github.com/boa-dev/boa/pull/2407)
- Parser Idempotency Fuzzer by @addisoncrump in [#2400](https://github.com/boa-dev/boa/pull/2400)
- Safe wrapper for `JsDate` by @anuvratsingh in [#2181](https://github.com/boa-dev/boa/pull/2181)
- Fix some Date tests by @jedel1043 in [#2431](https://github.com/boa-dev/boa/pull/2431)
- Boa Gc implementation draft by @nekevss in [#2394](https://github.com/boa-dev/boa/pull/2394)
- VM Fuzzer by @addisoncrump in [#2401](https://github.com/boa-dev/boa/pull/2401)
- Implement the `WeakRef` builtin by @jedel1043 in [#2438](https://github.com/boa-dev/boa/pull/2438)
- Refactor the `Date` builtin by @jedel1043 in [#2449](https://github.com/boa-dev/boa/pull/2449)
- Implement instruction flowgraph generator by @HalidOdat in [#2422](https://github.com/boa-dev/boa/pull/2422)
- `JsArrayBuffer` take method and docs by @nekevss in [#2454](https://github.com/boa-dev/boa/pull/2454)
- Set function names in object literal methods by @raskad in [#2460](https://github.com/boa-dev/boa/pull/2460)
- Redesign Intl API and implement some services by @jedel1043 in [#2478](https://github.com/boa-dev/boa/pull/2478)
- Cleanup `Context` APIs by @jedel1043 in [#2504](https://github.com/boa-dev/boa/pull/2504)
- Prepare `Promises` for new host hooks and job queue API by @jedel1043 in [#2528](https://github.com/boa-dev/boa/pull/2528)
- Implement host hooks and job queues APIs by @jedel1043 in [#2529](https://github.com/boa-dev/boa/pull/2529)
- First batch of `no_std` support for some sub-crates by @jedel1043 in [#2544](https://github.com/boa-dev/boa/pull/2544)
- Create `Source` to abstract JS code sources by @jedel1043 in [#2579](https://github.com/boa-dev/boa/pull/2579)
- Move increment and decrement operations to `Update` expression by @raskad in [#2565](https://github.com/boa-dev/boa/pull/2565)
- Implement binary `in` operation with private names by @raskad in [#2582](https://github.com/boa-dev/boa/pull/2582)
- Module parsing by @Razican in [#2411](https://github.com/boa-dev/boa/pull/2411)
- Implement `WeakSet` by @lupd in [#2586](https://github.com/boa-dev/boa/pull/2586)
- Implement `WeakMap` by @raskad in [#2597](https://github.com/boa-dev/boa/pull/2597)
- API to construct a `NativeFunction` from a native async function by @jedel1043 in [#2542](https://github.com/boa-dev/boa/pull/2542)
- Add `--strict` flag to cli by @HalidOdat in [#2689](https://github.com/boa-dev/boa/pull/2689)
- Add timeout to CI by @HalidOdat in [#2691](https://github.com/boa-dev/boa/pull/2691)
- Add ES5 and ES6 Conformance calculation to boa_tester by @ZackMitkin in [#2690](https://github.com/boa-dev/boa/pull/2690)
- Improve tester display for multiple editions by @jedel1043 in [#2720](https://github.com/boa-dev/boa/pull/2720)
- Implement `JsPromise` wrapper by @jedel1043 in [#2758](https://github.com/boa-dev/boa/pull/2758)
- Initial version of a JS -> Rust conversion trait. by @Razican in [#2276](https://github.com/boa-dev/boa/pull/2276)
- Implement `escape` and `unescape` by @jedel1043 in [#2768](https://github.com/boa-dev/boa/pull/2768)
- Implement debug object for CLI by @HalidOdat in [#2772](https://github.com/boa-dev/boa/pull/2772)
- Make `Realm` shareable between functions by @jedel1043 in [#2801](https://github.com/boa-dev/boa/pull/2801)
- Implement Annex-B string html methods by @HalidOdat in [#2798](https://github.com/boa-dev/boa/pull/2798)
- Implement annex-b `trimLeft` and `trimRight` string methods by @HalidOdat in [#2806](https://github.com/boa-dev/boa/pull/2806)
- Implement HTML comments and gate behind the `annex-b` feature by @jedel1043 in [#2817](https://github.com/boa-dev/boa/pull/2817)
- Implement `Intl.Segmenter` by @jedel1043 in [#2840](https://github.com/boa-dev/boa/pull/2840)
- Implement var initializers in for-in loops by @jedel1043 in [#2842](https://github.com/boa-dev/boa/pull/2842)
- Improve debug output of `JsNativeError` and `Realm` by @jedel1043 in [#2894](https://github.com/boa-dev/boa/pull/2894)
- Implement runtime limits for loops by @HalidOdat in [#2857](https://github.com/boa-dev/boa/pull/2857)
- Implement runtime limits for recursion by @HalidOdat in [#2904](https://github.com/boa-dev/boa/pull/2904)
- Implement annexB Block-Level Function Declarations by @raskad in [#2910](https://github.com/boa-dev/boa/pull/2910)
- Implement module execution by @jedel1043 in [#2922](https://github.com/boa-dev/boa/pull/2922)
- Type safe root shape by @HalidOdat in [#2940](https://github.com/boa-dev/boa/pull/2940)
- Implement dynamic imports by @jedel1043 in [#2932](https://github.com/boa-dev/boa/pull/2932)
- Implement pseudo-property `import.meta` by @jedel1043 in [#2956](https://github.com/boa-dev/boa/pull/2956)
- Implement `with` and object environments by @raskad in [#2692](https://github.com/boa-dev/boa/pull/2692)
- Add hooks to get the current time and timezone by @jedel1043 in [#2824](https://github.com/boa-dev/boa/pull/2824)
- Make `JsSymbol` thread-safe by @jedel1043 in [#2539](https://github.com/boa-dev/boa/pull/2539)
- Added a Boa runtime by @Razican in [#2743](https://github.com/boa-dev/boa/pull/2743)
- Implement `TryFromJs` for `JsObject` wrappers by @Razican in [#2809](https://github.com/boa-dev/boa/pull/2809)
- Allow passing owned `HostHooks` and `JobQueues` to `Context` by @jedel1043 in [#2811](https://github.com/boa-dev/boa/pull/2811)
- Implement `String.prototype.toLocaleUpper/LowerCase` by @jedel1043 in [#2822](https://github.com/boa-dev/boa/pull/2822)
- Implement constant folding optimization by @HalidOdat in [#2679](https://github.com/boa-dev/boa/pull/2679)
- Implement `Hidden classes` by @HalidOdat in [#2723](https://github.com/boa-dev/boa/pull/2723)
- show object kind, name and address when using dbg! by @jasonwilliams in [#2960](https://github.com/boa-dev/boa/pull/2960)
- Add convenience methods to `ModuleLoader` by @jedel1043 in [#3007](https://github.com/boa-dev/boa/pull/3007)
- Allow `JobQueue` to concurrently run jobs by @jedel1043 in [#3036](https://github.com/boa-dev/boa/pull/3036)
- Make `IntegerIndexed::byte_offset` public by @CryZe in [#3017](https://github.com/boa-dev/boa/pull/3017)
- Allow awaiting `JsPromise` from Rust code by @jedel1043 in [#3011](https://github.com/boa-dev/boa/pull/3011)
- Cache `cargo-tarpaulin` binary by @jedel1043 in [#3071](https://github.com/boa-dev/boa/pull/3071)
- add link to the main logo by @jasonwilliams in [#3082](https://github.com/boa-dev/boa/pull/3082)

### Bug Fixes

- Add unicode terminator to line comment by @creampnx-x in [#2301](https://github.com/boa-dev/boa/pull/2301)
- Fix function property order by @raskad in [#2305](https://github.com/boa-dev/boa/pull/2305)
- Fix some Array spec deviations by @raskad in [#2306](https://github.com/boa-dev/boa/pull/2306)
- Fix double conversion to primitive in `ToNumeric` by @raskad in [#2310](https://github.com/boa-dev/boa/pull/2310)
- Fixing the output for the test diffs in PRs by @Razican in [#2320](https://github.com/boa-dev/boa/pull/2320)
- Fix Regex literal parsing in MemberExpression by @tunz in [#2328](https://github.com/boa-dev/boa/pull/2328)
- Fix error in `Proxy` set implementation by @raskad in [#2369](https://github.com/boa-dev/boa/pull/2369)
- Allow LineTerminator before Semicolon in `continue` by @raskad in [#2371](https://github.com/boa-dev/boa/pull/2371)
- Fix var collisions in strict eval calls by @jedel1043 in [#2382](https://github.com/boa-dev/boa/pull/2382)
- Set `in` to `true` when parsing AssignmentExpression in ConditionalExpression by @raskad in [#2386](https://github.com/boa-dev/boa/pull/2386)
- Skip prototype field definition for arrow function by @raskad in [#2388](https://github.com/boa-dev/boa/pull/2388)
- Remove invalid optimization in addition by @raskad in [#2387](https://github.com/boa-dev/boa/pull/2387)
- Fix order dependent execution in assignment. by @HalidOdat in [#2378](https://github.com/boa-dev/boa/pull/2378)
- Add early error for `yield` in `GeneratorExpression` parameters by @raskad in [#2413](https://github.com/boa-dev/boa/pull/2413)
- Handle `__proto__` fields in object literals by @raskad in [#2423](https://github.com/boa-dev/boa/pull/2423)
- Fix built-ins/Array/prototype/toString/non-callable-join-string-tag.js test case by @akavi in [#2458](https://github.com/boa-dev/boa/pull/2458)
- Fix `PartialEq` for `JsBigInt` and `f64` by @raskad in [#2461](https://github.com/boa-dev/boa/pull/2461)
- Allow class expressions without identifier by @raskad in [#2464](https://github.com/boa-dev/boa/pull/2464)
- Fix to weak_trace for `boa_tester` by @nekevss in [#2470](https://github.com/boa-dev/boa/pull/2470)
- Fix unary operations on `this` by @veera-sivarajan in [#2507](https://github.com/boa-dev/boa/pull/2507)
- Fix postfix operator line terminator parsing by @raskad in [#2520](https://github.com/boa-dev/boa/pull/2520)
- Remove `Literal::Undefined` by @veera-sivarajan in [#2518](https://github.com/boa-dev/boa/pull/2518)
- Add early errors for 'eval' or 'arguments' in parameters by @raskad in [#2515](https://github.com/boa-dev/boa/pull/2515)
- Pass a receiver value in property getter opcodes by @raskad in [#2516](https://github.com/boa-dev/boa/pull/2516)
- Add regex literal early errors by @raskad in [#2517](https://github.com/boa-dev/boa/pull/2517)
- `Break` Opcode and `ByteCompiler` changes by @nekevss in [#2523](https://github.com/boa-dev/boa/pull/2523)
- Refactor some class features by @raskad in [#2513](https://github.com/boa-dev/boa/pull/2513)
- Recognize Directive Prologues correctly by @raskad in [#2521](https://github.com/boa-dev/boa/pull/2521)
- Correctly parse consecutive semicolons by @raskad in [#2533](https://github.com/boa-dev/boa/pull/2533)
- Fix some HoistableDeclaration parsing errors by @raskad in [#2532](https://github.com/boa-dev/boa/pull/2532)
- Return the correct value from a statement list by @raskad in [#2554](https://github.com/boa-dev/boa/pull/2554)
- Fix error for static class methods named `prototype` by @raskad in [#2552](https://github.com/boa-dev/boa/pull/2552)
- Avoid creating `prototype` property on methods by @raskad in [#2553](https://github.com/boa-dev/boa/pull/2553)
- Fix double property access on assignment ops by @raskad in [#2551](https://github.com/boa-dev/boa/pull/2551)
- Add early errors for escaped identifiers by @raskad in [#2546](https://github.com/boa-dev/boa/pull/2546)
- Fix failing collator tests by @jedel1043 in [#2575](https://github.com/boa-dev/boa/pull/2575)
- fuzzer: bubble up NoInstructionsRemain error instead of trying to handle as exception by @Mrmaxmeier in [#2566](https://github.com/boa-dev/boa/pull/2566)
- Try-catch-block control flow fix/refactor by @nekevss in [#2568](https://github.com/boa-dev/boa/pull/2568)
- Fix doc tests and add CI check by @jedel1043 in [#2606](https://github.com/boa-dev/boa/pull/2606)
- Fix string to number conversion for `infinity` by @raskad in [#2607](https://github.com/boa-dev/boa/pull/2607)
- Fix exponent operator by @HalidOdat in [#2681](https://github.com/boa-dev/boa/pull/2681)
- Update `README.md` cli options by @HalidOdat in [#2678](https://github.com/boa-dev/boa/pull/2678)
- Fix incorrect `Number.MIN_VALUE` value by @HalidOdat in [#2682](https://github.com/boa-dev/boa/pull/2682)
- Correctly run async tests by @jedel1043 in [#2683](https://github.com/boa-dev/boa/pull/2683)
- Fix value to bigint conversion by @HalidOdat in [#2688](https://github.com/boa-dev/boa/pull/2688)
- Fix Object constructor by @raskad in [#2694](https://github.com/boa-dev/boa/pull/2694)
- Fix get function opcode traces by @HalidOdat in [#2708](https://github.com/boa-dev/boa/pull/2708)
- Add early errors to dynamic function constructors by @raskad in [#2716](https://github.com/boa-dev/boa/pull/2716)
- Add negative zero handling for `Map.delete` by @raskad in [#2726](https://github.com/boa-dev/boa/pull/2726)
- Fix remaining `Set` tests by @raskad in [#2725](https://github.com/boa-dev/boa/pull/2725)
- Fix update expressions getting values multiple times by @raskad in [#2733](https://github.com/boa-dev/boa/pull/2733)
- Make if statements return their completion values by @raskad in [#2739](https://github.com/boa-dev/boa/pull/2739)
- Fix super call execution order by @raskad in [#2724](https://github.com/boa-dev/boa/pull/2724)
- Fix deserialization of `SpecEdition` by @jedel1043 in [#2762](https://github.com/boa-dev/boa/pull/2762)
- Add `json-parse-with-source` feature to `boa_tester` by @HalidOdat in [#2778](https://github.com/boa-dev/boa/pull/2778)
- Fix `Symbol.prototype[@@iterator]` by @HalidOdat in [#2800](https://github.com/boa-dev/boa/pull/2800)
- Fix `String.prototype.replace()` order of `ToString` execution by @HalidOdat in [#2799](https://github.com/boa-dev/boa/pull/2799)
- Fix `ThrowTypeError` intrinsic by @HalidOdat in [#2797](https://github.com/boa-dev/boa/pull/2797)
- Fix `String.prototype.substr()` by @HalidOdat in [#2805](https://github.com/boa-dev/boa/pull/2805)
- Fix destructive for-of loop assignments by @raskad in [#2803](https://github.com/boa-dev/boa/pull/2803)
- Fix `TypedArray`s minus zero key by @HalidOdat in [#2808](https://github.com/boa-dev/boa/pull/2808)
- Fix sync generator yield expressions by @raskad in [#2838](https://github.com/boa-dev/boa/pull/2838)
- Fix async generators by @raskad in [#2853](https://github.com/boa-dev/boa/pull/2853)
- Catch 'eval' and 'arguments' in setter method parameter by @raskad in [#2858](https://github.com/boa-dev/boa/pull/2858)
- Fix `PropertyKey` index parse by @HalidOdat in [#2843](https://github.com/boa-dev/boa/pull/2843)
- Fix `Date.prototype[Symbol.primitive]` incorrect attributes by @HalidOdat in [#2862](https://github.com/boa-dev/boa/pull/2862)
- Allow `Date` object to store invalid `NativeDateTime` by @HalidOdat in [#2861](https://github.com/boa-dev/boa/pull/2861)
- Fix panic when calling toString with radix by @HalidOdat in [#2863](https://github.com/boa-dev/boa/pull/2863)
- Fix incorrect `LoopContinue` instruction in while-do loops by @HalidOdat in [#2866](https://github.com/boa-dev/boa/pull/2866)
- Initialize `var` bindings in runtime environments with `undefined` by @raskad in [#2860](https://github.com/boa-dev/boa/pull/2860)
- Bugfix/new.target is not understood by the parser as an expression #2793 by @projectnoa in [#2878](https://github.com/boa-dev/boa/pull/2878)
- Fix `RegExp` constructor return value when pattern is a regexp by @HalidOdat in [#2880](https://github.com/boa-dev/boa/pull/2880)
- `RegExp` constructor should call `IsRegExp()` by @HalidOdat in [#2881](https://github.com/boa-dev/boa/pull/2881)
- Fix `for-of` expression parsing by @HalidOdat in [#2882](https://github.com/boa-dev/boa/pull/2882)
- Disallow strict directives with escaped sequences by @jedel1043 in [#2892](https://github.com/boa-dev/boa/pull/2892)
- Make `typeof` throw when accessing uninitialized variables by @raskad in [#2902](https://github.com/boa-dev/boa/pull/2902)
- Fix wrong name of `Function.prototype[Symbol.hasInstance]` by @raskad in [#2905](https://github.com/boa-dev/boa/pull/2905)
- Fix remaining object literal tests by @raskad in [#2906](https://github.com/boa-dev/boa/pull/2906)
- Add SyntaxErrors in GlobalDeclarationInstantiation by @raskad in [#2908](https://github.com/boa-dev/boa/pull/2908)
- Fix switch `default` execution by @HalidOdat in [#2907](https://github.com/boa-dev/boa/pull/2907)
- Add loop and switch return values by @raskad in [#2828](https://github.com/boa-dev/boa/pull/2828)
- Allow escaped `let` as expression by @HalidOdat in [#2916](https://github.com/boa-dev/boa/pull/2916)
- Allow `let` name in for-in loop in non-strict mode by @HalidOdat in [#2915](https://github.com/boa-dev/boa/pull/2915)
- Fix lexical environments in for loops by @raskad in [#2917](https://github.com/boa-dev/boa/pull/2917)
- Fix `GetSubstitution` by @HalidOdat in [#2933](https://github.com/boa-dev/boa/pull/2933)
- Allow escaped `async` as binding name by @jedel1043 in [#2936](https://github.com/boa-dev/boa/pull/2936)
- Fix tagged template creation by @raskad in [#2925](https://github.com/boa-dev/boa/pull/2925)
- Implement Private Runtime Environments by @raskad in [#2929](https://github.com/boa-dev/boa/pull/2929)
- Fix remaining ES5 `built-ins/RegExp` tests by @HalidOdat in [#2957](https://github.com/boa-dev/boa/pull/2957)
- Fix remaining static module bugs by @jedel1043 in [#2955](https://github.com/boa-dev/boa/pull/2955)
- Deny Unicode Escapes in boolean and null expressions by @veera-sivarajan in [#2931](https://github.com/boa-dev/boa/pull/2931)
- Fix `Date` for dynamic timezones by @jedel1043 in [#2877](https://github.com/boa-dev/boa/pull/2877)
- Fix ES5 selector by @veera-sivaraja in [#2924](https://github.com/boa-dev/boa/pull/2924)
- Labelled ByteCompiler Fix by @nekevss in [#2534](https://github.com/boa-dev/boa/pull/2534)
- Fix verbose test display by @jedel1043 in [#2731](https://github.com/boa-dev/boa/pull/2731)
- Fix WASM playground by @jedel1043 in [#2992](https://github.com/boa-dev/boa/pull/2992)
- Correctly initialize functions inside modules by @jedel1043 in [#2993](https://github.com/boa-dev/boa/pull/2993)
- Allow `true`, `false` and `null` in object patterns by @jedel1043 in [#2994](https://github.com/boa-dev/boa/pull/2994)
- Fix panic in optional expressions with private identifiers by @raskad in [#2995](https://github.com/boa-dev/boa/pull/2995)
- Fix prompt on windows by @ShaneEverittM in [#2986](https://github.com/boa-dev/boa/pull/2986)
- Fix panic in constructor call by @raskad in [#3001](https://github.com/boa-dev/boa/pull/3001)
- Unify async iterators and iterators compilation by @jedel1043 in [#2976](https://github.com/boa-dev/boa/pull/2976)
- Correctly parse `yield import(..)` expressions by @jedel1043 in [#3006](https://github.com/boa-dev/boa/pull/3006)
- Return the correct value during a labelled break by @raskad in [#2996](https://github.com/boa-dev/boa/pull/2996)
- Fix panics on empty return values by @raskad in [#3018](https://github.com/boa-dev/boa/pull/3018)
- Add early error for `await` in class static blocks by @raskad in [#3019](https://github.com/boa-dev/boa/pull/3019)
- Fix class constructor return value by @raskad in [#3028](https://github.com/boa-dev/boa/pull/3028)
- Fix super property access by @raskad in [#3026](https://github.com/boa-dev/boa/pull/3026)
- Skip reversing arguments in SuperCallDerived by @dirkdev98 in [#3062](https://github.com/boa-dev/boa/pull/3062)
- Mark header of rooted ephemerons when tracing by @jedel1043 in [#3049](https://github.com/boa-dev/boa/pull/3049)
- Copy `ABOUT.md` file to all published crates by @jedel1043 in [#3074](https://github.com/boa-dev/boa/pull/3074)
- Correctly handle finally..loop..break by @dirkdev98 in [#3073](https://github.com/boa-dev/boa/pull/3073)

### Internal Improvements

- Direct conversion from `u8` to `Opcode` by @HalidOdat [#2951](https://github.com/boa-dev/boa/pull/2951)
- Fix links in readme by @raskad in [#2304](https://github.com/boa-dev/boa/pull/2304)
- Switch to workspace inherited properties by @jedel1043 in [#2297](https://github.com/boa-dev/boa/pull/2297)
- Separate JsObjectType implementors to their own module by @CalliEve in [#2324](https://github.com/boa-dev/boa/pull/2324)
- First prototype for new `JsString` using UTF-16 by @jedel1043 in [#1659](https://github.com/boa-dev/boa/pull/1659)
- Split `Node` into `Statement`, `Expression` and `Declaration` by @jedel1043 in [#2319](https://github.com/boa-dev/boa/pull/2319)
- Changes neccesary -> necessary by @nekevss in [#2370](https://github.com/boa-dev/boa/pull/2370)
- Cleanup and speed-up CI by @RageKnify in [#2376](https://github.com/boa-dev/boa/pull/2376)
- Reduce documentation size in blog by @jedel1043 in [#2383](https://github.com/boa-dev/boa/pull/2383)
- Generate `Opcode` impl using macro by @jedel1043 in [#2391](https://github.com/boa-dev/boa/pull/2391)
- Extract the ast to a crate by @jedel1043 in [#2402](https://github.com/boa-dev/boa/pull/2402)
- Replace `contains` and friends with visitors by @jedel1043 in [#2403](https://github.com/boa-dev/boa/pull/2403)
- Rewrite some patterns with let-else and ok_or_else by @jedel1043 in [#2404](https://github.com/boa-dev/boa/pull/2404)
- Fix async tests result values by @jedel1043 in [#2406](https://github.com/boa-dev/boa/pull/2406)
- Rewrite scope analysis operations using visitors by @jedel1043 in [#2408](https://github.com/boa-dev/boa/pull/2408)
- Make `JsString` conform to miri tests by @jedel1043 in [#2412](https://github.com/boa-dev/boa/pull/2412)
- Reduced boilerplate code in the parser by @Razican in [#2410](https://github.com/boa-dev/boa/pull/2410)
- Extract the parser into a crate by @jedel1043 in [#2409](https://github.com/boa-dev/boa/pull/2409)
- Switch tarpaulin to llvm engine by @RageKnify in [#2432](https://github.com/boa-dev/boa/pull/2432)
- Cleanup `boa_tester` by @jedel1043 in [#2440](https://github.com/boa-dev/boa/pull/2440)
- Restructure lint lists in `boa_ast` by @raskad in [#2433](https://github.com/boa-dev/boa/pull/2433)
- Restructure lints in multiple crates by @raskad in [#2447](https://github.com/boa-dev/boa/pull/2447)
- Restructure lint lists in `boa_engine` by @raskad in [#2455](https://github.com/boa-dev/boa/pull/2455)
- Fix rust 1.66.0 lints by @raskad in [#2486](https://github.com/boa-dev/boa/pull/2486)
- Divide byte compiler by @e-codes-stuff in [#2425](https://github.com/boa-dev/boa/pull/2425)
- Cleanup inline annotations by @jedel1043 in [#2493](https://github.com/boa-dev/boa/pull/2493)
- [profiler] Cache StringId by @tunz in [#2495](https://github.com/boa-dev/boa/pull/2495)
- Improve identifier parsing by @jedel1043 in [#2581](https://github.com/boa-dev/boa/pull/2581)
- Remove Syntax Errors from Bytecompiler by @raskad in [#2598](https://github.com/boa-dev/boa/pull/2598)
- fix: RUSTSEC-2020-0071 in boa_engine by @hanabi1224 in [#2627](https://github.com/boa-dev/boa/pull/2627)
- Migrate tests to new test API by @jedel1043 in [#2619](https://github.com/boa-dev/boa/pull/2619)
- [regexp] new tests for unicode flag by @selfisekai in [#2656](https://github.com/boa-dev/boa/pull/2656)
- Handle surrogates in `String.fromCodePoint` by @jedel1043 in [#2659](https://github.com/boa-dev/boa/pull/2659)
- Bump Test262 and add new features by @jedel1043 in [#2729](https://github.com/boa-dev/boa/pull/2729)
- Fix cross-realm construction bugs by @jedel1043 in [#2786](https://github.com/boa-dev/boa/pull/2786)
- Lift `InternalObjectMethods` from `Object` by @jedel1043 in [#2790](https://github.com/boa-dev/boa/pull/2790)
- Implement async functions using generators by @jedel1043 in [#2821](https://github.com/boa-dev/boa/pull/2821)
- Improve strictness of `GeneratorState` by @jedel1043 in [#2837](https://github.com/boa-dev/boa/pull/2837)
- Upgraded to ICU 1.2 by @Razican in [#2826](https://github.com/boa-dev/boa/pull/2826)
- Fix setting properties inside `with` blocks by @jedel1043 in [#2847](https://github.com/boa-dev/boa/pull/2847)
- Create a unique `PromiseCapability` on each async function call by @jedel1043 in [#2846](https://github.com/boa-dev/boa/pull/2846)
- Refactor binding handling APIs by @jedel1043 in [#2870](https://github.com/boa-dev/boa/pull/2870)
- Refactor guards into a `ContextCleanupGuard` abstraction by @jedel1043 in [#2890](https://github.com/boa-dev/boa/pull/2890)
- Refactor binding declarations by @raskad in [#2887](https://github.com/boa-dev/boa/pull/2887)
- Cleanup some bytecompiler code by @raskad in [#2918](https://github.com/boa-dev/boa/pull/2918)
- Fix `use_self` lints by @raskad in [#2946](https://github.com/boa-dev/boa/pull/2946)
- Remove unused lint allows by @raskad in [#2968](https://github.com/boa-dev/boa/pull/2968)
- Decouple bytecompiler from CodeBlock by @HalidOdat in [#2669](https://github.com/boa-dev/boa/pull/2669)
- Clarity changes for the VM by @nekevss in [#2531](https://github.com/boa-dev/boa/pull/2531)
- Bump bitflags to 2.0.0 by @Razican in [#2666](https://github.com/boa-dev/boa/pull/2666)
- Replace deprecated set-output command by @karol-jani in [#2500](https://github.com/boa-dev/boa/pull/2500)
- Documentation Updates by @nekevss in [#2463](https://github.com/boa-dev/boa/pull/2463)
- Fixed typo in the docs by @Razican in [#2450](https://github.com/boa-dev/boa/pull/2450)
- update tasks.json by @jasonwilliams in [#2313](https://github.com/boa-dev/boa/pull/2313)
- Updated the Code of Conduct by @Razican in [#2365](https://github.com/boa-dev/boa/pull/2365)
- Bump serde_json from 1.0.85 to 1.0.86 by @jedel1043 in [#2341](https://github.com/boa-dev/boa/pull/2341)
- Add test case for issue #2719 by @jedel1043 in [#2980](https://github.com/boa-dev/boa/pull/2980)
- Remove unneded `num_bindings` in `Opcode`s and `CodeBlock` by @HalidOdat in [#2967](https://github.com/boa-dev/boa/pull/2967)
- Added period to sentence by @nekevss in [#2939](https://github.com/boa-dev/boa/pull/2939)
- Prune collected shared shapes by @HalidOdat in [#2941](https://github.com/boa-dev/boa/pull/2941)
- Separate declarative environment kinds by @jedel1043 in [#2921](https://github.com/boa-dev/boa/pull/2921)
- Shrink environment binding locators by @HalidOdat in [#2950](https://github.com/boa-dev/boa/pull/2950)
- Extract "About Boa" section into a separate file by @jedel1043 in [#2938](https://github.com/boa-dev/boa/pull/2938)
- Remove `arguments_binding` field from `CodeBlock` by @HalidOdat in [#2969](https://github.com/boa-dev/boa/pull/2969)
- Remove redundant `param_count` field from `CallFrame` by @HalidOdat in [#2962](https://github.com/boa-dev/boa/pull/2962)
- Direct length access on arrays by @HalidOdat in [#2796](https://github.com/boa-dev/boa/pull/2796)
- Prevent allocation of field names by @HalidOdat in [#2901](https://github.com/boa-dev/boa/pull/2901)
- Added unit tests for `boa_ast::Punctuator` by @Razican in [#2884](https://github.com/boa-dev/boa/pull/2884)
- Added unit tests for `boa_ast::Keyword` by @Razican in [#2883](https://github.com/boa-dev/boa/pull/2883)
- Make update operations reuse the last found binding locator by @jedel1043 in [#2876](https://github.com/boa-dev/boa/pull/2876)
- Docs update for boa_runtime and console documentation by @nekevss in [#2891](https://github.com/boa-dev/boa/pull/2891)
- Direct array element access on `ByValue` instructions by @HalidOdat in [#2827](https://github.com/boa-dev/boa/pull/2827)
- Optimize `String.prototype.normalize` by @jedel1043 in [#2848](https://github.com/boa-dev/boa/pull/2848)
- Fix more Annex B tests by @jedel1043 in [#2841](https://github.com/boa-dev/boa/pull/2841)
- Enable github queues and remove bors.toml by @jedel1043 in [#2899](https://github.com/boa-dev/boa/pull/2899)
- Shrink size of `IndexedProperties` by @HalidOdat in [#2757](https://github.com/boa-dev/boa/pull/2757)
- Added an example usage to documentation by @Razican in [#2742](https://github.com/boa-dev/boa/pull/2742)
- Don't construct prototype if not needed by @HalidOdat in [#2751](https://github.com/boa-dev/boa/pull/2751)
- Implement `is_identifier_(start/part)` using `icu_properties` by @jedel1043 in [#2865](https://github.com/boa-dev/boa/pull/2865)
- Add boa logo to remaining hosted docs by @nekevss in [#2740](https://github.com/boa-dev/boa/pull/2740)
- Added a bunch more tests by @Razican in [#2885](https://github.com/boa-dev/boa/pull/2885)
- Updated dependencies, removes `remove_dir_all`, which is vulnerable by @Razican in [#2685](https://github.com/boa-dev/boa/pull/2685)
- Updated README by @Razican in [#2825](https://github.com/boa-dev/boa/pull/2825)
- Remove panics on module compilation by @jedel1043 in [#2730](https://github.com/boa-dev/boa/pull/2730)
- Update icu dependencies by @raskad in [#2574](https://github.com/boa-dev/boa/pull/2574)
- Pin tarpaulin version to 0.22 by @jedel1043 in [#2562](https://github.com/boa-dev/boa/pull/2562)
- Improve the design of ephemerons in our GC by @jedel1043 in [#2530](https://github.com/boa-dev/boa/pull/2530)
- Pass locale data provider by ref instead of boxing by @jedel1043 in [#2508](https://github.com/boa-dev/boa/pull/2508)
- Fast path for static property keys by @tunz in [#2604](https://github.com/boa-dev/boa/pull/2604)
- Replace `criterion::black_box` with `std::hint::black_box` by @jedel1043 in [#2494](https://github.com/boa-dev/boa/pull/2494)
- Shrink objects by using `ThinVec`s by @HalidOdat in [#2752](https://github.com/boa-dev/boa/pull/2752)
- Redesign native functions and closures API by @jedel1043 in [#2499](https://github.com/boa-dev/boa/pull/2499)
- Make the `wasmbind` feature of the `chrono` crate optional by @Razican in [#2810](https://github.com/boa-dev/boa/pull/2810)
- Use opcode table rather than match by @tunz in [#2501](https://github.com/boa-dev/boa/pull/2501)
- Align iterator loops to the spec by @jedel1043 in [#2686](https://github.com/boa-dev/boa/pull/2686)
- Add AST node for parenthesized expressions by @raskad in [#2738](https://github.com/boa-dev/boa/pull/2738)
- Optimize Get/SetPropertyByName by @tunz in [#2608](https://github.com/boa-dev/boa/pull/2608)
- Keep Integer type for inc/dec of an integer by @tunz in [#2615](https://github.com/boa-dev/boa/pull/2615)
- Implement `CompletionRecords` for the Vm by @nekevss in [#2618](https://github.com/boa-dev/boa/pull/2618)
- Feature flag on builtins console import by @nekevss in [#2584](https://github.com/boa-dev/boa/pull/2584)
- Fix documentation links by @Razican in [#2741](https://github.com/boa-dev/boa/pull/2741)
- Updated syn to 2.0.3 by @Razican in [#2702](https://github.com/boa-dev/boa/pull/2702)
- Cleanup intrinsics and move to realm by @jedel1043 in [#2555](https://github.com/boa-dev/boa/pull/2555)
- Rename `check_parser` and `Identifier` by @jedel1043 in [#2576](https://github.com/boa-dev/boa/pull/2576)
- Fix rust 1.67 lints by @raskad in [#2567](https://github.com/boa-dev/boa/pull/2567)
- Avoid unneeded bounds checks in bytecode address patching by @HalidOdat in [#2680](https://github.com/boa-dev/boa/pull/2680)
- Rust 1.68 clippy fixes by @nekevss in [#2646](https://github.com/boa-dev/boa/pull/2646)
- Fix rust 1.70 lints by @raskad in [#2990](https://github.com/boa-dev/boa/pull/2990)
- Simplify/Refactor exception handling and last statement value by @HalidOdat in [#3053](https://github.com/boa-dev/boa/pull/3053)

# [0.16.0 (2022-09-25)](https://github.com/boa-dev/boa/compare/v0.15...v0.16)

### Feature Enhancements

- Implement getter and setter of `Object.prototype.__proto__` by @CYBAI in [#2110](https://github.com/boa-dev/boa/pull/2110)
- Execution stack & promises by @Razican in [#2107](https://github.com/boa-dev/boa/pull/2107)
- Add the `[[Done]]` field to iterators by @Razican in [#2125](https://github.com/boa-dev/boa/pull/2125)
- Fix for in/of loop initializer environment by @raskad in [#2135](https://github.com/boa-dev/boa/pull/2135)
- Implement `Promise.all` by @raskad in [#2140](https://github.com/boa-dev/boa/pull/2140)
- Implement `Promise.any` by @raskad in [#2145](https://github.com/boa-dev/boa/pull/2145)
- Implement `Promise.allSettled` by @raskad in [#2146](https://github.com/boa-dev/boa/pull/2146)
- Implement `super` expressions by @raskad in [#2116](https://github.com/boa-dev/boa/pull/2116)
- Implement `async function` and `await` by @raskad in [#2158](https://github.com/boa-dev/boa/pull/2158)
- Implementation of `JsMap` Wrapper by @nekevss in [#2115](https://github.com/boa-dev/boa/pull/2115)
- Safe wrapper for `JsSet` by @anuvratsingh in [#2162](https://github.com/boa-dev/boa/pull/2162)
- Implement `JsArrayBuffer` by @HalidOdat in [#2170](https://github.com/boa-dev/boa/pull/2170)
- Implement arrow function parsing based on `CoverParenthesizedExpressionAndArrowParameterList` by @raskad in [#2171](https://github.com/boa-dev/boa/pull/2171)
- Implement Generator Function Constructor by @raskad in [#2174](https://github.com/boa-dev/boa/pull/2174)
- Parse class private async generator methods by @raskad in [#2220](https://github.com/boa-dev/boa/pull/2220)
- Implement Async Generators by @raskad in [#2200](https://github.com/boa-dev/boa/pull/2200)
- Add field accessors to destructing assignment by @raskad in [#2213](https://github.com/boa-dev/boa/pull/2213)
- Added a bit more integer operation consistency to ByteDataBlock creation by @Razican in [#2272](https://github.com/boa-dev/boa/pull/2272)
- Implement Async-from-Sync Iterator Objects by @raskad in [#2234](https://github.com/boa-dev/boa/pull/2234)
- Add URI encoding and decoding functions by @Razican in [#2267](https://github.com/boa-dev/boa/pull/2267)
- Implement `for await...of` loops by @raskad in [#2286](https://github.com/boa-dev/boa/pull/2286)

### Bug Fixes

- Fix `eval` attributes by @raskad in [#2130](https://github.com/boa-dev/boa/pull/2130)
- Fix `this` in function calls by @raskad in [#2153](https://github.com/boa-dev/boa/pull/2153)
- Fix remaining `Promise` bugs by @raskad in [#2156](https://github.com/boa-dev/boa/pull/2156)
- Fix length/index in `32bit` architectures by @HalidOdat in [#2196](https://github.com/boa-dev/boa/pull/2196)
- Fix `yield` expression to end on line terminator by @raskad in [#2232](https://github.com/boa-dev/boa/pull/2232)
- Fix spread arguments in function calls by @raskad in [#2216](https://github.com/boa-dev/boa/pull/2216)
- Fix `arguments` object iterator function by @raskad in [#2231](https://github.com/boa-dev/boa/pull/2231)
- check history file exist if not create it by @udhaykumarbala in [#2245](https://github.com/boa-dev/boa/pull/2245)
- Do not auto-insert semicolon in `VariableDeclarationList` by @tunz in [#2266](https://github.com/boa-dev/boa/pull/2266)
- Fix property access of call expression by @tunz in [#2273](https://github.com/boa-dev/boa/pull/2273)
- fix computed property methods can call super methods by @creampnx-x in [#2274](https://github.com/boa-dev/boa/pull/2274)
- Fix regex literal `/[/]/` by @tunz in [#2277](https://github.com/boa-dev/boa/pull/2277)
- Fixed assignment expression parsing by @Razican in [#2268](https://github.com/boa-dev/boa/pull/2268)
- Fix labelled block statement by @creampnx-x in [#2285](https://github.com/boa-dev/boa/pull/2285)
- Implement missing global object internal methods by @raskad in [#2287](https://github.com/boa-dev/boa/pull/2287)

### Internal Improvements

- Fix spec links for some object operation methods by @CYBAI in [#2111](https://github.com/boa-dev/boa/pull/2111)
- Only run benchmarks on PRs when a label is set by @raskad in [#2114](https://github.com/boa-dev/boa/pull/2114)
- Refactor `construct` and `PromiseCapability` to preserve `JsObject` invariants by @jedel1043 in [#2136](https://github.com/boa-dev/boa/pull/2136)
- Remove `string-interner` dependency and implement custom string `Interner` by @jedel1043 in [#2147](https://github.com/boa-dev/boa/pull/2147)
- Fix clippy 1.62.0 lints by @raskad in [#2154](https://github.com/boa-dev/boa/pull/2154)
- Store call frames in `Vec` instead of singly-linked list by @HalidOdat in [#2164](https://github.com/boa-dev/boa/pull/2164)
- Dense/Packed JavaScript arrays by @HalidOdat in [#2167](https://github.com/boa-dev/boa/pull/2167)
- Fix Rust 1.63 clippy lints by @raskad in [#2230](https://github.com/boa-dev/boa/pull/2230)
- Removed some `unsafe_empty_trace!()` calls to improve performance by @Razican in [#2233](https://github.com/boa-dev/boa/pull/2233)
- Add integer type to fast path of `to_property_key` by @tunz in [#2261](https://github.com/boa-dev/boa/pull/2261)

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.14...v0.15

# [0.15.0 (2022-06-10)](https://github.com/boa-dev/boa/compare/v0.14...v0.15)

### Feature Enhancements

- Deploy playground to custom destination dir by @jedel1043 in [#1943](https://github.com/boa-dev/boa/pull/1943)
- add README for crates.io publish by @superhawk610 in [#1952](https://github.com/boa-dev/boa/pull/1952)
- migrated to clap 3 by @manthanabc in [#1957](https://github.com/boa-dev/boa/pull/1957)
- Implement unscopables for Array.prototype by @NorbertGarfield in [#1963](https://github.com/boa-dev/boa/pull/1963)
- Retrieve feature-based results for Test262 runs by @NorbertGarfield in [#1980](https://github.com/boa-dev/boa/pull/1980)
- Added better error handling for the Boa tester by @Razican in [#1984](https://github.com/boa-dev/boa/pull/1984)
- Add From<f32> for JsValue by @lastmjs in [#1990](https://github.com/boa-dev/boa/pull/1990)
- Implement Classes by @raskad in [#1976](https://github.com/boa-dev/boa/pull/1976)
- Allow `PropertyName`s in `BindingProperty`in `ObjectBindingPattern` by @raskad in [#2022](https://github.com/boa-dev/boa/pull/2022)
- Allow `Initializer` after `ArrayBindingPattern` in `FormalParameter` by @raskad in [#2002](https://github.com/boa-dev/boa/pull/2002)
- Allow unicode escaped characters in identifiers that are keywords by @raskad in [#2021](https://github.com/boa-dev/boa/pull/2021)
- Feature `JsTypedArray`s by @HalidOdat in [#2003](https://github.com/boa-dev/boa/pull/2003)
- Allow creating object with true/false property names by @lupd in [#2028](https://github.com/boa-dev/boa/pull/2028)
- Implement `get RegExp.prototype.hasIndices` by @HalidOdat in [#2031](https://github.com/boa-dev/boa/pull/2031)
- Partial implementation for Intl.DateTimeFormat by @NorbertGarfield in [#2025](https://github.com/boa-dev/boa/pull/2025)
- Allow `let` as variable declaration name by @raskad in [#2044](https://github.com/boa-dev/boa/pull/2044)
- cargo workspaces fixes #2001 by @jasonwilliams in [#2026](https://github.com/boa-dev/boa/pull/2026)
- Move redeclaration errors to parser by @raskad in [#2027](https://github.com/boa-dev/boa/pull/2027)
- Feature `JsFunction` by @HalidOdat in [#2015](https://github.com/boa-dev/boa/pull/2015)
- Improve `JsString` performance by @YXL76 in [#2042](https://github.com/boa-dev/boa/pull/2042)
- Implement ResolveLocale helper by @NorbertGarfield in [#2036](https://github.com/boa-dev/boa/pull/2036)
- Refactor `IdentifierReference` parsing by @raskad in [#2055](https://github.com/boa-dev/boa/pull/2055)
- Implement the global `eval()` function by @raskad in [#2041](https://github.com/boa-dev/boa/pull/2041)
- DateTimeFormat helpers by @NorbertGarfield in [#2064](https://github.com/boa-dev/boa/pull/2064)
- Create `Date` standard constructor by @jedel1043 in [#2079](https://github.com/boa-dev/boa/pull/2079)
- Implement `ProxyBuilder` by @jedel1043 in [#2076](https://github.com/boa-dev/boa/pull/2076)
- Remove `strict` flag from `Context` by @raskad in [#2069](https://github.com/boa-dev/boa/pull/2069)
- Integrate ICU4X into `Intl` module by @jedel1043 in [#2083](https://github.com/boa-dev/boa/pull/2083)
- Implement `Function` constructor by @raskad in [#2090](https://github.com/boa-dev/boa/pull/2090)
- Parse private generator methods in classes by @raskad in [#2092](https://github.com/boa-dev/boa/pull/2092)

### Bug Fixes

- Fix link to the playground by @raskad in [#1947](https://github.com/boa-dev/boa/pull/1947)
- convert inner datetime to local in `to_date_string` by @superhawk610 in [#1953](https://github.com/boa-dev/boa/pull/1953)
- Fix panic on AST dump in JSON format by @kilotaras in [#1959](https://github.com/boa-dev/boa/pull/1959)
- Fix panic in do while by @pdogr in [#1968](https://github.com/boa-dev/boa/pull/1968)
- Support numbers with multiple leading zeroes by @lupd in [#1979](https://github.com/boa-dev/boa/pull/1979)
- Fix length properties on array methods by @lupd in [#1983](https://github.com/boa-dev/boa/pull/1983)
- Allow boolean/null as property identifier by dot operator assignment by @lupd in [#1985](https://github.com/boa-dev/boa/pull/1985)
- fix(vm): off-by-one in code block stringification. by @tsutton in [#1999](https://github.com/boa-dev/boa/pull/1999)
- Indicate bigint has constructor by @lupd in [#2008](https://github.com/boa-dev/boa/pull/2008)
- Change `ArrayBuffer` `byteLength` to accessor property by @lupd in [#2010](https://github.com/boa-dev/boa/pull/2010)
- Fix `ArrayBuffer.isView()` by @HalidOdat in [#2019](https://github.com/boa-dev/boa/pull/2019)
- Fix casting negative number to usize in `Array.splice` by @lupd in [#2030](https://github.com/boa-dev/boa/pull/2030)
- Fix `Symbol` and `BigInt` constructors by @HalidOdat in [#2032](https://github.com/boa-dev/boa/pull/2032)
- Make `Array.prototype` an array object by @HalidOdat in [#2033](https://github.com/boa-dev/boa/pull/2033)
- Fix early return in `for in loop` head by @raskad in [#2043](https://github.com/boa-dev/boa/pull/2043)

### Internal Improvements

- docs: update README by structuring the topics by @ftonato in [#1958](https://github.com/boa-dev/boa/pull/1958)
- Migrate to NPM and cleanup Playground by @jedel1043 in [#1951](https://github.com/boa-dev/boa/pull/1951)
- Fix performance bottleneck in VM by @pdogr in [#1973](https://github.com/boa-dev/boa/pull/1973)
- Remove `git2` and `hex` dependencies by @raskad in [#1992](https://github.com/boa-dev/boa/pull/1992)
- Fix rust 1.60 clippy lints by @raskad in [#2014](https://github.com/boa-dev/boa/pull/2014)
- Refactor `RegExp` constructor methods by @raskad in [#2049](https://github.com/boa-dev/boa/pull/2049)
- Fixing build for changes in clippy for Rust 1.61 by @Razican in [#2082](https://github.com/boa-dev/boa/pull/2082)

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.14...v0.15

# [0.14.0 (2022-03-15) - Virtual Machine](https://github.com/boa-dev/boa/compare/v0.13...v0.14)

### Feature Enhancements

- Implement functions for vm by @HalidOdat in [#1433](https://github.com/boa-dev/boa/pull/1433)
- Implement Object.getOwnPropertyNames and Object.getOwnPropertySymbols by @kevinputera in [#1606](https://github.com/boa-dev/boa/pull/1606)
- Implement `Symbol.prototype.valueOf` by @hle0 in [#1618](https://github.com/boa-dev/boa/pull/1618)
- Implement Array.prototype.at() by @nekevss in [#1613](https://github.com/boa-dev/boa/pull/1613)
- Implement Array.from by @nrabulinski [#1831](https://github.com/boa-dev/boa/pull/1831)
- Implement String.fromCharCode by @hle0 in [#1619](https://github.com/boa-dev/boa/pull/1619)
- Implement `Typed Array` built-in by @Razican in [#1552](https://github.com/boa-dev/boa/pull/1552)
- Implement arguments exotic objects by @jedel1043 in [#1522](https://github.com/boa-dev/boa/pull/1522)
- Allow `BindingPattern`s as `CatchParameter` by @lowr in [#1628](https://github.com/boa-dev/boa/pull/1628)
- Implement `Symbol.prototype[ @@toPrimitive ]` by @Nimpruda in [#1634](https://github.com/boa-dev/boa/pull/1634)
- Implement Generator parsing by @raskad in [#1575](https://github.com/boa-dev/boa/pull/1575)
- Implement Object.hasOwn and improve Object.prototype.hasOwnProperty by @kevinputera in [#1639](https://github.com/boa-dev/boa/pull/1639)
- Hashbang lexer support by @nekevss in [#1631](https://github.com/boa-dev/boa/pull/1631)
- Implement `delete` operator in the vm by @raskad in [#1649](https://github.com/boa-dev/boa/pull/1649)
- Implement Object.fromEntries by @kevinputera in [#1660](https://github.com/boa-dev/boa/pull/1660)
- Initial implementation for increment/decrement in VM by @abhishekc-sharma in [#1621](https://github.com/boa-dev/boa/pull/1621)
- Implement `Proxy` object by @raskad in [#1664](https://github.com/boa-dev/boa/pull/1664)
- Implement object literals for vm by @raskad in [#1668](https://github.com/boa-dev/boa/pull/1668)
- Implement Array findLast and findLastIndex by @bsinky in [#1665](https://github.com/boa-dev/boa/pull/1665)
- Implement `DataView` built-in object by @Nimpruda in [#1662](https://github.com/boa-dev/boa/pull/1662)
- Clean-up contribution guidelines, dependencies, Test262, MSRV by @Razican in [#1683](https://github.com/boa-dev/boa/pull/1683)
- Implement Async Generator Parsing by @nekevss in [#1669](https://github.com/boa-dev/boa/pull/1669)
- Implement prototype of `Intl` built-in by @hle0 in [#1622](https://github.com/boa-dev/boa/pull/1622)
- Add limited console.trace implementation by @osman-turan in [#1623](https://github.com/boa-dev/boa/pull/1623)
- Allow `BindingPattern` in function parameters by @am-a-man in [#1666](https://github.com/boa-dev/boa/pull/1666)
- Small test ux improvements by @orndorffgrant in [#1704](https://github.com/boa-dev/boa/pull/1704)
- Implement missing vm operations by @raskad in [#1697](https://github.com/boa-dev/boa/pull/1697)
- Added fallible allocation to data blocks by @Razican in [#1728](https://github.com/boa-dev/boa/pull/1728)
- Document CodeBlock by @TheDoctor314 in [#1691](https://github.com/boa-dev/boa/pull/1691)
- Generic `JsResult<R>` in `context.throw_` methods by @HalidOdat in [#1734](https://github.com/boa-dev/boa/pull/1734)
- Implement `String.raw( template, ...substitutions )` by @HalidOdat in [#1741](https://github.com/boa-dev/boa/pull/1741)
- Updated test262 suite and dependencies by @Razican in [#1755](https://github.com/boa-dev/boa/pull/1755)
- Lexer string interning by @Razican in [#1758](https://github.com/boa-dev/boa/pull/1758)
- Adjust `compile` and `execute` to avoid clones by @Razican in [#1778](https://github.com/boa-dev/boa/pull/1778)
- Interner support in the parser by @Razican in [#1765](https://github.com/boa-dev/boa/pull/1765)
- Convert `Codeblock` variables to `Sym` by @raskad in [#1798](https://github.com/boa-dev/boa/pull/1798)
- Using production builds for WebAssembly by @Razican in [#1825](https://github.com/boa-dev/boa/pull/1825)
- Give the arrow function its proper name by @rumpl in [#1832](https://github.com/boa-dev/boa/pull/1832)
- Unwrap removal by @Razican in [#1842](https://github.com/boa-dev/boa/pull/1842)
- Feature `JsArray` by @HalidOdat in [#1746](https://github.com/boa-dev/boa/pull/1746)
- Rename "Boa" to boa_engine, moved GC and profiler to their crates by @Razican in [#1844](https://github.com/boa-dev/boa/pull/1844)
- Added conversions from and to serde_json's Value type by @Razican in [#1851](https://github.com/boa-dev/boa/pull/1851)
- Toggleable `JsValue` internals displaying by @HalidOdat in [#1865](https://github.com/boa-dev/boa/pull/1865)
- Implement generator execution by @raskad in [#1790](https://github.com/boa-dev/boa/pull/1790)
- Feature arrays with empty elements by @HalidOdat in [#1870](https://github.com/boa-dev/boa/pull/1870)
- Removed reference counted pointers from `JsValue` variants by @Razican in [#1866](https://github.com/boa-dev/boa/pull/1866)
- Implement `Object.prototype.toLocaleString()` by @HalidOdat in [#1875](https://github.com/boa-dev/boa/pull/1875)
- Implement `AggregateError` by @HalidOdat in [#1888](https://github.com/boa-dev/boa/pull/1888)
- Implement destructing assignments for assignment expressions by @raskad in [#1895](https://github.com/boa-dev/boa/pull/1895)
- Added boa examples by @elasmojs in [#1161](https://github.com/boa-dev/boa/pull/1161)

### Bug Fixes

- Fix BigInt and Number comparison by @HalidOdat [#1887](https://github.com/boa-dev/boa/pull/1887)
- Fix broken structure links in the documentation by @abhishekc-sharma in [#1612](https://github.com/boa-dev/boa/pull/1612)
- Use function name from identifiers in assignment expressions by @raskad [#1908](https://github.com/boa-dev/boa/pull/1908)
- Fix integer parsing by @nrabulinski in [#1614](https://github.com/boa-dev/boa/pull/1614)
- Fix `Number.toExponential` and `Number.toFixed` by @nrabulinski in [#1620](https://github.com/boa-dev/boa/pull/1620)
- Badge updates by @atouchet in [#1638](https://github.com/boa-dev/boa/pull/1638)
- refactor: fix construct_error functions by @RageKnify in [#1703](https://github.com/boa-dev/boa/pull/1703)
- Fix internal vm tests by @raskad in [#1718](https://github.com/boa-dev/boa/pull/1718)
- Removed a bunch of warnings and clippy errors by @Razican in [#1754](https://github.com/boa-dev/boa/pull/1754)
- Fix some broken links in the profiler documentation by @Razican in [#1762](https://github.com/boa-dev/boa/pull/1762)
- Add proxy handling in `isArray` method by @raskad in [#1777](https://github.com/boa-dev/boa/pull/1777)
- Copy/paste fix in Proxy error message by @icecream17 in [#1787](https://github.com/boa-dev/boa/pull/1787)
- Fixed #1768 by @Razican in [#1820](https://github.com/boa-dev/boa/pull/1820)
- Fix string.prototype methods and add static string methods by @jevancc in [#1123](https://github.com/boa-dev/boa/pull/1123)
- Handle allocation errors by @y21 in [#1850](https://github.com/boa-dev/boa/pull/1850)
- Fix wasm use outside browsers by @Razican in [#1846](https://github.com/boa-dev/boa/pull/1846)
- Add assertion to check that a break label is identified at compile-time by @VTCAKAVSMoACE in [#1852](https://github.com/boa-dev/boa/pull/1852)
- Correct reference error message by @aaronmunsters in [#1855](https://github.com/boa-dev/boa/pull/1855)
- Fixing main branch workflows by @Razican in [#1858](https://github.com/boa-dev/boa/pull/1858)
- Correct pop_on_return behaviour by @VTCAKAVSMoACE in [#1853](https://github.com/boa-dev/boa/pull/1853)
- Fix equality between objects and `undefined` or `null` by @HalidOdat in [#1872](https://github.com/boa-dev/boa/pull/1872)
- Removing the panic in favour of an error result by @Razican in [#1874](https://github.com/boa-dev/boa/pull/1874)
- Make `Object.getOwnPropertyDescriptors` spec compliant by @HalidOdat in [#1876](https://github.com/boa-dev/boa/pull/1876)
- Make `Error` and `%NativeError%` spec compliant by @HalidOdat in [#1879](https://github.com/boa-dev/boa/pull/1879)
- Fix `Number.prototype.toString` when passing `undefined` as radix by @HalidOdat in [#1877](https://github.com/boa-dev/boa/pull/1877)
- Cleanup vm stack on function return by @raskad in [#1880](https://github.com/boa-dev/boa/pull/1880)
- `%NativeError%.[[prototype]]` should be `Error` constructor by @HalidOdat in [#1883](https://github.com/boa-dev/boa/pull/1883)
- Make `StringToNumber` spec compliant by @HalidOdat in [#1881](https://github.com/boa-dev/boa/pull/1881)
- Fix `PropertyKey` to `JsValue` conversion by @HalidOdat in [#1886](https://github.com/boa-dev/boa/pull/1886)
- Make iterator spec complaint by @HalidOdat in [#1889](https://github.com/boa-dev/boa/pull/1889)
- Implement `Number.parseInt` and `Number.parseFloat` by @HalidOdat in [#1894](https://github.com/boa-dev/boa/pull/1894)
- Fix unreachable panics in compile_access by @VTCAKAVSMoACE in [#1861](https://github.com/boa-dev/boa/pull/1861)
- Continue panic fixes by @VTCAKAVSMoACE in [#1896](https://github.com/boa-dev/boa/pull/1896)
- Deny const declarations without initializer inside for loops by @jedel1043 in [#1903](https://github.com/boa-dev/boa/pull/1903)
- Fix try/catch/finally related bugs and add tests by @jedel1043 in [#1901](https://github.com/boa-dev/boa/pull/1901)
- Compile StatementList after parse passes on negative tests by @raskad in [#1906](https://github.com/boa-dev/boa/pull/1906)
- Prevent breaks without loop or switch from causing panics by @VTCAKAVSMoACE in [#1860](https://github.com/boa-dev/boa/pull/1860)
- Fix postfix increment and decrement return values by @raskad in [#1913](https://github.com/boa-dev/boa/pull/1913)

### Internal Improvements

- Rewrite initialization of builtins to use the `BuiltIn` trait by @jedel1043 in [#1586](https://github.com/boa-dev/boa/pull/1586)
- Unify object creation with `empty` and `from_proto_and_data` methods by @jedel1043 in [#1567](https://github.com/boa-dev/boa/pull/1567)
- VM Tidy Up by @jasonwilliams in [#1610](https://github.com/boa-dev/boa/pull/1610)
- Fix master refs to main by @jasonwilliams in [#1637](https://github.com/boa-dev/boa/pull/1637)
- Refresh vm docs and fix bytecode trace output by @raskad [#1921](https://github.com/boa-dev/boa/pull/1921)
- Change type of object prototypes to `Option<JsObject>` by @jedel1043 in [#1640](https://github.com/boa-dev/boa/pull/1640)
- Refactor `Function` internal methods and implement `BoundFunction` objects by @jedel1043 in [#1583](https://github.com/boa-dev/boa/pull/1583)
- change that verbosity comparison to > 2 by @praveenbakkal in [#1680](https://github.com/boa-dev/boa/pull/1680)
- Respect rust 1.56 by @RageKnify in [#1681](https://github.com/boa-dev/boa/pull/1681)
- Add bors to CI by @RageKnify in [#1684](https://github.com/boa-dev/boa/pull/1684)
- Adding VM conformance output to PR checks by @Razican in [#1685](https://github.com/boa-dev/boa/pull/1685)
- Start removing non-VM path by @jasonwilliams in [#1747](https://github.com/boa-dev/boa/pull/1747)
- Using upstream benchmark action by @Razican in [#1753](https://github.com/boa-dev/boa/pull/1753)
- Fix bors hanging by @RageKnify in [#1767](https://github.com/boa-dev/boa/pull/1767)
- add more timers on object functions by @jasonwilliams in [#1775](https://github.com/boa-dev/boa/pull/1775)
- Update the PR benchmarks action by @Razican in [#1774](https://github.com/boa-dev/boa/pull/1774)
- General code clean-up and new lint addition by @Razican in [#1809](https://github.com/boa-dev/boa/pull/1809)
- Reduced the size of AST nodes by @Razican in [#1821](https://github.com/boa-dev/boa/pull/1821)
- Using the new formatting arguments from Rust 1.58 by @Razican in [#1834](https://github.com/boa-dev/boa/pull/1834)
- Rework RegExp struct to include bitflags field by @aaronmunsters in [#1837](https://github.com/boa-dev/boa/pull/1837)
- Ignore wastefull `RegExp` tests by @raskad in [#1840](https://github.com/boa-dev/boa/pull/1840)
- Refactor the environment for runtime performance by @raskad in [#1829](https://github.com/boa-dev/boa/pull/1829)
- Refactor mapped `Arguments` object by @raskad in [#1849](https://github.com/boa-dev/boa/pull/1849)
- Fixed dependabot for submodule by @Razican in [#1856](https://github.com/boa-dev/boa/pull/1856)
- Refactorings for Rust 1.59 by @RageKnify in [#1867](https://github.com/boa-dev/boa/pull/1867)
- Removing internal deprecated functions by @HalidOdat in [#1854](https://github.com/boa-dev/boa/pull/1854)
- Remove `toInteger` and document the `string` builtin by @jedel1043 in [#1884](https://github.com/boa-dev/boa/pull/1884)
- Extract `Intrinsics` struct from `Context` and cleanup names by @jedel1043 in [#1890](https://github.com/boa-dev/boa/pull/1890)

**Full Changelog**: https://github.com/boa-dev/boa/compare/v0.13...v0.14

# [0.13.0 (2021-09-30) - Many new features and refactors](https://github.com/boa-dev/boa/compare/v0.12.0...v0.13.0)

Feature Enhancements:

- [FEATURE #1526](https://github.com/boa-dev/boa/pull/1526): Implement ComputedPropertyName for accessor properties in ObjectLiteral (@raskad)
- [FEATURE #1365](https://github.com/boa-dev/boa/pull/1365): Implement splice method (@neeldug)
- [FEATURE #1364](https://github.com/boa-dev/boa/pull/1364): Implement spread for objects (@FrancisMurillo)
- [FEATURE #1525](https://github.com/boa-dev/boa/pull/1525): Implement Object.preventExtensions() and Object.isExtensible() (@HalidOdat)
- [FEATURE #1508](https://github.com/boa-dev/boa/pull/1508): Implement Object.values() (@HalidOdat)
- [FEATURE #1332](https://github.com/boa-dev/boa/pull/1332): Implement Array.prototype.sort (@jedel1043)
- [FEATURE #1417](https://github.com/boa-dev/boa/pull/1471): Implement Object.keys and Object.entries (@skyne98)
- [FEATURE #1406](https://github.com/boa-dev/boa/pull/1406): Implement destructuring assignments (@raskad)
- [FEATURE #1469](https://github.com/boa-dev/boa/pull/1469): Implement String.prototype.replaceAll (@raskad)
- [FEATURE #1442](https://github.com/boa-dev/boa/pull/1442): Implement closure functions (@HalidOdat)
- [FEATURE #1390](https://github.com/boa-dev/boa/pull/1390): Implement RegExp named capture groups (@raskad)
- [FEATURE #1424](https://github.com/boa-dev/boa/pull/1424): Implement Symbol.for and Symbol.keyFor (@HalidOdat)
- [FEATURE #1375](https://github.com/boa-dev/boa/pull/1375): Implement `at` method for string (@neeldug)
- [FEATURE #1369](https://github.com/boa-dev/boa/pull/1369): Implement normalize method (@neeldug)
- [FEATURE #1334](https://github.com/boa-dev/boa/pull/1334): Implement Array.prototype.copyWithin (@jedel1043)
- [FEATURE #1326](https://github.com/boa-dev/boa/pull/1326): Implement get RegExp[@@species] (@raskad)
- [FEATURE #1314](https://github.com/boa-dev/boa/pull/1314): Implement RegExp.prototype [ @@search ] ( string ) (@raskad)
- [FEATURE #1451](https://github.com/boa-dev/boa/pull/1451): Feature prelude module (@HalidOdat)
- [FEATURE #1523](https://github.com/boa-dev/boa/pull/1523): Allow moving NativeObject variables into closures as external captures (@jedel1043)

Bug Fixes:

- [BUG #1521](https://github.com/boa-dev/boa/pull/1521): Added "js" feature for getrandom for WebAssembly builds (@Razican)
- [BUG #1528](https://github.com/boa-dev/boa/pull/1528): Always return undefined from functions that do not return (@raskad)
- [BUG #1518](https://github.com/boa-dev/boa/pull/1518): Moving a JsObject inside a closure caused a panic (@jedel1043)
- [BUG #1502](https://github.com/boa-dev/boa/pull/1502): Adjust EnumerableOwnPropertyNames to use all String type property keys (@raskad)
- [BUG #1415](https://github.com/boa-dev/boa/pull/1415): Fix panic on bigint size (@neeldug)
- [BUG #1477](https://github.com/boa-dev/boa/pull/1477): Properly handle NaN in new Date() (@raskad)
- [BUG #1449](https://github.com/boa-dev/boa/pull/1449): Make Array.prototype methods spec compliant (@HalidOdat)
- [BUG #1353](https://github.com/boa-dev/boa/pull/1353): Make Array.prototype.concat spec compliant (@neeldug)
- [BUG #1384](https://github.com/boa-dev/boa/pull/1384): bitwise not operation (spec improvements) (@neeldug)
- [BUG #1374](https://github.com/boa-dev/boa/pull/1374): Match and regexp construct fixes (@neeldug)
- [BUG #1366](https://github.com/boa-dev/boa/pull/1366): Use lock for map iteration (@joshwd36)
- [BUG #1360](https://github.com/boa-dev/boa/pull/1360): Adjust a comment to be next to the correct module (@teymour-aldridge)
- [BUG #1349](https://github.com/boa-dev/boa/pull/1349): Fixes Array.protoype.includes (@neeldug)
- [BUG #1348](https://github.com/boa-dev/boa/pull/1348): Fixes unshift maximum size (@neeldug)
- [BUG #1339](https://github.com/boa-dev/boa/pull/1339): Scripts should not be considered in a block (@macmv)
- [BUG #1312](https://github.com/boa-dev/boa/pull/1312): Fix display for nodes (@macmv)
- [BUG #1347](https://github.com/boa-dev/boa/pull/1347): Fix stringpad abstract operation (@neeldug)
- [BUG #1584](https://github.com/boa-dev/boa/pull/1584): Refactor the Math builtin object (spec compliant) (@jedel1043)
- [BUG #1535](https://github.com/boa-dev/boa/pull/1535): Refactor JSON.parse (@raskad)
- [BUG #1572](https://github.com/boa-dev/boa/pull/1572): Refactor builtin Map intrinsics to follow more closely the spec (@jedel1043)
- [BUG #1445](https://github.com/boa-dev/boa/pull/1445): improve map conformance without losing perf (@neeldug)
- [BUG #1488](https://github.com/boa-dev/boa/pull/1488): Date refactor (@raskad)
- [BUG #1463](https://github.com/boa-dev/boa/pull/1463): Return function execution result from constructor if the function returned (@raskad)
- [BUG #1434](https://github.com/boa-dev/boa/pull/1434): Refactor regexp costructor (@raskad)
- [BUG #1350](https://github.com/boa-dev/boa/pull/1350): Refactor / Implement RegExp functions (@RageKnify) (@raskad)
- [BUG #1331](https://github.com/boa-dev/boa/pull/1331): Implement missing species getters (@raskad)

Internal Improvements:

- [INTERNAL #1569](https://github.com/boa-dev/boa/pull/1569): Refactor EnvironmentRecordTrait functions (@raskad)
- [INTERNAL #1464](https://github.com/boa-dev/boa/pull/1464): Optimize integer negation (@HalidOdat)
- [INTERNAL #1550](https://github.com/boa-dev/boa/pull/1550): Add strict mode flag to Context (@raskad)
- [INTERNAL #1561](https://github.com/boa-dev/boa/pull/1561): Implement abstract operation GetPrototypeFromConstructor (@jedel1043)
- [INTERNAL #1309](https://github.com/boa-dev/boa/pull/1309): Implement Display for function objects(@kvnvelasco)
- [INTERNAL #1492](https://github.com/boa-dev/boa/pull/1492): Implement new get_or_undefined method for `JsValue` (@jedel1043)
- [INTERNAL #1553](https://github.com/boa-dev/boa/pull/1553): Fix benchmark action in CI (@jedel1043)
- [INTERNAL #1547](https://github.com/boa-dev/boa/pull/1547): Replace FxHashMap with IndexMap in object properties (@raskad)
- [INTERNAL #1435](https://github.com/boa-dev/boa/pull/1435): Constant JsStrings (@HalidOdat)
- [INTERNAL #1499](https://github.com/boa-dev/boa/pull/1499): Updated the Test262 submodule (@Razican)
- [INTERNAL #1458](https://github.com/boa-dev/boa/pull/1458): Refactor the JS testing system (@bartlomieju)
- [INTERNAL #1485](https://github.com/boa-dev/boa/pull/1485): Implement abstract operation CreateArrayFromList (@jedel1043)
- [INTERNAL #1465](https://github.com/boa-dev/boa/pull/1465): Feature throw Error object (@HalidOdat)
- [INTERNAL #1493](https://github.com/boa-dev/boa/pull/1493): Rename boa::Result to JsResult (@bartlomieju)
- [INTERNAL #1457](https://github.com/boa-dev/boa/pull/1457): Rename Value to JsValue (@HalidOdat)
- [INTERNAL #1460](https://github.com/boa-dev/boa/pull/1460): Change StringGetOwnProperty to produce the same strings that the lexer produces (@raskad)
- [INTERNAL #1425](https://github.com/boa-dev/boa/pull/1425): Extract PropertyMap struct from Object (@jedel1043)
- [INTERNAL #1432](https://github.com/boa-dev/boa/pull/1432): Proposal of new PropertyDescriptor design (@jedel1043)
- [INTERNAL #1383](https://github.com/boa-dev/boa/pull/1383): clippy lints and cleanup of old todos (@neeldug)
- [INTERNAL #1346](https://github.com/boa-dev/boa/pull/1346): Implement gh-page workflow on release (@FrancisMurillo)
- [INTERNAL #1422](https://github.com/boa-dev/boa/pull/1422): Refactor internal methods and make some builtins spec compliant (@HalidOdat)
- [INTERNAL #1419](https://github.com/boa-dev/boa/pull/1419): Fix DataDescriptor Value to possibly be empty (@raskad)
- [INTERNAL #1357](https://github.com/boa-dev/boa/pull/1357): Add Example to Execute a Function of a Script File (@schrieveslaach)
- [INTERNAL #1408](https://github.com/boa-dev/boa/pull/1408): Refactor JavaScript bigint rust type (@HalidOdat)
- [INTERNAL #1380](https://github.com/boa-dev/boa/pull/1380): Custom JavaScript string rust type (@HalidOdat)
- [INTERNAL #1382](https://github.com/boa-dev/boa/pull/1382): Refactor JavaScript symbol rust type (@HalidOdat)
- [INTERNAL #1361](https://github.com/boa-dev/boa/pull/1361): Redesign bytecode virtual machine (@HalidOdat)
- [INTERNAL #1381](https://github.com/boa-dev/boa/pull/1381): Fixed documentation warnings (@Razican)
- [INTERNAL #1352](https://github.com/boa-dev/boa/pull/1352): Respect Rust 1.53 (@RageKnify)
- [INTERNAL #1356](https://github.com/boa-dev/boa/pull/1356): Respect Rust fmt updates (@RageKnify)
- [INTERNAL #1338](https://github.com/boa-dev/boa/pull/1338): Fix cargo check errors (@neeldug)
- [INTERNAL #1329](https://github.com/boa-dev/boa/pull/1329): Allow Value.set_field to throw (@raskad)
- [INTERNAL #1333](https://github.com/boa-dev/boa/pull/1333): adds condition to avoid triggers from dependabot (@neeldug)
- [INTERNAL #1337](https://github.com/boa-dev/boa/pull/1337): Fix github actions (@neeldug)

# [0.12.0 (2021-06-07) - `Set`, accessors, `@@toStringTag` and no more panics](https://github.com/boa-dev/boa/compare/v0.11.0...v0.12.0)

Feature Enhancements:

- [FEATURE #1085](https://github.com/boa-dev/boa/pull/1085): Add primitive promotion for method calls on `GetField` (@RageKnify)
- [FEATURE #1033](https://github.com/boa-dev/boa/pull/1033): Implement `Reflect` built-in object (@tofpie)
- [FEATURE #1151](https://github.com/boa-dev/boa/pull/1151): Fully implement `EmptyStatement` (@SamuelQZQ)
- [FEATURE #1158](https://github.com/boa-dev/boa/pull/1158): Include name in verbose results output of `boa-tester` (@0x7D2B)
- [FEATURE #1225](https://github.com/boa-dev/boa/pull/1225): Implement `Math[ @@toStringTag ]` (@HalidOdat)
- [FEATURE #1224](https://github.com/boa-dev/boa/pull/1224): Implement `JSON[ @@toStringTag ]` (@HalidOdat)
- [FEATURE #1222](https://github.com/boa-dev/boa/pull/1222): Implement `Symbol.prototype.description` accessor (@HalidOdat)
- [FEATURE #1221](https://github.com/boa-dev/boa/pull/1221): Implement `RegExp` flag accessors (@HalidOdat)
- [FEATURE #1240](https://github.com/boa-dev/boa/pull/1240): Stop ignoring a bunch of tests (@Razican)
- [FEATURE #1132](https://github.com/boa-dev/boa/pull/1132): Implement `Array.prototype.flat`/`flatMap` (@davimiku)
- [FEATURE #1235](https://github.com/boa-dev/boa/pull/1235): Implement `Object.assign( target, ...sources )` (@HalidOdat)
- [FEATURE #1243](https://github.com/boa-dev/boa/pull/1243): Cross realm symbols (@HalidOdat)
- [FEATURE #1249](https://github.com/boa-dev/boa/pull/1249): Implement `Map.prototype[ @@toStringTag ]` (@wylie39)
- [FEATURE #1111](https://github.com/boa-dev/boa/pull/1111): Implement `Set` builtin object (@RageKnify)
- [FEATURE #1265](https://github.com/boa-dev/boa/pull/1265): Implement `BigInt.prototype[ @@toStringTag ]` (@n14littl)
- [FEATURE #1102](https://github.com/boa-dev/boa/pull/1102): Support Unicode escape in identifier names (@jevancc)
- [FEATURE #1273](https://github.com/boa-dev/boa/pull/1273): Add default parameter support (@0x7D2B)
- [FEATURE #1292](https://github.com/boa-dev/boa/pull/1292): Implement `symbol.prototype[ @@ToStringTag ]` (@moadmmh)
- [FEATURE #1291](https://github.com/boa-dev/boa/pull/1291): Support `GetOwnProperty` for `string` exotic object (@jarkonik)
- [FEATURE #1296](https://github.com/boa-dev/boa/pull/1296): Added the `$262` object to the Test262 test runner (@Razican)
- [FEATURE #1127](https://github.com/boa-dev/boa/pull/1127): Implement `Array.of` (@camc)

Bug Fixes:

- [BUG #1071](https://github.com/boa-dev/boa/pull/1071): Fix attribute configurable of the length property of arguments (@tofpie)
- [BUG #1073](https://github.com/boa-dev/boa/pull/1073): Fixed spelling (@vishalsodani)
- [BUG #1072](https://github.com/boa-dev/boa/pull/1072): Fix `get`/`set` as short method name in `object` (@tofpie)
- [BUG #1077](https://github.com/boa-dev/boa/pull/1077): Fix panics from multiple borrows of `Map` (@joshwd36)
- [BUG #1079](https://github.com/boa-dev/boa/pull/1079): Fix lexing escapes in string literal (@jevancc)
- [BUG #1075](https://github.com/boa-dev/boa/pull/1075): Fix out-of-range panics of `Date` (@jevancc)
- [BUG #1084](https://github.com/boa-dev/boa/pull/1084): Fix line terminator in string literal (@jevancc)
- [BUG #1110](https://github.com/boa-dev/boa/pull/1110): Fix parsing floats panics and bugs (@jevancc)
- [BUG #1202](https://github.com/boa-dev/boa/pull/1202): Fix a typo in `gc.rs` (@teymour-aldridge)
- [BUG #1201](https://github.com/boa-dev/boa/pull/1201): Return optional value in `to_json` functions (@fermian)
- [BUG #1223](https://github.com/boa-dev/boa/pull/1223): Update cli name in Readme (@sphinxc0re)
- [BUG #1175](https://github.com/boa-dev/boa/pull/1175): Handle early errors for declarations in `StatementList` (@0x7D2B)
- [BUG #1270](https://github.com/boa-dev/boa/pull/1270): Fix `Context::register_global_function()` (@HalidOdat)
- [BUG #1135](https://github.com/boa-dev/boa/pull/1135): Fix of instructions.rs comment, to_precision impl and rfc changes (@NathanRoyer)
- [BUG #1272](https://github.com/boa-dev/boa/pull/1272): Fix `Array.prototype.filter` (@tofpie & @Razican)
- [BUG #1280](https://github.com/boa-dev/boa/pull/1280): Fix slice index panic in `add_rest_param` (@0x7D2B)
- [BUG #1284](https://github.com/boa-dev/boa/pull/1284): Fix `GcObject` `to_json` mutable borrow panic (@0x7D2B)
- [BUG #1283](https://github.com/boa-dev/boa/pull/1283): Fix panic in regex execution (@0x7D2B)
- [BUG #1286](https://github.com/boa-dev/boa/pull/1286): Fix construct usage (@0x7D2B)
- [BUG #1288](https://github.com/boa-dev/boa/pull/1288): Fixed `Math.hypot.length` bug (@moadmmh)
- [BUG #1285](https://github.com/boa-dev/boa/pull/1285): Fix environment record panics (@0x7D2B)
- [BUG #1302](https://github.com/boa-dev/boa/pull/1302): Fix VM branch (@jasonwilliams)

Internal Improvements:

- [INTERNAL #1067](https://github.com/boa-dev/boa/pull/1067): Change `Realm::global_object` field from `Value` to `GcObject` (@RageKnify)
- [INTERNAL #1048](https://github.com/boa-dev/boa/pull/1048): VM Trace output fixes (@jasonwilliams)
- [INTERNAL #1109](https://github.com/boa-dev/boa/pull/1109): Define all property methods of constructors (@RageKnify)
- [INTERNAL #1126](https://github.com/boa-dev/boa/pull/1126): Remove unnecessary wraps for non built-in functions (@RageKnify)
- [INTERNAL #1044](https://github.com/boa-dev/boa/pull/1044): Removed duplicated code in `vm.run` using macros (@stephanemagnenat)
- [INTERNAL #1103](https://github.com/boa-dev/boa/pull/1103): Lazy evaluation for cooked template string (@jevancc)
- [INTERNAL #1156](https://github.com/boa-dev/boa/pull/1156): Rework environment records (@0x7D2B)
- [INTERNAL #1181](https://github.com/boa-dev/boa/pull/1181): Merge `Const`/`Let`/`Var` `DeclList` into `DeclarationList` (@0x7D2B)
- [INTERNAL #1234](https://github.com/boa-dev/boa/pull/1234): Separate `Symbol` builtin (@HalidOdat)
- [INTERNAL #1131](https://github.com/boa-dev/boa/pull/1131): Make environment methods take `&mut Context` (@HalidOdat)
- [INTERNAL #1271](https://github.com/boa-dev/boa/pull/1271): Make `same_value` and `same_value_zero` static methods (@HalidOdat)
- [INTERNAL #1276](https://github.com/boa-dev/boa/pull/1276): Cleanup (@Razican)
- [INTERNAL #1279](https://github.com/boa-dev/boa/pull/1279): Add test comparison to Test262 result compare (@Razican)
- [INTERNAL #1293](https://github.com/boa-dev/boa/pull/1293): Fix test262 comment formatting (@0x7D2B)
- [INTERNAL #1294](https://github.com/boa-dev/boa/pull/1294): Don't consider panic fixes as "new failures" (@Razican)

# [0.11.0 (2021-01-14) - Faster Parsing & Better compliance](https://github.com/boa-dev/boa/compare/v0.10.0...v0.11.0)

Feature Enhancements:

- [FEATURE #836](https://github.com/boa-dev/boa/pull/836):
  Async/Await parse (@Lan2u)
- [FEATURE #704](https://github.com/boa-dev/boa/pull/704):
  Implement for...of loops (@joshwd36)
- [FEATURE #770](https://github.com/boa-dev/boa/pull/770):
  Support for symbols as property keys for `Object.defineProperty` (@georgeroman)
- [FEATURE #717](https://github.com/boa-dev/boa/pull/717):
  Strict Mode Lex/Parse (@Lan2u)
- [FEATURE #800](https://github.com/boa-dev/boa/pull/800):
  Implement `console` crate feature - Put `console` object behind a feature flag (@HalidOdat)
- [FEATURE #804](https://github.com/boa-dev/boa/pull/804):
  Implement `EvalError` (@HalidOdat)
- [FEATURE #805](https://github.com/boa-dev/boa/pull/805):
  Implement `Function.prototype.call` (@RageKnify)
- [FEATURE #806](https://github.com/boa-dev/boa/pull/806):
  Implement `URIError` (@HalidOdat)
- [FEATURE #811](https://github.com/boa-dev/boa/pull/811):
  Implement spread operator using iterator (@croraf)
- [FEATURE #844](https://github.com/boa-dev/boa/pull/844):
  Allow UnaryExpression with prefix increment/decrement (@croraf)
- [FEATURE #798](https://github.com/boa-dev/boa/pull/798):
  Implement Object.getOwnPropertyDescriptor() and Object.getOwnPropertyDescriptors() (@JohnDoneth)
- [FEATURE #847](https://github.com/boa-dev/boa/pull/847):
  Implement Map.prototype.entries() (@croraf)
- [FEATURE #859](https://github.com/boa-dev/boa/pull/859):
  Implement spec compliant Array constructor (@georgeroman)
- [FEATURE #874](https://github.com/boa-dev/boa/pull/874):
  Implement Map.prototype.values and Map.prototype.keys (@croraf)
- [FEATURE #877](https://github.com/boa-dev/boa/pull/877):
  Implement Function.prototype.apply (@georgeroman)
- [FEATURE #908](https://github.com/boa-dev/boa/pull/908):
  Implementation of `instanceof` operator (@morrien)
- [FEATURE #935](https://github.com/boa-dev/boa/pull/935):
  Implement String.prototype.codePointAt (@devinus)
- [FEATURE #961](https://github.com/boa-dev/boa/pull/961):
  Implement the optional `space` parameter in `JSON.stringify` (@tofpie)
- [FEATURE #962](https://github.com/boa-dev/boa/pull/962):
  Implement Number.prototype.toPrecision (@NathanRoyer)
- [FEATURE #983](https://github.com/boa-dev/boa/pull/983):
  Implement Object.prototype.isPrototypeOf (@tofpie)
- [FEATURE #995](https://github.com/boa-dev/boa/pull/995):
  Support Numeric separators (@tofpie)
- [FEATURE #1013](https://github.com/boa-dev/boa/pull/1013):
  Implement nullish coalescing (?? and ??=) (@tofpie)
- [FEATURE #987](https://github.com/boa-dev/boa/pull/987):
  Implement property accessors (@tofpie)
- [FEATURE #1018](https://github.com/boa-dev/boa/pull/1018):
  Implement logical assignment operators (&&= and ||=) (@tofpie)
- [FEATURE #1019](https://github.com/boa-dev/boa/pull/1019):
  Implement early errors for non-assignable nodes in assignment (@tofpie)
- [FEATURE #1020](https://github.com/boa-dev/boa/pull/1020):
  Implement Symbol.toPrimitive (@tofpie)
- [FEATURE #976](https://github.com/boa-dev/boa/pull/976):
  Implement for..in (@tofpie)
- [FEATURE #1026](https://github.com/boa-dev/boa/pull/1026):
  Implement String.prototype.split (@jevancc)
- [FEATURE #1047](https://github.com/boa-dev/boa/pull/1047):
  Added syntax highlighting for numbers, identifiers and template literals (@Razican)
- [FEATURE #1003](https://github.com/boa-dev/boa/pull/1003):
  Improve Unicode support for identifier names (@jevancc)

Bug Fixes:

- [BUG #782](https://github.com/boa-dev/boa/pull/782):
  Throw TypeError if regexp is passed to startsWith, endsWith, includes (@pt2121)
- [BUG #788](https://github.com/boa-dev/boa/pull/788):
  Fixing a duplicated attribute in test262 results (@Razican)
- [BUG #790](https://github.com/boa-dev/boa/pull/790):
  Throw RangeError when BigInt division by zero occurs (@JohnDoneth)
- [BUG #785](https://github.com/boa-dev/boa/pull/785):
  Fix zero argument panic in JSON.parse() (@JohnDoneth)
- [BUG #749](https://github.com/boa-dev/boa/pull/749):
  Fix Error constructors to return rather than throw (@RageKnify)
- [BUG #777](https://github.com/boa-dev/boa/pull/777):
  Fix cyclic JSON.stringify / primitive conversion stack overflows (@vgel)
- [BUG #799](https://github.com/boa-dev/boa/pull/799):
  Fix lexer span panic with carriage return (@vgel)
- [BUG #812](https://github.com/boa-dev/boa/pull/812):
  Fix 2 bugs that caused Test262 to fail (@RageKnify)
- [BUG #826](https://github.com/boa-dev/boa/pull/826):
  Fix tokenizing Unicode escape sequence in string literal (@HalidOdat)
- [BUG #825](https://github.com/boa-dev/boa/pull/825):
  calling "new" on a primitive value throw a type error (@dlemel8)
- [BUG #853](https://github.com/boa-dev/boa/pull/853)
  Handle invalid Unicode code point in the string literals (@jevancc)
- [BUG #870](https://github.com/boa-dev/boa/pull/870)
  Fix JSON stringification for fractional numbers (@georgeroman)
- [BUG #807](https://github.com/boa-dev/boa/pull/807):
  Make boa::parse emit error on invalid input, not panic (@georgeroman)
- [BUG #880](https://github.com/boa-dev/boa/pull/880):
  Support more number literals in BigInt's from string constructor (@georgeroman)
- [BUG #885](https://github.com/boa-dev/boa/pull/885):
  Fix `BigInt.prototype.toString()` radix checks (@georgeroman)
- [BUG #882](https://github.com/boa-dev/boa/pull/882):
  Fix (panic) remainder by zero (@georgeroman)
- [BUG #884](https://github.com/boa-dev/boa/pull/884):
  Fix some panics related to BigInt operations (@georgeroman)
- [BUG #888](https://github.com/boa-dev/boa/pull/888):
  Fix some panics in String.prototype properties (@georgeroman)
- [BUG #902](https://github.com/boa-dev/boa/pull/902):
  Fix Accessors panics (@HalidOdat)
- [BUG #959](https://github.com/boa-dev/boa/pull/959):
  Fix Unicode character escape sequence parsing (@tofpie)
- [BUG #964](https://github.com/boa-dev/boa/pull/964):
  Fix single line comment lexing with CRLF line ending (@tofpie)
- [BUG #919](https://github.com/boa-dev/boa/pull/919):
  Reduce the number of `Array`-related panics (@jakubfijalkowski)
- [BUG #968](https://github.com/boa-dev/boa/pull/968):
  Fix unit tests that can be failed due to daylight saving time (@tofpie)
- [BUG #972](https://github.com/boa-dev/boa/pull/972):
  Fix enumerable attribute on array length property (@tofpie)
- [BUG #974](https://github.com/boa-dev/boa/pull/974):
  Fix enumerable attribute on string length property (@tofpie)
- [BUG #981](https://github.com/boa-dev/boa/pull/981):
  Fix prototypes for Number, String and Boolean (@tofpie)
- [BUG #999](https://github.com/boa-dev/boa/pull/999):
  Fix logical expressions evaluation (@tofpie)
- [BUG #1001](https://github.com/boa-dev/boa/pull/1001):
  Fix comparison with infinity (@tofpie)
- [BUG #1004](https://github.com/boa-dev/boa/pull/1004):
  Fix panics surrounding `Object.prototype.hasOwnProperty()` (@HalidOdat)
- [BUG #1005](https://github.com/boa-dev/boa/pull/1005):
  Fix panics surrounding `Object.defineProperty()` (@HalidOdat)
- [BUG #1021](https://github.com/boa-dev/boa/pull/1021):
  Fix spread in new and call expressions (@tofpie)
- [BUG #1023](https://github.com/boa-dev/boa/pull/1023):
  Fix attributes on properties of functions and constructors (@tofpie)
- [BUG #1017](https://github.com/boa-dev/boa/pull/1017):
  Don't panic when function parameters share names (@AnnikaCodes)
- [BUG #1024](https://github.com/boa-dev/boa/pull/1024):
  Fix delete when the property is not configurable (@tofpie)
- [BUG #1027](https://github.com/boa-dev/boa/pull/1027):
  Supress regress errors on invalid escapes for regex (@jasonwilliams
- [BUG #1031](https://github.com/boa-dev/boa/pull/1031):
  Fixed some extra regex panics (@Razican)
- [BUG #1049](https://github.com/boa-dev/boa/pull/1049):
  Support overriding the `arguments` variable (@AnnikaCodes)
- [BUG #1050](https://github.com/boa-dev/boa/pull/1050):
  Remove panic on named capture groups (@Razican)
- [BUG #1046](https://github.com/boa-dev/boa/pull/1046):
  Remove a few different panics (@Razican)
- [BUG #1051](https://github.com/boa-dev/boa/pull/1051):
  Fix parsing of arrow functions with 1 argument (@Lan2u)
- [BUG #1045](https://github.com/boa-dev/boa/pull/1045):
  Add newTarget to construct (@tofpie)
- [BUG #659](https://github.com/boa-dev/boa/pull/659):
  Error handling in environment (@54k1)

Internal Improvements:

- [INTERNAL #735](https://github.com/boa-dev/boa/pull/735):
  Move exec implementations together with AST node structs (@georgeroman)
- [INTERNAL #724](https://github.com/boa-dev/boa/pull/724):
  Ignore tests for code coverage count (@HalidOdat)
- [INTERNAL #768](https://github.com/boa-dev/boa/pull/768)
  Update the benchmark Github action (@Razican)
- [INTERNAL #722](https://github.com/boa-dev/boa/pull/722):
  `ConstructorBuilder`, `ObjectInitializer`, cache standard objects and fix global object attributes (@HalidOdat)
- [INTERNAL #783](https://github.com/boa-dev/boa/pull/783):
  New test262 results format (This also reduces the payload size for the website) (@Razican)
- [INTERNAL #787](https://github.com/boa-dev/boa/pull/787):
  Refactor ast/node/expression into ast/node/call and ast/node/new (@croraf)
- [INTERNAL #802](https://github.com/boa-dev/boa/pull/802):
  Make `Function.prototype` a function (@HalidOdat)
- [INTERNAL #746](https://github.com/boa-dev/boa/pull/746):
  Add Object.defineProperties and handle props argument in Object.create (@dvtkrlbs)
- [INTERNAL #774](https://github.com/boa-dev/boa/pull/774):
  Switch from `regex` to `regress` for ECMA spec-compliant regex implementation (@neeldug)
- [INTERNAL #794](https://github.com/boa-dev/boa/pull/794):
  Refactor `PropertyDescriptor` (Improved performance) (@HalidOdat)
- [INTERNAL #824](https://github.com/boa-dev/boa/pull/824):
  [parser Expression] minor expression macro simplification (@croraf)
- [INTERNAL #833](https://github.com/boa-dev/boa/pull/833):
  Using unstable sort for sorting keys on `to_json()` for GC objects (@Razican)
- [INTERNAL #837](https://github.com/boa-dev/boa/pull/837):
  Set default-run to `boa` removing need for `--bin` (@RageKnify)
- [INTERNAL #841](https://github.com/boa-dev/boa/pull/841):
  Minor refactor and rename in eval() method (@croraf)
- [INTERNAL #840](https://github.com/boa-dev/boa/pull/840):
  fix(profiler): update profiler to match current measureme api (@neeldug)
- [INTERNAL #838](https://github.com/boa-dev/boa/pull/838):
  style(boa): minor cleanup (@neeldug)
- [INTERNAL #869](https://github.com/boa-dev/boa/pull/869):
  Updated cache in workflows (@Razican)
- [INTERNAL #873](https://github.com/boa-dev/boa/pull/873)
  Removed cache from MacOS builds (@Razican)
- [INTERNAL #835](https://github.com/boa-dev/boa/pull/835):
  Move `Object` internal object methods to `GcObject` (@HalidOdat)
- [INTERNAL #886](https://github.com/boa-dev/boa/pull/886):
  Support running a specific test/suite in boa_tester (@georgeroman)
- [INTERNAL #901](https://github.com/boa-dev/boa/pull/901):
  Added "unimplemented" syntax errors (@Razican)
- [INTERNAL #911](https://github.com/boa-dev/boa/pull/911):
  Change Symbol hash to `u64` (@HalidOdat)
- [INTERNAL #912](https://github.com/boa-dev/boa/pull/912):
  Feature `Context::register_global_property()` (@HalidOdat)
- [INTERNAL #913](https://github.com/boa-dev/boa/pull/913):
  Added check to ignore semicolon in parser (@AngelOnFira)
- [INTERNAL #915](https://github.com/boa-dev/boa/pull/915):
  Improve lexer by make cursor iterate over bytes (@jevancc)
- [INTERNAL #952](https://github.com/boa-dev/boa/pull/952):
  Upgraded rustyline and test262 (@Razican)
- [INTERNAL #960](https://github.com/boa-dev/boa/pull/960):
  Fix unresolved links in documentation (@tofpie)
- [INTERNAL #979](https://github.com/boa-dev/boa/pull/979):
  Read file input in bytes instead of string (@tofpie)
- [INTERNAL #1014](https://github.com/boa-dev/boa/pull/1014):
  StatementList: Rename `statements` to `items` (@AnnikaCodes)
- [INTERNAL #860](https://github.com/boa-dev/boa/pull/860):
  Investigation into ByteCode Interpreter (@jasonwilliams)
- [INTERNAL #1042](https://github.com/boa-dev/boa/pull/1042):
  Add receiver parameter to object internal methods (@tofpie)
- [INTERNAL #1030](https://github.com/boa-dev/boa/pull/1030):
  VM: Implement variable declaration (var, const, and let) (@AnnikaCodes)
- [INTERNAL #1010](https://github.com/boa-dev/boa/pull/1010):
  Modify environment binding behaviour of function (@54k1)

# [0.10.0 (2020-09-29) - New Lexer & Test 262 Harness](https://github.com/boa-dev/boa/compare/v0.9.0...v0.10.0)

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

The majority of the old parser was 1 big function (called [`parse`](https://github.com/boa-dev/boa/blob/019033eff066e8c6ba9456139690eb214a0bf61d/boa/src/syntax/parser.rs#L353)) which had some pattern matching on each token coming in.
The easy branches could generate expressions (which were basically AST Nodes), the more involved branches would recursively call into the same function, until eventually you had an expression generated.

This only worked so far, eventually debugging parsing problems were difficult, also more bugs were being raised against the parser which couldn't be fixed.

We decided to break the parser into more of a state-machine. The initial decision for this was inspired by [Fedor Indutny](https://github.com/indutny) who did a talk at (the last) JSConf EU about how he broke up the old node-parser to make it more maintanable. He goes into more detail here https://www.youtube.com/watch?v=x3k_5Mi66sY&feature=youtu.be&t=530

The new parser has functions to match the states of parsing in the spec. For example https://tc39.es/ecma262/#prod-VariableDeclaration has a matching function `read_variable_declaration`. This not only makes it better to maintain but easier for new contributors to get involed, as following the parsing logic of the spec is easier than before.

Once finished some optimisations were added by [HalidOdat](https://github.com/HalidOdat) to use references to the tokens instead of cloning them each time we take them from the lexer.
This works because the tokens live just as long as the parser operations do, so we don't need to copy the tokens.
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

We now have [benchmarks which run against master](https://boajs.dev/boa/dev/bench/)!
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
