# Components

Every component in Outconceive is a character in the parallel `components` string. In Markout, you use `{type:binding}` syntax.

## Input

```
| {input:username}
| {input:email validate:required,email}
| {input:search col-8}
```

Renders `<input type="text">` bound to the state key. Supports `validate:` and `col-` sizing.

## Password

```
| {password:pass}
```

Renders `<input type="password">` with masked characters.

## Button

```
| {button:submit "Save" primary}
| {button:cancel "Cancel" outline}
| {button:delete "Remove" danger}
```

The first argument is the action name (dispatched on click). The quoted string is the label. The last argument is the style.

## Checkbox

```
| {checkbox:agree} I agree to the terms
```

Renders `<input type="checkbox">` bound to a boolean state key. Use `app.getBool('agree')` to read the value.

## Radio

```
| {radio:choice}
```

Renders `<input type="radio">` bound to a state key.

## Select

```
| {select:theme}
```

Renders `<select>` bound to a state key.

## Textarea

```
| {textarea:content}
```

Renders `<textarea>` bound to a state key.

## Label

Static text is automatically a label. For reactive labels bound to state:

```
| Status: {label:status}
| Count: {label:count animate:fade}
```

The label's text content updates when the bound state key changes.

## Link

```
| {link:home href=https://example.com}
```

Renders `<a>` with the specified URL.

## Image

```
| {image:photo href=/images/photo.jpg}
```

Renders `<img>` with the specified source URL.

## Divider

```
| {divider}
```

Renders `<hr>`.

## Pill

A rounded tag/chip for status indicators:

```
| {pill:status "Active" primary}
| {pill:tag "Beta" warning}
| {pill:env "Production" danger}
```

Reactive — bind to a state key and the text updates:

```javascript
app.set('status', 'Deployed');
```

## Badge

A compact notification counter:

```
| Messages {badge:msg_count "5" danger}
| Updates {badge:updates "99+" primary}
```

Typically placed after a label or button to show a count.

## Progress Bar

A horizontal bar that fills based on a numeric state value (0-100):

```
| Upload: {progress:upload}
| CPU: {progress:cpu danger}
```

```javascript
app.set('upload', '75');  // 75% filled, animates smoothly
```

Supports style colors: `primary` (default), `danger`, `warning`, `success`.

## Sparkline

An inline SVG chart rendered from comma-separated values:

```
| Revenue {sparkline:revenue}
| Errors {sparkline:errors danger}
```

```javascript
app.set('revenue', '10,25,18,30,45,38,52');
```

Updates reactively — set the state key to new values and the chart redraws. Renders as an SVG polyline with a filled area underneath.

## Spacer

A layout component for controlling spacing between items in a row:

```
| Logo  {spacer:end}  {button:login "Sign In" primary}
```

Modes:

| Syntax | Effect |
|--------|--------|
| `{spacer:end}` | Pushes everything after it to the right (`flex:1`) |
| `{spacer:evenly}` | Equal distribution between items (use between each) |
| `{spacer:col-3}` | Fixed 3/12 width gap |
| `{spacer:col-4-end}` | Fill up to end of column 4 (next item starts at column 5) |

Tab-stop alignment:

```
| {label:id col-1}  {spacer:col-4-end}  {label:name col-4}  {spacer:col-8-end}  {button:edit "Edit" col-4}
```

## Popover

A tooltip that appears on hover. Add `popover:` to any component:

```
| {button:info "Details" outline popover:"More information here"}
| {label:tip popover:"Extra context on hover"}
| {pill:help "?" primary popover:"Need help?"}
```

Pure CSS — no JavaScript needed. Works on buttons, labels, pills, inputs, or any component.

## Editor

A rich text editor powered by a separate Rust/WASM engine. Features are opt-in — only the formatting you declare is available:

```
@editor bold italic code heading list bind:content
@end editor
```

A minimal notes field:

```
@editor bold italic bind:notes
@end editor
```

A full-featured editor:

```
@editor bold italic underline strikethrough code heading list ordered-list quote code-block link hr bind:article
@end editor
```

Available features:

| Feature | Description |
|---------|-------------|
| `bold` | Bold text (Ctrl+B) |
| `italic` | Italic text (Ctrl+I) |
| `underline` | Underlined text (Ctrl+U) |
| `strikethrough` | Strikethrough text |
| `code` | Inline code (Ctrl+`) |
| `heading` | Heading levels |
| `list` | Unordered list |
| `ordered-list` | Ordered list |
| `quote` | Block quote |
| `code-block` | Fenced code block |
| `link` | Hyperlinks (Ctrl+K) |
| `hr` | Horizontal divider |

The feature list is a contract — if `bold` isn't declared, pasted bold text is stripped. Content can never contain formatting the developer didn't opt into.

Bind to state with `bind:key` to read/write the editor's markdown content:

```javascript
app.on('save', function() {
    var editor = app.getEditor('content');
    var markdown = editor.getContent();
});
```

## Component Attributes

All components support these optional attributes:

| Attribute | Example | Effect |
|-----------|---------|--------|
| `col-N` | `col-6` | Width as N/12 columns |
| `col-N[M]` | `col-3[5]` | Width as N/M fraction |
| `sm:col-N` | `md:col-6` | Responsive sizing |
| `validate:rules` | `validate:required,email` | Form validation |
| `animate:name` | `animate:fade` | CSS animation |
| `route:path` | `route:/dashboard` | Navigation link |
| `fetch:url` | `fetch:/api/data` | Data loading trigger |
| `popover:"text"` | `popover:"Help text"` | Hover tooltip |

## Styles

| Keyword | Color |
|---------|-------|
| `primary` | Blue accent |
| `secondary` | Gray |
| `danger` | Red |
| `warning` | Yellow |
| `info` | Cyan |
| `outline` | Transparent with border |
| `ghost` | Transparent, no border |
| `dark` | Dark |
| `light` | Light |
