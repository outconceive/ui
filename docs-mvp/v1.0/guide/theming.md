# Theming

Outconceive's stylesheet uses CSS custom properties. Switch themes instantly with no re-render.

## Built-in Themes

```javascript
Outconceive.theme('light');  // Default
Outconceive.theme('dark');   // Deep navy
Outconceive.theme('nord');   // Arctic blue
```

## Custom Themes

Pass an object to override any CSS variable:

```javascript
Outconceive.theme({
    'bg': '#fdf6e3',
    'text': '#657b83',
    'accent': '#268bd2',
    'card-bg': '#eee8d5',
    'danger': '#dc322f',
});
```

## Per-Instance Themes

Apply a theme to one instance without affecting others:

```javascript
sidebar.theme('dark');
mainView.theme('light');
```

## CSS Variables

| Variable | Default (light) | Description |
|----------|----------------|-------------|
| `--mc-bg` | `#f5f5f5` | Page background |
| `--mc-text` | `#333` | Primary text |
| `--mc-text-muted` | `#555` | Secondary text |
| `--mc-card-bg` | `#fff` | Card background |
| `--mc-card-shadow` | `0 2px 8px rgba(0,0,0,0.1)` | Card shadow |
| `--mc-border` | `#ccc` | Input/button border |
| `--mc-input-bg` | `#fff` | Input background |
| `--mc-accent` | `#4a90d9` | Primary accent color |
| `--mc-accent-hover` | `#3a7bc8` | Accent hover state |
| `--mc-danger` | `#dc3545` | Danger/error color |
| `--mc-warning` | `#ffc107` | Warning color |
| `--mc-success` | `#28a745` | Success/valid color |
| `--mc-radius` | `4px` | Border radius |
| `--mc-radius-lg` | `8px` | Large border radius |

All components — inputs, buttons, cards, labels — transition smoothly (0.3s) when the theme changes.
