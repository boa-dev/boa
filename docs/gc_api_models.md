# GC API Models Investigation

This document captures the current state of Boa's garbage-collector API model
work for issue [#2631](https://github.com/boa-dev/boa/issues/2631).

## Why this exists

Issue #2631 asks for a GC API that:

- Prevents unsafe cross-context sharing patterns.
- Makes rooting and unrooting hard to misuse.
- Stays compatible with compacting and concurrent collectors.
- Enables future snapshotting support.

## Current model in `boa_gc`

Today, `boa_gc` uses a mark-sweep collector with:

- Per-allocation `ref_count` (number of handles).
- Per-collection `non_root_count` discovery (`trace_non_roots` pass).
- Root detection at collection time (`is_rooted = non_root_count < ref_count`).

This model fixed earlier runtime costs tied to rooting churn (see #2773 and
PR #3109), but it still keeps API and implementation tightly coupled.

## Investigated API families

### 1. Context/brand-based APIs (Josephine-style constraints)

Key idea:

- Values are branded by a context/lifetime token.
- Incorrect cross-context usage fails at compile time.

Pros:

- Very strong correctness guarantees.
- Makes illegal sharing structurally impossible.

Cons:

- Harder to integrate with current public API ergonomics.
- Significant migration cost across engine and embedder-facing types.

### 2. Handle-scope APIs (Neon/V8 style)

Key idea:

- User-facing values are handles tied to scopes.
- Root lifetime is explicit through scope nesting.

Pros:

- Clear ownership and lifetime model.
- Works well with compacting and moving collectors.

Cons:

- Requires API redesign around handle scopes.
- Can add ergonomic overhead for internal engine paths.

### 3. Arena/session APIs (`gc-arena` style)

Key idea:

- Allocation and access happen inside explicit arena sessions.
- Tracing safety enforced through session boundaries.

Pros:

- Strong safety model.
- Good fit for incremental and moving collectors.

Cons:

- Large mismatch with Boa's existing pervasive `Gc<T>` usage.
- Requires broad architectural refactor.

### 4. Root-discovery via handles (shredder/rune-inspired direction)

Key idea:

- Keep lightweight GC handles.
- Determine roots by traversing handle graph and heap references.

Pros:

- Incremental transition path from current `Gc<T>` model.
- Reduced API friction compared to strict lifetime branding.
- Compatible with prototyping allocator/collector separation.

Cons:

- Needs careful weak/ephemeron semantics and invariants.
- Requires clear API boundaries to avoid accidental misuse.

## Alignment with current organization direction

Experimental GC API and architecture work moved to
[`boa-dev/oscars`](https://github.com/boa-dev/oscars), which allows faster
iteration without destabilizing `boa` mainline.

This document treats `oscars` as the proving ground for:

- Collector/allocator separation.
- Handle/root API experimentation.
- Compaction/concurrency readiness.

Once validated, stable pieces should be upstreamed into `boa`.

## Proposed execution path

1. Continue API experiments in `oscars`, including explicit invariants and
   benchmark baselines.
2. Define an acceptance checklist for upstreaming into `boa`:
   - no cross-context unsoundness,
   - predictable rooting semantics,
   - weak/ephemeron behavior parity,
   - no regressions in engine benchmarks and test262 flow.
3. Upstream in slices:
   - shared low-risk internals first,
   - then public API transitions with compatibility shims.

## Status

Investigation is active and now has a documented direction:

- Prototype major GC API shifts in `oscars`.
- Upstream validated changes into `boa` in reviewable increments.

This closes the "investigate model families and choose direction" part of
#2631, while implementation remains an ongoing engineering track.
