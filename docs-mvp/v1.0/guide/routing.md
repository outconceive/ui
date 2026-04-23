# Routing

Outconceive includes hash-based routing for single-page apps. Each route maps a URL hash to Markout content.

## Setup

```javascript
var mainView = new Outconceive(new OutconceiveApp());
mainView.mount('main');

var router = new OutconceiveRouter(mainView);
router
    .route('/dashboard', '@card padding:24\n| Dashboard\n@end card')
    .route('/settings', '@card padding:24\n| Settings\n@end card')
    .default('/dashboard')
    .start();
```

When the URL changes to `#/settings`, the router swaps the main view's Markout content.

## Declarative Navigation

Use `route:` on buttons to create nav links without JS:

```
| {button:nav "Dashboard" ghost route:/dashboard}
| {button:nav "Users" ghost route:/users}
| {button:nav "Settings" ghost route:/settings}
```

Clicking the button changes the URL hash and triggers the router. The active route's button gets a `route-active` CSS class automatically.

## Route Registration

Routes accept Markout strings or functions:

```javascript
// Static content
router.route('/about', '| About this app');

// Dynamic content (function called on each navigation)
router.route('/profile', function() {
    return '@card padding:24\n| Hello, ' + username + '\n@end card';
});
```

## API

| Method | Description |
|--------|-------------|
| `router.route(path, markout)` | Register a route |
| `router.default(path)` | Set default route |
| `router.start()` | Begin listening to hash changes |
| `router.stop()` | Stop listening |
| `router.navigate(path)` | Programmatic navigation |

## Browser History

Back/forward buttons work via the `hashchange` event. The router syncs automatically.
