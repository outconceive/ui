# Outconceive UI — Build Summary

## Origin Story

Outconceive started as a rich text editor experiment. The editor used a novel "parallel strings" architecture — each line of text has a `content` string and a `styles` string of equal length, where each character in the styles string encodes formatting (bold, italic, code, etc.). No AST, no tree, no recursive data structures.

That editor worked. 10,000 lines, 1.9ms keystroke latency, 243 Rust tests, incremental VDOM diffing. It proved that flat models with positional identity can replace tree-based architectures.

Meanwhile, a prior framework project called Minimact hit a wall. It used hex-path IDs to address nodes in a component tree. Every structural change — insert, reorder, delete — required cascading ID updates. The tree-addressing problem killed it.

The insight: the parallel strings model from the editor eliminates the tree entirely. A component's identity is its line number and character offset — trivially computable, zero maintenance. Outconceive was born.

## What Was Built

### Core Framework (Rust/WASM)

- **Parallel strings document model** — each UI row is a `Line` with four equal-length strings: `content`, `components`, `state_keys`, `styles`
- **Component span grouping** — consecutive same-type characters form component spans rendered to DOM elements
- **Flat state store** — `HashMap<String, StateValue>` with dirty tracking and reverse index (state key → line numbers)
- **VDOM generation** — document lines → VNode tree (nested only at container boundaries)
- **Recursive diff algorithm** — borrowed from the editor, operates line-by-line
- **Incremental rendering** — only changed lines re-render, O(1) for single-state updates
- **Markout parser and emitter** — markdown-like markup language, round-trips cleanly
- **SSR renderer** — VNode → HTML string for server-side rendering
- **wasm_bindgen API** — 25+ methods exposed to JavaScript

### Markout Language

A declarative markup where one line = one row of UI:

```
@card padding:24
| Username  {input:name validate:required}
| {button:submit "Save" primary}  {spacer:end}  {label:status animate:fade}
@end card
```

Features implemented:
- 17 component types: input, password, button, checkbox, radio, select, textarea, label, link, image, divider, spacer, pill, badge, progress, sparkline, custom
- Container directives: `@card`, `@form`, `@section`, `@nav`, `@header`, `@footer`, `@main`, `@aside`, `@columns`
- Column sizing: `col-6`, `col-3[5]` (custom denominators)
- Responsive breakpoints: `sm:col-6`, `md:col-4`, `lg:col-3`, `xl:col-2`
- Validation rules: `validate:required,email,min:3`
- Animations: `animate:fade`, `bounce`, `slide`, `scale`, `pulse`, `shake`, `glow`
- Routing: `route:/dashboard`
- Data fetching: `fetch:/api/users`
- Popovers: `popover:"Tooltip text"`
- Spacer modes: `{spacer:end}`, `{spacer:evenly}`, `{spacer:col-4-end}`
- Lists/repeaters: `@each items` / `@end each`
- Templates: `@define name` / `@use name scope=prefix`
- CSS columns: `@columns cols:3 gap:16 height:400px`
- Container config: `padding:24`, `max-width:600px`, `height:400px`

### JavaScript Runtime

- **`Outconceive` class** — developer-facing API wrapping the WASM core
  - `from_markout()` / `mount()` / `hydrate()` / `unmount()`
  - `set()` / `get()` / `getBool()`
  - `computed()` — derived state with multi-pass convergence
  - `effect()` — side effects on state change
  - `memo()` / `getMemo()` — cached computations
  - `on()` — action handlers for button clicks
  - `validate()` / `clearValidation()` / `addValidator()` — form validation engine
  - `fetch()` — declarative HTTP with auto loading/error states
  - `persist()` / `clearPersisted()` — localStorage auto-save/restore
  - `theme()` — per-instance or global theming
  - `animate()` — programmatic animation triggers
  - `connect()` / `publish()` / `subscribe()` / `syncState()` — bus integration
  - `source()` — export current Markout
- **`DomPatcher`** — applies VDOM patches to real DOM, SVG namespace support
- **`EventRouter`** — browser events → WASM state mutations, auto-handles remove/fetch/route
- **`OutconceiveRouter`** — hash-based page routing with declarative `route:` links
- **`OutconceiveBus`** — pub/sub event bus for inter-instance messaging with shared state

### CSS

