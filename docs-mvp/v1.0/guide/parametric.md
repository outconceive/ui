# Parametric Layout

Outconceive's grid system works great for forms and standard layouts. But when you need precise spatial relationships — "center this under that," "stretch a divider between these two elements," "align these SVGs vertically" — you need `@parametric`.

## Basic Example

```
@parametric
| {label:title "Dashboard"}
| {input:search center-x:title gap-y:16}
| {button:go "Search" primary gap-x:8:search center-y:search}
@end parametric
```

Inside `@parametric`, every component positions itself relative to other named components. No grid cells, no flex ordering — just relationships.

## How It Works

Each component's binding name is its identity. `{input:search}` can be referenced as `search` by any other element.

The Rust solver:
1. Collects all elements and their constraints
2. Builds a dependency graph
3. Topologically sorts (elements are solved after their references)
4. Computes absolute positions
5. Emits absolutely-positioned DOM elements

The first element with no constraints anchors at (0, 0). Everything else is relative.

## Constraints

### Edge Alignment

Align your edges with another element's edges:

```
| {label:title "Settings"}
| {input:name left:title gap-y:8:title}
| {input:email left:title gap-y:8:name}
```

All three elements share the same left edge.

### Stretching

When both `left:` and `right:` are set, the element stretches between them:

```
| {label:start "From"}
| {label:end "To" gap-x:200:start center-y:start}
| {divider:line left:start right:end gap-y:16:start}
```

The divider stretches from `start`'s left edge to `end`'s right edge.

### Center Alignment

```
| {label:heading "Create Account"}
| {input:email center-x:heading gap-y:16}
| {button:submit "Sign Up" primary center-x:heading gap-y:12:email}
```

Both the input and button are horizontally centered under the heading.

### Vertical Center

```
| {image:chart1 href=chart.svg}
| {image:chart2 href=stats.svg center-y:chart1 gap-x:1rem:chart1}
```

Two images side by side, vertically centered on each other's midpoint. If they're different heights, the shorter one shifts to align.

### Spacing Units

Gap constraints support multiple units:

| Unit | Example | Meaning |
|------|---------|---------|
| `px` | `gap-y:16` or `gap-y:16px` | 16 pixels (default) |
| `rem` | `gap-x:1rem` | 1 root em (16px) |
| `em` | `gap-y:2em` | 2 ems (32px) |
| `%` | `gap-x:10%` | 10% |

### Size Matching

Make one element match another's dimensions:

```
| {input:name}
| {input:email width:name gap-y:8:name left:name}
```

Both inputs will be the same width.

### Distribution

Center an element between two others:

```
| {label:left "Start"}
| {label:right "End" gap-x:300:left}
| {label:mid "Middle" distribute-x:left:right center-y:left}
```

`mid` is placed at the horizontal midpoint between `left` and `right`.

## Nesting

`@parametric` is a container like any other — it can go inside a `@card`, a grid column, or any layout:

```
@card padding:24
| Header
@parametric
| {label:title "Dashboard"}
| {input:search center-x:title gap-y:16}
@end parametric
| Footer
@end card
```

## Parametric vs Grid

| | Grid (`col-N`) | Parametric |
|-|---------------|------------|
| **Model** | 12-column fractions | Named relationships |
| **Best for** | Forms, standard layouts | Data viz, custom alignment |
| **Positioning** | Flow-based | Absolute within container |
| **Element identity** | Anonymous | Named via binding |
| **Sizing** | Column fractions | Intrinsic or matched |
