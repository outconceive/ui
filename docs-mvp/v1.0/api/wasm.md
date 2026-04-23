# WASM API

The `OutconceiveApp` struct is the Rust/WASM core, exposed via `wasm_bindgen`. All methods return `JsValue` (serialized patches or data).

## Constructor

```javascript
import init, { OutconceiveApp } from './pkg/outconceive.js';
await init();
var app = new OutconceiveApp();
```

## Document

| Method | Signature | Returns |
|--------|-----------|---------|
| `from_markout(input)` | `string → void` | Parses Markout, loads document |
| `to_markout()` | `→ string` | Serializes document to Markout |
| `initial_render()` | `→ JsValue (VNode)` | Renders full VDOM |
| `render()` | `→ JsValue (Patch[])` | Full render and diff |

## State

| Method | Signature | Returns |
|--------|-----------|---------|
| `update_state(key, value)` | `string, string → JsValue (Patch[])` | Set text state, return patches |
| `toggle_state(key)` | `string → JsValue (Patch[])` | Toggle boolean, return patches |
| `get_state(key)` | `string → JsValue (string)` | Read text state |
| `get_state_bool(key)` | `string → bool` | Read boolean state |

## Lists

| Method | Signature | Returns |
|--------|-----------|---------|
| `add_list_item(key, json)` | `string, string → JsValue (Patch[])` | Append item from JSON object |
| `remove_list_item(key, idx)` | `string, usize → JsValue (Patch[])` | Remove item by index |
| `set_list_item(key, idx, json)` | `string, usize, string → JsValue (Patch[])` | Update item fields |
| `get_list_count(key)` | `string → usize` | Get number of items |

JSON format: `'{"text": "value", "done": false}'`

## SSR

| Method | Signature | Returns |
|--------|-----------|---------|
| `render_to_html()` | `→ string` | Render current document to HTML |
| `markout_to_html(input)` | `static, string → string` | Render Markout directly to HTML |

## IDE

| Method | Signature | Returns |
|--------|-----------|---------|
| `insert_component(line, type, label, key, style)` | `usize, string×4 → Patch[]` | Insert component at line |
| `insert_container(line, tag, config)` | `usize, string×2 → Patch[]` | Insert container (start + end) |
| `update_line_component(line, type, label, key, style)` | `usize, string×4 → Patch[]` | Replace line content |
| `remove_line_at(idx)` | `usize → Patch[]` | Delete a line |
| `move_line(from, to)` | `usize×2 → Patch[]` | Reorder lines |
| `get_line_info(idx)` | `usize → JsValue (LineInfo)` | Get parallel strings + meta |
| `get_line_count()` | `→ usize` | Total line count |

### LineInfo Object

```javascript
{
    content: "Hello",
    components: "LLLLL",
    state_keys: "_____",
    styles: "     ",
    is_container_start: false,
    is_container_end: false,
    tag: null,
    config: null
}
```

## Patch Types

All mutation methods return an array of patches:

```javascript
[
    { type: "Replace", path: [0], node: { ... } },
    { type: "UpdateText", path: [1, 0], text: "new value" },
    { type: "SetAttribute", path: [2], key: "class", value: "mc-primary" },
    { type: "Insert", path: [3], node: { ... } },
    { type: "Remove", path: [4] },
    { type: "RemoveAttribute", path: [5], key: "checked" }
]
```

Apply with `patcher.applyPatches(patches)`.