- 30+ CSS custom properties for theming
- 3 built-in themes: light, dark, nord
- Custom theme support via variable objects
- 12-column responsive grid with 4 breakpoints (sm/md/lg/xl)
- Component styles: pills, badges, progress bars, sparklines
- 8 CSS animations with keyframes
- Popover tooltips via `::after` pseudo-elements
- Validation states (invalid/valid/error)
- Route-active highlighting
- Smooth 0.3s theme transitions

### Visual IDE

- Component toolbar: 7 component buttons + 3 container buttons
- Click-to-select rows in design mode
- Property panel with component details
- Editable Markout source panel with "Apply Changes"
- Design/preview mode toggle
- Delete, move up/down actions
- Status bar with line count

### Demo Pages (11)

| Page | URL | Features Demonstrated |
|------|-----|----------------------|
| `index.html` | Main demo | Login form, counter, columns |
| `multi.html` | Multi-mount | 4 independent instances, routing, nav |
| `todo.html` | Todo app | `@each` lists, add/remove, computed summary |
| `computed.html` | Computed state | Price calculator, derived values, effects |
| `validation.html` | Validation | Form validation with error display |
| `fetch.html` | Data fetching | Mock API, loading states, list population |
| `bus.html` | Messaging | 4 instances communicating via event bus |
| `persist.html` | Persistence | localStorage save/restore |
| `theme.html` | Theming | Light/dark/nord/custom theme switching |
| `responsive.html` | Responsive | Breakpoint-aware form layout |
| `animate.html` | Animations | Entrance, continuous, programmatic |
| `ssr.html` | SSR + hydration | Server render → client hydrate |
| `templates.html` | Templates | `@define`/`@use` with scoped state |
| `widgets.html` | Widgets | Pills, badges, progress, sparklines, popovers |
| `ide.html` | Visual IDE | Toolbar, properties, source editor |

### Documentation (VitePress)

- Homepage with hero, features, quick start
- 9 guide pages: getting started, core concepts, Markout, components, state, routing, lists, theming, templates, multi-mount, SSR, IDE
- 4 API reference pages: Markout syntax, JavaScript API, WASM API, CSS reference
- 4 architecture pages: overview, parallel strings, why no trees, performance
- Local search, dark mode, versioned paths

## Stats

| Metric | Value |
|--------|-------|
| Rust tests | 138 |
| WASM binary | ~230KB |
| JS runtime | ~20KB (5 files) |
| CSS | ~12KB |
| Demo pages | 15 |
| Doc pages | 20 (1,800+ lines) |
| Markout component types | 17 |
| Container types | 9 |
| CSS animations | 8 |
| Built-in themes | 3 |
| Responsive breakpoints | 4 |
| Validation rules | 7 |
| Spacer modes | 4 |

## Architecture

```
Markout → Parse → Document (flat Vec<Line>) → VDOM → Diff → Patches → DOM
```

No trees. No AST. No reconciliation. No build step. Position is identity.

## Feature Roadmap Completed

### Tier 1 — Core DX
1. Routing (hash-based, declarative `route:` links)
2. Lists/Repeaters (`@each` with auto-scoped state)
3. Computed State (multi-pass convergence, effects, memo)
4. Validation (declarative rules, custom validators)

### Tier 2 — Real Apps
5. Fetch/HTTP (declarative `fetch:`, loading/error states)
6. Inter-instance Messaging (OutconceiveBus, syncState)
7. Persistence (localStorage auto-save/restore)
8. Theming (CSS variables, 3 presets, custom themes)

### Tier 3 — Production
9. Responsive Layouts (4 breakpoints, 12-column grid)
10. Transitions/Animations (8 built-in, programmatic triggers)
11. SSR + Hydration (Rust HTML renderer, JS hydrate)
12. Templates/Slots (`@define`/`@use` with scope)

### Additional
- Column sizing with custom denominators (`col-3[5]`)
- CSS columns layout (`@columns`)
- Spacers (`{spacer:end}`, `{spacer:col-4-end}`)
- Popovers (`popover:"text"`)
- Widgets (pill, badge, progress bar, sparkline)
- Visual IDE (toolbar, properties, source editor)
- Documentation site (VitePress, 20 pages)
- Multi-mount island architecture

## The Key Insight

The environment needs trees. Web performance doesn't.
