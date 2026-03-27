# Issue: Fix documentation typos and CommonMark doc link formatting

## Title (for GitHub)

**fix(docs): Fix typos and doc link formatting in documentation**

## Description (for GitHub issue body)

### Summary

This issue tracks small documentation and comment fixes across the Boa codebase: CommonMark-compliant doc link formatting, grammar fixes ("a" → "an" where appropriate), and a typo in the VM debugging docs.

### Changes

1. **Doc link formatting (CommonMark)**  
   Reference-style links in doc comments require a space after the colon per [CommonMark](https://spec.commonmark.org/0.31.2/#link-reference-definitions). Fix:
   - `core/runtime/src/fetch/request.rs` (line 49): `[mdn]:https://` → `[mdn]: https://`
   - `core/parser/src/parser/statement/declaration/mod.rs` (line 8): Ensure `[spec]:` link has a space and that the following `mod export;` is on its own line (currently the link and module declaration are concatenated).

2. **Grammar: article "a" → "an"**  
   - `core/engine/src/vm/call_frame/mod.rs` (lines 259, 263): "a integer" → "an integer"
   - `core/engine/src/vm/runtime_limits.rs` (lines 32, 43): "throw and error" → "throw an error"

3. **Typo in docs**  
   - `docs/vm.md` (line 397): "binarys" → "binaries"

### Why

- Doc links without a space may not render as links in all Markdown parsers.
- Correct grammar improves readability and consistency.
- Single-word typo fix in user-facing debugging documentation.

### Scope

- Documentation and comments only; no behavior changes.
- All edits are in a small number of files and lines.
