# Markout Syntax

Markout is a markdown-like markup language for defining UIs. Each line starting with `|` defines a row of components.

## Basic Structure

```
| Plain text becomes a label
| {input:fieldname}
| {button:action "Label" style}
```

## Components

Components use `{type:binding "label" style}` syntax:

| Syntax | Renders As |
|--------|-----------|
| `{input:name}` | `<input type="text">` |
| `{password:pass}` | `<input type="password">` |
| `{button:action "Label" primary}` | `<button class="mc-primary">` |
| `{checkbox:agreed}` | `<input type="checkbox">` |
| `{radio:choice}` | `<input type="radio">` |
| `{select:option}` | `<select>` |
| `{textarea:content}` | `<textarea>` |
| `{label:key}` | Reactive `<span>` bound to state |
| `{link:nav href=url}` | `<a href="url">` |
| `{image:img href=src}` | `<img src="src">` |
| `{divider}` | `<hr>` |

## Text

Plain text in a `|` line becomes label components:

```
| Username  {input:name}
```

Here, `Username  ` is a static label, `{input:name}` is a text input bound to the `name` state key.

## Styles

Add a style keyword after the binding:

```
| {button:submit "Save" primary}
| {button:cancel "Cancel" outline}
| {button:delete "Delete" danger}
```

Available styles: `primary`, `secondary`, `danger`, `warning`, `info`, `dark`, `light`, `outline`, `ghost`.

## Containers

Wrap rows in containers using `@tag` / `@end tag`:

```
@card padding:24
| Card content here
@end card

@form
| {input:email validate:required,email}
| {button:submit "Submit" primary}
@end form

@section
| Section content
@end section
```

Container config is key:value pairs: `padding:24`, `max-width:600px`, `shadow:md`.

## Columns Layout

CSS columns for newspaper-style flow:

```
@columns cols:3 gap:16 height:400px
| Content flows across columns automatically...
| More content...
@end columns
```

## Column Sizing

Control component width with `col-N` (12-column grid):

```
| {input:name col-8}  {button:go "Go" primary col-4}
```

Custom denominators: `col-3[5]` = 3/5ths width.

## Responsive Sizing

Breakpoint-aware sizing:

```
| {input:name col-12 md:col-6 lg:col-4}
```

Breakpoints: `sm` (576px), `md` (768px), `lg` (992px), `xl` (1200px).

## Validation

Add validation rules to inputs:

```
| {input:email validate:required,email}
| {input:name validate:required,min:3}
| {password:pass validate:required,min:8}
```

Rules: `required`, `email`, `number`, `url`, `min:N`, `max:N`, `pattern:regex`.

## Routing

Declarative navigation with `route:`:

```
| {button:nav "Dashboard" ghost route:/dashboard}
| {button:nav "Settings" ghost route:/settings}
```

## Fetch

Declarative data loading with `fetch:`:

```
| {button:load "Load Users" primary fetch:/api/users}
```

## Animations

Add entrance/continuous animations:

```
| {label:status animate:fade}
| {button:go "Go" primary animate:bounce}
```

Available: `fade`, `slide`, `slide-up`, `scale`, `bounce`, `pulse`, `shake`, `glow`.

## Lists / Repeaters

Render dynamic lists with `@each`:

```
@each todos
| {checkbox:done}  {label:text}  {button:remove "X" danger}
@end each
```

State keys inside `@each` are auto-scoped: `done` → `todos.0.done`, `todos.1.done`, etc.

## Templates

Define reusable fragments with `@define` / `@use`:

```
@define contact_card
@card padding:16
| {label:name}
| {label:email}
| {button:msg "Message" primary}
@end card
@end define

@use contact_card scope=alice
@use contact_card scope=bob
```

Each `@use` gets independent state via scoping.
