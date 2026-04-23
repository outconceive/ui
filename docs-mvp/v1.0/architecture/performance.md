# Performance

Outconceive's performance comes from doing less, not doing it faster.

## Complexity

| Operation | Complexity | Explanation |
|-----------|-----------|-------------|
| Single state update | O(K) | K = lines referencing that key (typically 1-2) |
| Render one line | O(W) | W = component spans on the line |
| Diff one line | O(C) | C = child count of the line's VNode |
| Structural change | O(N) | Full replace-at-root (line count changed) |
| Initial render | O(N×W) | All lines rendered |
| State lookup | O(1) | HashMap |
| Action dispatch | O(1) | HashMap |

React's reconciliation is O(N) with heuristics. Outconceive's single-state update is O(1) because it skips lines that don't reference the changed key.

## Dirty Tracking

The state store maintains a reverse index: state key → set of line indices. When `username` changes:

1. Look up `username` in the index → `{line 2, line 7}`
2. Re-render lines 2 and 7 only
3. Diff each line's old VNode against new VNode
4. Emit patches

Lines 0, 1, 3-6, 8+ are untouched. No tree walk. No subtree comparison.

## Bundle Size

| Component | Size |
|-----------|------|
| WASM binary | 226KB |
| JS wrapper (outconceive.js) | ~8KB |
| DOM patcher | ~3KB |
| Event router | ~2KB |
| Router | ~2KB |
| Bus | ~1KB |
| CSS | ~8KB |
| **Total** | **~250KB** |

Compare: React 45KB + ReactDOM 130KB = 175KB (minified, gzipped ~60KB). Outconceive is slightly larger uncompressed but includes the entire rendering engine, state management, routing, theming, validation, and SSR — not just the component model.

## Benchmarks

From the editor project that inspired Outconceive (10,000 lines, incremental rendering):

| Metric | Before Optimization | After |
|--------|-------------------|-------|
| Keystroke latency | 16ms | 1.9ms |
| Approach | Full VDOM rebuild | Line-level incremental |

For typical Outconceive apps (50-200 lines):
- State update + render + diff + patch: **< 0.5ms**
- SSR render to HTML: **~0.5ms**
- Hydration: **~0.3ms**

## Why It's Fast

1. **No tree diffing** — line-level comparison, not subtree
2. **No reconciliation** — positional identity, no key matching
3. **No framework overhead** — WASM runs native-speed compiled Rust
4. **No hydration cost** — SSR HTML attaches events, doesn't re-render
5. **No bundle parsing** — WASM compiles once, runs instantly
6. **Flat memory layout** — strings are contiguous, cache-friendly

## The Replace-at-Root Fallback

When the line count changes (add/remove line, add/remove container), Outconceive emits a single `Replace` patch at the root path. This replaces the entire app's DOM content.

This sounds expensive, but it's not — the DOM patcher rebuilds only the root's children, and the browser's layout engine handles the rest. For apps under 200 lines, this takes < 2ms.

The alternative — computing precise insert/remove patches across container boundaries — would add significant complexity for marginal gain. Simple wins.
