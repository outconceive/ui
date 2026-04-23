# Core Concepts

## The Problem with Trees

Every mainstream web framework — React, Vue, Svelte, Angular — models UI as a tree of nested components. This creates three fundamental problems:

1. **Identity**: How do you address a node in a tree? Keys, refs, generated IDs — all brittle when the tree changes.
2. **Reconciliation**: Comparing two trees is O(N³). React's heuristic reduces this to O(N), but it's still complex and error-prone.
3. **Structural changes**: Insert a node at the top of a list? Every sibling's identity shifts.

Outconceive eliminates all three by not having a tree.

## Parallel Strings

Every UI row in Outconceive is a `Line` with four parallel strings of equal length:

```
content:    "Username  ________  Login "
components: "LLLLLLLLLLIIIIIIIIIIBBBBBB"
state_keys: "__________username__submit"
styles:     "                    pppppp"
```

Each character position maps to:
- **What it is** — the `components` string (L=label, I=input, B=button)
- **What state it binds to** — the `state_keys` string
- **How it looks** — the `styles` string (p=primary, d=danger, etc.)

The renderer groups consecutive same-type positions into component spans and generates DOM elements.

## Positional Identity

A component's identity is its **line number + character offset**. Both are trivially computable — no generated IDs, no keys, no path strings. When you update state, Outconceive finds the lines that reference that key and re-renders only those lines. O(1) for single-state updates.

## The Line Model

```rust
struct Line {
    content:    String,   // Display text
    components: String,   // Component type per position
    state_keys: String,   // State binding per position
    styles:     String,   // Visual style per position
    meta:       MetaLine, // Block-level metadata
}
```

Containers (cards, sections, forms) are just start/end sentinel lines:

```
Line 0: meta.format = CONTAINER_START, tag = "card"
Line 1: content row (label + input + button)
Line 2: meta.format = CONTAINER_END
```

No nesting in the data model. The renderer maintains a stack to produce nested DOM output.

## State Store

State is a flat key-value store:

```javascript
app.set('username', 'Alice');       // Text
app.set('count', '42');             // Number as text
app.set('agree', true);             // Boolean (checkbox)
```

For lists, keys are dot-scoped: `todos.0.text`, `todos.1.done`, etc.

When a value changes, Outconceive:
1. Marks the key as dirty
2. Finds lines that reference the key (via a reverse index)
3. Re-renders only those lines
4. Diffs against the cached VDOM
5. Emits minimal DOM patches

## Rendering Pipeline

```
Markout → Parse → Document (flat lines) → VDOM → Diff → Patches → DOM
```

This is the same pipeline for every operation — initial render, state update, structural change. The only variable is how many lines get re-rendered.

## No Build Step

The entire framework is 140KB of WASM + 10KB of JS. Include it with a script tag. Write Markout inline. No compiler, no bundler, no transpiler.
