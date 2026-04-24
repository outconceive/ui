# Markout Reference

Complete syntax reference for the Markout markup language.

## Content Rows

Lines starting with `|` define UI rows:

```
| Plain text
| Text with {input:binding}
| {button:action "Label" style}
```

## Component Syntax

```
{type:binding "label" style attribute1 attribute2}
```

| Part | Required | Description |
|------|----------|-------------|
| `type` | Yes | Component type (`input`, `button`, `label`, etc.) |
| `binding` | No | State key (after `:`) |
| `"label"` | No | Display text (quoted) |
| `style` | No | Visual style keyword |
| Attributes | No | `col-N`, `validate:`, `animate:`, `route:`, `fetch:`, `popover:` |

## Component Types

| Type | HTML Output | State Binding |
|------|------------|---------------|
| `input` | `<input type="text">` | Read/write text |
| `password` | `<input type="password">` | Read/write text |
| `button` | `<button>` | Action name (click) |
| `checkbox` | `<input type="checkbox">` | Boolean toggle |
| `radio` | `<input type="radio">` | Boolean |
| `select` | `<select>` | Read/write text |
| `textarea` | `<textarea>` | Read/write text |
| `label` | `<span>` | Read-only display |
| `pill` | `<span class="mc-pill">` | Read-only display (rounded tag) |
| `badge` | `<span class="mc-badge">` | Read-only display (counter) |
| `progress` | `<div class="mc-progress">` | Numeric 0-100 (bar width) |
| `sparkline` | `<svg>` | Comma-separated values (chart) |
| `spacer` | `<span>` | Layout mode (`end`, `evenly`, `col-N-end`) |
| `link` | `<a>` | Navigation |
| `image` | `<img>` | Display |
| `divider` | `<hr>` | None |

## Containers

```
@tag config
| content...
@end tag
```

| Directive | Renders As |
|-----------|-----------|
| `@card` | `<div class="mc-card">` |
| `@form` | `<form>` |
| `@section` | `<section>` |
| `@nav` | `<nav>` |
| `@header` | `<header>` |
| `@footer` | `<footer>` |
| `@main` | `<main>` |
| `@aside` | `<aside>` |
| `@columns` | `<div>` with CSS columns |
| `@editor` | Rich text editor (WASM-powered) |

## Editor

```
@editor feature1 feature2 ... bind:stateKey
@end editor
```

Features are opt-in. Only declared features are available in the toolbar, keyboard shortcuts, and content. Undeclared formatting is stripped from pasted text.

