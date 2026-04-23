# Visual IDE

The Outconceive IDE is a browser-based visual editor where you build UIs by clicking toolbar buttons instead of writing code. The editor IS the compiler — clicking "Insert Input" writes the component character directly into the parallel strings.

## Opening the IDE

Navigate to `/ide.html` on the dev server. The IDE has four sections:

- **Toolbar** — component and container buttons
- **Canvas** — live preview of your app (design mode)
- **Side Panel** — properties editor and Markout source viewer
- **Status Bar** — line count and current mode

## Design Mode vs Preview Mode

- **Design Mode** — click rows to select them (blue outline), use toolbar to insert/delete components, edit properties in the side panel
- **Preview Mode** — interact with the running app (type in inputs, click buttons). Toggle with the eye icon in the toolbar.

## Inserting Components

1. Click a row to set the insertion point (or click nothing to append at the end)
2. Click a component button: Input, Button, Checkbox, Select, etc.
3. Fill in the properties (label, state key, style) in the side panel
4. Click "Insert"

The component appears on the canvas immediately.

## Editing Properties

1. Click a row on the canvas
2. The side panel shows the row's content, state key, and Markout source
3. Edit values and they update live

## Source Editor

Click the "Source" tab in the side panel to see the full Markout source. The source is editable — modify it and click "Apply Changes" to re-render the canvas.

## Toolbar Buttons

**Components:** Label, Input, Password, Button, Checkbox, Select, Textarea, Divider

**Containers:** Card, Form, Section

**Actions:** Delete, Move Up, Move Down

**Mode:** Preview toggle, Source view

## The Editor as Compiler

In traditional frameworks, the build step is:

```
Source Code → Parser → AST → Optimizer → Bundler → Runtime
```

In Outconceive:

```
Visual Editor → Parallel Strings (already in WASM memory) → Runtime
```

When you click "Insert Input", no code is generated. The editor writes the component character into the parallel string at the cursor's offset. The WASM engine updates instantly.
