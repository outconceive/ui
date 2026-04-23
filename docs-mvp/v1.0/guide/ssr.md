# SSR & Hydration

Render Markout to static HTML on the server (or in WASM), then hydrate on the client for instant interactivity.

## Server-Side Rendering

```javascript
var app = new OutconceiveApp();
app.from_markout(markoutSource);
var html = app.render_to_html();
// Send `html` to the client
```

Or use the static method without an instance:

```javascript
var html = OutconceiveApp.markout_to_html(markoutSource);
```

The output is a complete HTML string with all components rendered — inputs, buttons, labels, containers.

## Client-Side Hydration

Instead of `mount()` (which re-renders from scratch), use `hydrate()` to attach to existing SSR HTML:

```javascript
// Server already injected HTML into #app
var app = new Outconceive(new OutconceiveApp());
app.from_markout(sameMarkoutSource);
app.hydrate('app');  // Attach events, no re-render
```

## mount() vs hydrate()

| | `mount()` | `hydrate()` |
|---|-----------|-------------|
| Clears DOM | Yes | No |
| Renders VDOM | Yes | No (caches only) |
| Attaches events | Yes | Yes |
| Use case | Client-only | After SSR |
| Time to interactive | ~2ms | ~0.3ms |

## Timing

SSR rendering takes ~0.5ms for a typical form. Hydration takes ~0.3ms. Combined: the user sees content instantly (no loading spinner) and can interact in under 1ms after WASM loads.

## Workflow

1. Server renders Markout → HTML string
2. HTML is included in the page response (or injected via `innerHTML`)
3. Client loads WASM (~140KB)
4. Client calls `hydrate()` — events attached, app is live
5. Subsequent state updates use the normal patch pipeline

:::warning
The Markout source passed to `hydrate()` must match what was rendered on the server. If they differ, the VDOM cache will be out of sync with the DOM, and patches may apply incorrectly.
:::