| Feature | Toolbar | Shortcut |
|---------|---------|----------|
| `bold` | **B** | Ctrl+B |
| `italic` | *I* | Ctrl+I |
| `underline` | U | Ctrl+U |
| `strikethrough` | S | — |
| `code` | `<>` | Ctrl+` |
| `heading` | H | — |
| `list` | • | — |
| `ordered-list` | 1. | — |
| `quote` | " | — |
| `code-block` | {} | — |
| `link` | 🔗 | Ctrl+K |
| `hr` | — | — |

`bind:key` syncs the editor's markdown content to an Outconceive state key.

## Container Config

Key-value pairs separated by spaces:

| Key | Example | CSS Output |
|-----|---------|-----------|
| `padding` | `padding:24` | `padding: 24px` |
| `max-width` | `max-width:600px` | `max-width: 600px` |
| `width` | `width:50%` | `width: 50%` |
| `height` | `height:400px` | `height: 400px; overflow: hidden` |
| `cols` | `cols:3` | `column-count: 3` |
| `gap` | `gap:16` | `column-gap: 16px; gap: 16px` |

## Styles

| Keyword | CSS Class |
|---------|----------|
| `primary` | `mc-primary` |
| `secondary` | `mc-secondary` |
| `danger` | `mc-danger` |
| `warning` | `mc-warning` |
| `info` | `mc-info` |
| `outline` | `mc-outline` |
| `ghost` | `mc-ghost` |
| `dark` | `mc-dark` |
| `light` | `mc-light` |

## Column Sizing

| Syntax | Width |
|--------|-------|
| `col-6` | 50% (6/12) |
| `col-4` | 33.3% (4/12) |
| `col-3[5]` | 60% (3/5) |
| `sm:col-6` | 50% at 576px+ |
| `md:col-4` | 33.3% at 768px+ |
| `lg:col-3` | 25% at 992px+ |
| `xl:col-2` | 16.7% at 1200px+ |

## Validation Rules

| Rule | Description |
|------|-------------|
| `required` | Non-empty |
| `email` | Valid email format |
| `number` | Numeric value |
| `url` | Valid URL |
| `min:N` | Minimum N characters |
| `max:N` | Maximum N characters |
| `pattern:regex` | Custom regex match |

## Animations

| Name | Effect |
|------|--------|
| `fade` | Opacity 0→1 |
| `slide` | Slide from left |
| `slide-up` | Slide from bottom |
| `scale` | Scale 0.9→1 |
| `bounce` | Elastic scale |
| `pulse` | Continuous opacity pulse |
| `shake` | Horizontal shake |
| `glow` | Continuous shadow pulse |

## Spacer Modes

| Syntax | Effect |
|--------|--------|
| `{spacer:end}` | `flex:1` — push remaining items to the right |
| `{spacer:evenly}` | `flex:1` — equal distribution (place between items) |
| `{spacer:col-N}` | Fixed N/12 width gap |
| `{spacer:col-N-end}` | Fill up to end of column N (tab stop alignment) |

## Popover

Add to any component for a hover tooltip:

```
{button:info "Help" popover:"Detailed help text"}
```

## Parametric Layout

`@parametric` is a constraint-based layout container. Components position themselves relative to each other by name instead of grid columns.

```
@parametric
| {label:title "Dashboard"}
| {input:search center-x:title gap-y:16}
| {button:go "Search" primary right:search gap-x:8 center-y:search}
| {divider:line left:title right:go gap-y:24:go}
@end parametric
```

Each component's binding name (e.g., `search`) becomes its identity for constraint references.

### Edge Alignment

| Constraint | Meaning |
|-----------|---------|
| `left:ref` | My left edge = ref's left edge |
| `right:ref` | My right edge = ref's right edge |
| `top:ref` | My top edge = ref's top edge |
| `bottom:ref` | My bottom edge = ref's bottom edge |

When both `left:` and `right:` are set, the element stretches between them.

### Center Alignment

| Constraint | Meaning |
|-----------|---------|
| `center-x:ref` | My horizontal center = ref's horizontal center |
| `center-y:ref` | My vertical center = ref's vertical center |

### Spacing (Gap)

| Constraint | Meaning |
|-----------|---------|
| `gap-x:N:ref` | N units to the right of ref |
| `gap-y:N:ref` | N units below ref |
| `gap-x:N` | N units right of the nearest referenced element |
| `gap-y:N` | N units below the nearest referenced element |

Supports `px` (default), `rem`, `em`, and `%`:

```
| {image:b center-y:a gap-x:1rem:a}
| {label:c left:a gap-y:2em:b}
```

### Sizing

| Constraint | Meaning |
|-----------|---------|
| `width:ref` | My width = ref's width |
| `height:ref` | My height = ref's height |

### Distribution

| Constraint | Meaning |
|-----------|---------|
| `distribute-x:a:b` | Center horizontally between a and b |
| `distribute-y:a:b` | Center vertically between a and b |

### How It Works

The first element with no constraints anchors at (0, 0). The Rust solver builds a dependency graph, topologically sorts it, and computes absolute positions for each element. The container auto-sizes to the bounding box of all elements. No runtime DOM measurement — everything is computed in WASM.

## Special Directives

| Directive | Description |
|-----------|-------------|
| `@each key` / `@end each` | Repeat template for each list item |
| `@define name` / `@end define` | Define reusable template |
| `@use name scope=prefix` | Instantiate template with scoped state |
| `@parametric` / `@end parametric` | Constraint-based layout container |
