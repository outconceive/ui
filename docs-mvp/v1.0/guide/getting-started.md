# Getting Started

Outconceive UI is a web framework that replaces component trees with flat, parallel strings. You write Markout — a markdown-like syntax — and get a fully interactive app with no build step.

## Installation

### Script Tag (Quickest)

```html
<script src="js/dom-patcher.js"></script>
<script src="js/event-router.js"></script>
<script src="js/outconceive.js"></script>
<script type="module">
  import init, { OutconceiveApp } from './pkg/outconceive.js';
  await init();
  // Your app here
</script>
```

### From Source

```bash
git clone https://github.com/outconceive-ui/outconceive.git
cd outconceive
cargo test           # Run 133 Rust tests
wasm-pack build --target web --out-dir www/pkg
node serve.js        # Dev server on port 9096
```

## Your First App

```javascript
var app = new Outconceive(new OutconceiveApp());

app.from_markout(`
  @card padding:24
  | Welcome to Outconceive
  | Name: {input:name}
  | {button:greet "Say Hello" primary}
  | {label:greeting animate:fade}
  @end card
`);

app.mount('app');

app.on('greet', (wasmApp) => {
  var name = app.get('name') || 'World';
  app.set('greeting', 'Hello, ' + name + '!');
});
```

This creates a card with a text input, a button, and a reactive label — all from 6 lines of Markout.

## What Just Happened?

1. **`from_markout()`** parsed the Markout into parallel strings (content, components, state keys, styles)
2. **`mount('app')`** rendered the VDOM to the DOM and attached event handlers
3. **`on('greet')`** registered an action handler — when the button is clicked, it updates the `greeting` state
4. **`set('greeting')`** updated the WASM state, diffed the VDOM, and applied minimal patches to the DOM

No virtual DOM tree was built. No reconciliation algorithm ran. The diff happened at the line level — O(1) for a single state change.

## Next Steps

- [Core Concepts](/v1.0/guide/core-concepts) — understand parallel strings and the line model
- [Markout Syntax](/v1.0/guide/markout) — the full markup reference
- [Components](/v1.0/guide/components) — all available component types
