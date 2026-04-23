# Why No Trees

Every mainstream web framework models UI as a tree. Outconceive doesn't. Here's why.

## The Tree-Addressing Problem

A tree needs a way to address its nodes. Consider a simple list:

```
div
├── item-0: "Alice"
├── item-1: "Bob"
└── item-2: "Carol"
```

Insert "Dave" at position 0. Every sibling shifts:

```
div
├── item-0: "Dave"   ← new
├── item-1: "Alice"  ← was item-0
├── item-2: "Bob"    ← was item-1
└── item-3: "Carol"  ← was item-2
```

Three items changed identity without changing content. The framework must detect that "Alice" moved from position 0 to position 1 — or wastefully destroy and recreate the DOM node.

React solves this with `key` props. Vue uses `:key`. Both are manual, error-prone, and add complexity.

## How Minimact Failed

Outconceive's predecessor, Minimact, used 8-digit hex paths to address DOM nodes:

```
div         [10000000]
  span      [10000000.10000000]
  span      [10000000.20000000]
```

This is a tightly coupled map of the tree. The second the tree shifts — insert, delete, reorder — the map breaks. Hex paths are essentially reinventing the tree-addressing problem with bigger numbers.

The framework spent more CPU updating IDs than rendering UI.

## The Insight

The rich text editor that preceded Outconceive used a different model: parallel `content` + `styles` strings. No tree. No identity management. Position IS identity.

It worked. 10,000-line documents, 1.9ms keystroke latency, incremental updates. The tree was never needed.

## Positional Identity

In Outconceive, a component's address is `(line_index, character_offset)`. Both are integers. Both are O(1) to compute. Neither changes when siblings are added or removed (because lines are independent).

| Operation | Tree (React) | Flat (Outconceive) |
|-----------|-------------|---------------|
| Address a node | Walk path, check keys | Array index |
| Insert at top | Shift all sibling keys | Append line, replace-at-root |
| Update one item | Diff subtree | Diff one line |
| Delete an item | Re-key siblings | Remove line, replace-at-root |

## The Cost

Outconceive pays one cost: when the line count changes (insert or delete), it falls back to a full replace-at-root. This is the equivalent of React's "just re-render everything" — but it happens at the VDOM level, not the DOM level. The DOM patcher still applies minimal changes.

For typical apps (50-200 lines), this takes under 2ms.

## The Lesson

The tree is not the DOM. The DOM happens to be a tree, but your data model doesn't have to be. Flat is fine. Flat is faster. Flat is simpler.

The environment needs trees. Web performance doesn't.
