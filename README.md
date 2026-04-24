# Outconceive UI

A Rust/WASM web framework with its own markup language. Write declarative UI in **Markout**, get reactive DOM with no build step, no virtual DOM diffing overhead, and no component trees.

## What it looks like

```
@card padding:24
| Welcome back, {label:user animate:fade}
| Email     {input:email validate:required,email col-6}
| Password  {password:pass validate:required,min:8 col-6}
| {button:login "Sign In" primary}  {spacer:end}  {link:forgot "Forgot password?" ghost}
@end card
```

```javascript
const app = Outconceive.from_markout(markout);
app.mount('#app');

app.on('login', () => {
    if (app.validate()) {
        app.fetch('/api/login', { email: app.get('email'), pass: app.get('pass') });
    }
});

app.computed('user', () => app.get('email').split('@')[0] || 'stranger');
app.persist('email');
app.theme('dark');
```

That's a validated login form with computed display name, dark theme, persistent email, and async submit. No JSX, no transpiler, no node_modules.

## Features

**17 components** — input, password, button, checkbox, radio, select, textarea, label, link, image, divider, spacer, pill, badge, progress bar, sparkline, custom

**9 containers** — `@card`, `@form`, `@section`, `@nav`, `@header`, `@footer`, `@main`, `@aside`, `@columns`

**Reactive state** — `set()` / `get()` / `computed()` / `effect()` / `memo()` with automatic dirty tracking and incremental re-render

**Routing** — hash-based with declarative `route:/path` links and `route-active` CSS class

**Lists** — `@each items` / `@end each` with auto-scoped state per item

**Form validation** — `validate:required,email,min:3,max:50,pattern:regex` with custom validators

**Data fetching** — `fetch:/api/endpoint` with automatic loading/error state management

**Theming** — 3 built-in themes (light, dark, nord), 30+ CSS custom properties, custom themes via JS

**Responsive grid** — 12-column with `col-6`, `col-3[5]` (custom denominators), and 4 breakpoints (`sm:col-6`, `md:col-4`, `lg:col-3`, `xl:col-2`)

**8 animations** — fade, slide, slide-up, scale, bounce, pulse, shake, glow — declarative or programmatic

**SSR + hydration** — Rust renders HTML strings, JS hydrates without re-render

**Templates** — `@define` / `@use` with scoped state for reusable blocks

**Persistence** — `app.persist('key')` auto-saves/restores from localStorage

**Multi-mount** — independent instances as islands, connected via pub/sub event bus

**Popovers** — `popover:"Tooltip text"` on any component, pure CSS

**Visual IDE** — toolbar, click-to-select, property panel, live Markout source editor

## Why no trees?

Every other framework models UI as a tree: components own children, identity requires generated keys, structural changes cascade through the hierarchy.

Outconceive uses **parallel strings**. Each UI row is four equal-length strings:

```
content:    "Username  ________  Login "
components: "LLLLLLLLLLIIIIIIIIIIBBBBBB"
state_keys: "__________username__submit"
styles:     "                    pppppp"
```

A component's identity is its line number and character offset — trivially computable, zero maintenance. No reconciliation, no key diffing, no tree walks. State updates re-render only the affected lines: O(1).

## Getting started

```html
<script type="module">
    import init, { OutconceiveApp } from './pkg/outconceive.js';
    await init();

    const app = Outconceive.from_markout(OutconceiveApp, `
        @card padding:16
        | Count: {label:count}
        | {button:inc "+" primary}  {button:dec "-" danger}
        @end card
    `);

    app.mount('#app');
    app.set('count', '0');
    app.on('inc', () => app.set('count', String(+app.get('count') + 1)));
    app.on('dec', () => app.set('count', String(+app.get('count') - 1)));
</script>
```

## Tech stack

| Layer | Technology | Size |
|-------|-----------|------|
| Core | Rust → WASM | ~230KB |
| Runtime | Vanilla JS (no dependencies) | ~20KB |
| Styles | CSS custom properties | ~12KB |
| Docs | VitePress | 20 pages |

138 Rust tests. No build step required. No npm dependencies at runtime.

## Build

```sh
# Rust tests
cargo test

# WASM build
wasm-pack build --target web

# Dev server
node serve.js
# → http://localhost:9096
```

## Demos

15 demo pages covering login forms, todo lists, price calculators, form validation, data fetching, inter-instance messaging, persistence, theming, responsive layouts, animations, SSR + hydration, templates, widgets, and the visual IDE.

## Docs

Full documentation site built with VitePress — guides, API reference, and architecture deep dives.

## License

MIT
