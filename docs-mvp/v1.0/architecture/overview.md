# Architecture Overview

## The Pipeline

```
Markout → Parse → Document (flat Vec<Line>) → VDOM → Diff → Patches → DOM
```

Every operation — initial render, state update, structural change — follows this pipeline. The only variable is how many lines get re-rendered.

## System Diagram

```
┌─────────────────────────────────────────────────────┐
│                      Browser                         │
│                                                      │
│  ┌─────────────┐                                     │
│  │  Visual IDE  │ ── toolbar clicks write directly    │
│  │  (optional)  │    into parallel strings            │
│  └──────┬───────┘                                    │
│         ▼                                            │
│  ┌──────────────┐    ┌────────────┐   ┌───────────┐ │
│  │   Document   │───▶│  VDOM Gen  │──▶│   Diff    │ │
│  │  Vec<Line>   │    │  (Rust)    │   │  (Rust)   │ │
│  │  + StateStore│    └────────────┘   └─────┬─────┘ │
│  └──────────────┘                           │       │
│         ▲                              Patches      │
│         │                                   │       │
│  ┌──────┴───────┐                    ┌──────▼─────┐ │
│  │ Event Router │◀───────────────────│ DOM Patcher │ │
│  │    (JS)      │                    │    (JS)     │ │
│  └──────────────┘                    └────────────┘ │
└─────────────────────────────────────────────────────┘
```

## Layers

| Layer | Language | Responsibility |
|-------|----------|---------------|
| **Markout Parser** | Rust | Parse markup → Document model |
| **Document Model** | Rust | Flat `Vec<Line>` with parallel strings + state store |
| **VDOM Renderer** | Rust | Document → VNode tree (only at container boundaries) |
| **Differ** | Rust | Old VNode vs new VNode → Patch list |
| **DOM Patcher** | JS | Apply patches to real DOM |
| **Event Router** | JS | Browser events → WASM state mutations |
| **Outconceive Wrapper** | JS | Developer API (computed, effects, fetch, persist, etc.) |

## The Editor as Compiler

The Visual IDE is not a separate tool — it's the same rendering pipeline running in design mode. Clicking "Insert Input" on the toolbar writes a component character into the parallel string at the cursor's offset. The WASM engine diffs and patches immediately.

```
Traditional:  Source Code → Parser → AST → Optimizer → Bundler → Runtime
Outconceive:       Visual Editor → Parallel Strings (in WASM memory) → Runtime
```

No parse step. No AST. No build.

## Island Architecture

Each `Outconceive` instance is fully independent — own WASM core, own state, own DOM patcher. Mount multiple instances to different DOM elements. Communicate via `OutconceiveBus` event bus.

This is the same concept as Astro's islands, but with one runtime (140KB WASM) instead of React + Vue + Svelte adapters.
