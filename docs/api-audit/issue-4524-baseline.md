# API Audit Baseline for #4524

Date: 2026-03-01
Repo: `boa-dev/boa`
Issue: https://github.com/boa-dev/boa/issues/4524

## Goal

Create a reproducible baseline of the current public Rust API surface before proposing 1.0 stabilization changes.

## Tooling used

- `cargo-public-api 0.51.0`
- `cargo-semver-checks 0.46.0`
- `nightly-x86_64-pc-windows-msvc` (required by `cargo-public-api` in this environment)

## Baseline snapshots produced

Generated with:

```powershell
$pkgs = @(
  'boa_ast','boa_engine','boa_gc','boa_icu_provider','boa_interner',
  'boa_macros','boa_parser','boa_runtime','boa_string','small_btree','tag_ptr'
)
foreach ($p in $pkgs) {
  cargo +nightly public-api -p $p -ss --color never `
    2> "docs/api-audit/logs/$p.public-api.stderr.log" `
    | Out-File "docs/api-audit/baseline-public-api/$p.txt" -Encoding utf8
}
```

Output directories (local baseline artifacts):

- `docs/api-audit/baseline-public-api/`
- `docs/api-audit/logs/`

Note: these generated snapshot/log files are intended as local audit artifacts and do not need to be committed.

## Snapshot size (line count per crate)

- `boa_ast`: 5633
- `boa_engine`: 7904
- `boa_gc`: 1165
- `boa_icu_provider`: 2
- `boa_interner`: 154
- `boa_macros`: 13
- `boa_parser`: 517
- `boa_runtime`: 687
- `boa_string`: 414
- `small_btree`: 129
- `tag_ptr`: 23

## Quick code-shape scan (coarse)

Regex scan over source trees (`pub struct`, `pub enum`, `pub trait`, `pub field`, `#[non_exhaustive]`) indicates:

- Highest exposed surface: `boa_engine`, `boa_ast`
- Public-field candidates exist mostly in `boa_engine`, `boa_string`, and small pockets in `boa_ast` / `boa_runtime`
- `#[non_exhaustive]` is used, but sparsely

This is a coarse signal only. Decisions should be taken from the `cargo public-api` output and crate-level maintainership intent.

## Proposed next narrow slice

Start with one crate and one concern to keep review load low:

1. crate: `boa_engine`
2. concern: public fields that should be getters or reduced visibility
3. output: short audit table (`item`, `current`, `proposed`, `breakage risk`, `migration path`)

After maintainer alignment on that first table, proceed with small PRs by concern type.

Initial slice document: `docs/api-audit/boa_engine-public-fields-slice.md`
