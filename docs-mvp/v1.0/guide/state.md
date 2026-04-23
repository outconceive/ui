# State & Reactivity

Outconceive uses a flat key-value state store. When a value changes, only the lines that reference that key re-render.

## Basic State

```javascript
app.set('name', 'Alice');
app.set('count', '42');
app.set('active', true);

app.get('name');        // 'Alice'
app.getBool('active');  // true
```

State keys are strings. Values are stored as text internally — booleans are toggled via `toggle_state`.

## Reactive Labels

Labels bound to state keys update automatically:

```
| Hello, {label:name}!
```

When `app.set('name', 'Bob')` is called, the label updates to "Bob" — only that line re-renders.

## Computed State

Derived values that auto-recalculate when dependencies change:

```javascript
app.computed('total', ['price', 'qty'], function(get) {
    var p = parseFloat(get('price') || '0');
    var q = parseFloat(get('qty') || '0');
    return '$' + (p * q).toFixed(2);
});
```

The `get` helper provides typed access:
- `get('key')` — text value
- `get.bool('key')` — boolean
- `get.count('list')` — list item count

Computed values can depend on other computed values — Outconceive runs multiple passes until values stabilize.

## Effects

Side effects that fire when dependencies change:

```javascript
app.effect(['total'], function(get) {
    console.log('Total updated:', get('total'));
});
```

Effects run after all computed values settle. They only fire when the tracked value actually changes.

## Memo

Cached expensive computations:

```javascript
app.memo('filtered', ['items', 'query'], function(get) {
    // Expensive filtering...
    return result;
});

var result = app.getMemo('filtered');
```

Only recomputes when dependencies change.

## List State

Lists use dot-scoped keys: `todos.0.text`, `todos.1.done`.

```javascript
// Add an item
app.app.add_list_item('todos', JSON.stringify({
    text: 'Buy milk', done: false
}));

// Remove an item
app.app.remove_list_item('todos', 0);

// Get count
app.app.get_list_count('todos');

// Read scoped value
app.get('todos.0.text');
```

## Persistence

Auto-save state to localStorage:

```javascript
// Persist specific keys
app.persist('myapp', ['name', 'email', 'theme']);

// Persist all state changes
app.persist('myapp');

// Clear saved data
app.clearPersisted();
```

State is restored on next page load automatically.

:::tip
Only persist user-facing state (form values, preferences). Don't persist transient state like loading indicators.
:::
