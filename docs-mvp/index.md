---
layout: home

hero:
  name: Outconceive UI
  text: The Parallel Strings Framework
  tagline: Build web UIs with flat markup, not nested trees. One line of Markout = one row of your app. No JSX, no virtual DOM trees, no reconciliation complexity.
  actions:
    - theme: brand
      text: Get Started
      link: /v1.0/guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/outconceive-ui/outconceive

features:
  - icon: ⚡
    title: No Trees
    details: Components are flat lines with parallel strings — no nesting, no recursive reconciliation. Position is identity.
  - icon: 📝
    title: Markout
    details: A markdown-like syntax for UI. One line defines a row of components with state bindings, styles, and layout.
  - icon: 🦀
    title: Rust/WASM Core
    details: All rendering, diffing, and state management runs in 140KB of WebAssembly. No JS framework dependency.
  - icon: 🎨
    title: Visual IDE
    details: The editor IS the compiler. Click toolbar buttons to insert components. Modals configure properties. No code needed.
  - icon: 🏝️
    title: Island Architecture
    details: Mount independent Outconceive instances anywhere. Each island has its own state, its own WASM core. One script tag.
  - icon: 🚀
    title: SSR + Hydration
    details: Render Markout to HTML on the server. Hydrate on the client. Sub-millisecond time to interactive.
---

## The Counter App

The iconic framework demo, in one line of Markout:

```
{button:dec "-" outline}  {label:count}  {button:inc "+" primary}
```

Compare to React:

```jsx
function Counter() {
  const [count, setCount] = useState(0);
  return (
    <div>
      <button onClick={() => setCount(c => c - 1)}>-</button>
      <span>{count}</span>
      <button onClick={() => setCount(c => c + 1)}>+</button>
    </div>
  );
}
```

Same result. Zero nesting. Zero build step.

## Quick Start

```bash
npm install outconceive
```

```html
<div id="app"></div>
<script src="outconceive.js"></script>
<script type="module">
  import init, { OutconceiveApp } from './pkg/outconceive.js';
  await init();

  var app = new Outconceive(new OutconceiveApp());
  app.from_markout(`
    @card padding:24
    | Hello, {input:name}!
    | {button:greet "Say Hello" primary}
    | {label:message}
    @end card
  `);
  app.mount('app');

  app.on('greet', (wasmApp) => {
    app.set('message', 'Hello, ' + app.get('name') + '!');
  });
</script>
```
