# Multi-Mount & Messaging

Mount independent Outconceive instances to different DOM elements. Each has its own state, WASM core, and event handlers.

## Mounting Multiple Instances

```html
<div id="header"></div>
<div id="sidebar"></div>
<div id="main"></div>
```

```javascript
var header = new Outconceive(new OutconceiveApp());
header.from_markout('| My App  {input:search}');
header.mount('header');

var sidebar = new Outconceive(new OutconceiveApp());
sidebar.from_markout('| {button:nav "Home" ghost route:/home}');
sidebar.mount('sidebar');

var main = new Outconceive(new OutconceiveApp());
main.from_markout('@card padding:24\n| Welcome\n@end card');
main.mount('main');
```

Page layout is pure CSS on the host elements. Each instance is an independent island.

## Unmounting

```javascript
main.unmount();  // Clears DOM, removes event handlers
```

## Event Bus

Instances communicate via `OutconceiveBus`:

```javascript
var bus = new OutconceiveBus();

// Instance A publishes
sidebar.connect(bus);
sidebar.on('nav_click', function() {
    sidebar.publish('navigate', { page: 'users' });
});

// Instance B subscribes
main.connect(bus);
main.subscribe('navigate', function(data, self) {
    self.from_markout(pages[data.page]);
});
```

## Bus API

| Method | Description |
|--------|-------------|
| `bus.on(event, handler)` | Subscribe to events |
| `bus.emit(event, data)` | Publish an event |
| `bus.once(event, handler)` | One-shot subscription |
| `bus.off(event, handler)` | Unsubscribe |
| `bus.set(key, value)` | Set shared state (auto-emits `state:key`) |
| `bus.get(key)` | Read shared state |
| `bus.watch(key, handler)` | Subscribe + get current value |

## Synced State

Two-way state sync between an instance and the bus:

```javascript
instanceC.syncState('username', bus);
instanceD.syncState('username', bus);
```

Type in Instance C's input → Instance D's label updates in real-time. Both instances share the `username` key through the bus.

:::tip
This is Outconceive's answer to React Context or Vue's provide/inject — but across fully independent WASM instances with no shared runtime.
:::
