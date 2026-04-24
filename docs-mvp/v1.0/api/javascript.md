# JavaScript API

The `Outconceive` class wraps the WASM core with a developer-friendly API.

## Constructor

```javascript
var app = new Outconceive(new OutconceiveApp());
```

## Lifecycle

| Method | Description |
|--------|-------------|
| `app.from_markout(source)` | Parse Markout and load document |
| `app.mount(target)` | Render to DOM element (id string or element) |
| `app.hydrate(target)` | Attach to existing SSR HTML without re-render |
| `app.unmount()` | Remove from DOM, clean up |
| `app.source()` | Get current Markout source |

## State

| Method | Description |
|--------|-------------|
| `app.set(key, value)` | Set state (string or boolean) |
| `app.get(key)` | Get state as string |
| `app.getBool(key)` | Get state as boolean |

## Computed State

```javascript
app.computed(key, deps, fn)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | string | State key to write the computed value to |
| `deps` | string[] | State keys that trigger recomputation |
| `fn` | function | `fn(get)` → returns computed value |

The `get` helper:
- `get(key)` — text value
- `get.bool(key)` — boolean
- `get.count(key)` — list count

## Effects

```javascript
app.effect(deps, fn)
```

Runs `fn(get)` when any dep changes. Fires after computed values settle.

## Memo

```javascript
app.memo(key, deps, fn)
var value = app.getMemo(key)
```

Cached computation. Only recomputes when deps change.

## Actions

```javascript
app.on(actionName, fn)
```

Registers a handler for button clicks. `fn` receives the WASM `OutconceiveApp` instance.

## Validation

| Method | Description |
|--------|-------------|
| `app.validate()` | Returns `{ valid: bool, errors: { key: message } }` |
| `app.clearValidation()` | Remove all error states |
| `app.addValidator(name, fn)` | Register custom validator |

Custom validator: `fn(value, param)` → returns error string or `null`.

## Fetch

```javascript
app.fetch(stateKey, url, options?)
```

Options:
- `method` — HTTP method (default: GET)
- `body` — Request body
- `headers` — Request headers
- `transform` — `fn(data)` to transform response
- `onSuccess` — Callback on success
- `onError` — Callback on error

Auto-managed state: `key._loading`, `key._error`.

## Persistence

| Method | Description |
|--------|-------------|
| `app.persist(namespace, keys?)` | Auto-save/restore state to localStorage |
| `app.clearPersisted()` | Clear saved data |

## Theming

| Method | Description |
|--------|-------------|
| `Outconceive.theme(name)` | Set global theme (`'light'`, `'dark'`, `'nord'`) |
| `Outconceive.theme(vars)` | Set custom theme via CSS variable object |
| `app.theme(name)` | Set per-instance theme |

## Animation

```javascript
app.animate(stateKey, animationName)
```

Re-triggers a CSS animation on the element bound to `stateKey`.

## Bus Integration

| Method | Description |
|--------|-------------|
| `app.connect(bus)` | Attach to a OutconceiveBus |
| `app.publish(event, data)` | Emit event via bus |
| `app.subscribe(event, fn)` | Listen for bus events. `fn(data, self)` |
| `app.syncState(key, bus)` | Two-way state sync with bus |

## Editor

```javascript
var editor = app.getEditor(bindKey)
```

Returns the `EditorBridge` instance for the `@editor` bound to the given state key, or `null` if not found.

| Method | Description |
|--------|-------------|
| `editor.getContent()` | Get editor content as markdown |
| `editor.setContent(md)` | Set editor content from markdown |
| `editor.destroy()` | Remove editor and free WASM memory |

The editor module (`editor-bridge.js`) is an ES module — import it before mounting:

```javascript
import { EditorBridge } from './js/editor-bridge.js';
window.EditorBridge = EditorBridge;
```

Pages without `@editor` containers don't need to load it.

## Static Methods

| Method | Description |
|--------|-------------|
| `Outconceive.bus()` | Create a new OutconceiveBus instance |
| `Outconceive.theme(name)` | Set global theme |
| `Outconceive.themes` | Available theme names |
