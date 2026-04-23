# Parallel Strings

The core innovation of Outconceive is the parallel strings model — four strings of equal length that encode a UI row.

## The Model

```
content:    "Username  ________  Login "
components: "LLLLLLLLLLIIIIIIIIIIBBBBBB"
state_keys: "__________username__submit"
styles:     "                    pppppp"
```

Each character position answers four questions simultaneously:
1. **What text to show** — the `content` string
2. **What component type** — the `components` string
3. **What state to bind** — the `state_keys` string
4. **How to style it** — the `styles` string

## Component Span Grouping

The renderer scans the `components` string and groups consecutive identical characters into spans:

```
Position: 0123456789...
Components: LLLLLLLLLLIIIIIIIIIIBBBBBB
            └────L────┘└────I────┘└──B──┘
```

This produces three `ComponentSpan` structs:
- Label (positions 0-9)
- Input (positions 10-19)
- Button (positions 20-25)

Each span knows its start, end, component type, content slice, state key, and style.

## Why This Works

### No Identity Problem

In React, every component needs a key for reconciliation. In Outconceive, a component's identity is `(line_index, char_offset)` — both are array indices, trivially computable.

### No Structural Diffing

React's VDOM diff walks two trees recursively. Outconceive's diff operates line-by-line. Change one state key → find the lines that reference it → diff only those lines. O(1) for a single state update.

### No AST

JSX compiles to `createElement()` calls that build an abstract syntax tree. Markout parses directly into parallel strings — the same data structure the renderer reads. No intermediate representation.

## Comparison

| Aspect | AST-Based (React) | Parallel Strings (Outconceive) |
|--------|-------------------|--------------------------|
| Data model | Nested tree of objects | Flat strings, equal length |
| Component identity | Generated keys/refs | Array index |
| Diffing scope | Entire subtree | Single line |
| Structural change | Re-key, re-reconcile | Replace-at-root fallback |
| Memory layout | Scattered heap objects | Contiguous string data |
| Parse step | JSX → createElement → VDOM | Markout → parallel strings (direct) |

## The Continuation Marker

The `.` character in the `components` string extends a component across positions without repeating the type:

```
components: "B....."   // Button spanning 6 positions
```

This is used internally when a component needs explicit width control separate from its content length.

## Trade-offs

The parallel strings model trades **compositional depth** for **operational simplicity**. You can't nest a button inside a paragraph inside a card inside a modal in a single line. Instead, you use container sentinels (`@card`/`@end card`) for nesting, keeping each line flat.

This is the right trade-off for form-heavy, data-driven UIs — which is most of the web.
