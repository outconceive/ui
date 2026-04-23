# Outconceive UI

A web UI framework that replaces hierarchical component trees with flat, parallel strings. No trees, no recursive reconciliation, no hex paths — just lines and positional identity.

## Core Concept

Every UI row is a `Line` with four parallel strings of equal length:

```
content:    "Username  ________  Login "
components: "LLLLLLLLLLIIIIIIIIIIBBBBBB"
state_keys: "__________username__submit"
styles:     "                    pppppp"
```

Each character position maps to a component type (`L`=label, `I`=input, `B`=button), a state binding, and a visual style. The renderer groups consecutive same-type positions into spans and generates real DOM elements.

## Architecture

```
Visual IDE (toolbar + modals)
        │
        ▼
  Document Model ── flat Vec<Line> with parallel strings
        │
        ▼
  VDOM Generation (Rust/WASM) ── component spans → VNodes
        │
        ▼
  Diff ── line-level incremental (O(1) for single-line edits)
        │
        ▼
  DOM Patcher (JS) ── minimal patches to real DOM
```

- **Rust/WASM core** — all rendering, diffing, and state management
- **No server dependency** — pure client-side
- **Incremental by default** — only changed lines re-render

## Component Types

| Char | Component    | Renders As            |
|------|--------------|-----------------------|
| `L`  | Label        | `<span>`              |
| `I`  | Text Input   | `<input type="text">` |
| `P`  | Password     | `<input type="password">` |
| `B`  | Button       | `<button>`            |
| `C`  | Checkbox     | `<input type="checkbox">` |
| `R`  | Radio        | `<input type="radio">`|
| `S`  | Select       | `<select>`            |
| `T`  | Textarea     | `<textarea>`          |
| `G`  | Image        | `<img>`               |
| `K`  | Link         | `<a>`                 |
| `D`  | Divider      | `<hr>`                |
| `X`  | Custom       | Registry lookup       |

## Containers

Block-level containers use start/end sentinel lines:

```rust
doc.container_start("card", Some("shadow:md"));
doc.row("Hello  ", "LLLLLLL", "_______", "       ");
doc.container_end("card");
```

Renders as nested DOM without a tree model — containers are just ranges between sentinels.

## State

Global key-value store with positional binding:

```rust
// State update → finds lines referencing "username" → re-renders only those lines
let patches = core.update_state_text("username", "Alice");
```

## Building

```sh
cargo test
```

## Origin

Outconceive evolved from two prior projects:

1. **A rich text editor** using parallel `content` + `styles` strings — proved that flat models with positional identity can replace tree-based architectures
2. **Minimact** — a server-first React alternative that hit a wall with hex-path component IDs that broke on every structural change

The parallel strings model from the editor eliminated the tree-addressing problem entirely. A component's identity is its line number and character offset — trivially computable, zero maintenance.
