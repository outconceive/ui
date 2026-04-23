# Lists & Repeaters

The `@each` directive renders a template for every item in a list.

## Basic Usage

```
@each todos
| {checkbox:done}  {label:text}  {button:remove "X" danger}
@end each
```

State keys inside `@each` are auto-scoped. `done` becomes `todos.0.done`, `todos.1.done`, etc.

## Managing List Data

```javascript
// Add items
app.app.add_list_item('todos', JSON.stringify({
    text: 'Buy milk', done: false
}));

app.app.add_list_item('todos', JSON.stringify({
    text: 'Write code', done: true
}));

// Remove by index
app.app.remove_list_item('todos', 0);

// Update a specific item
app.app.set_list_item('todos', 1, JSON.stringify({
    text: 'Updated text', done: true
}));

// Get count
app.app.get_list_count('todos');
```

## Auto-Remove Buttons

Buttons with state keys ending in `remove` or `delete` inside `@each` automatically get wired to remove the item:

```
@each items
| {label:name}  {button:remove "X" danger}
@end each
```

Clicking "X" removes that item from the list — no action handler needed.

## Computed Summaries

Use computed state to derive values from lists:

```javascript
app.computed('summary', ['todos'], function(get) {
    var count = get.count('todos');
    var done = 0;
    for (var i = 0; i < count; i++) {
        if (get.bool('todos.' + i + '.done')) done++;
    }
    return done + '/' + count + ' completed';
});
```

```
| {label:summary}
```

## Example: Todo App

```
@card padding:24
| {input:new_todo col-9}  {button:add "Add" primary col-3}
@end card

@each todos
| {checkbox:done}  {label:text}  {button:remove "X" danger}
@end each

| {label:summary}
```

```javascript
app.on('add', function(wasmApp) {
    var text = wasmApp.get_state('new_todo');
    if (!text) return;
    wasmApp.add_list_item('todos', JSON.stringify({ text: text, done: false }));
    app.patcher.applyPatches(wasmApp.update_state('new_todo', ''));
    app.patcher.applyPatches(wasmApp.render());
});
```
