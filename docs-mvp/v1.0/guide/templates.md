# Templates & Slots

Define reusable Markout fragments with `@define` and instantiate them with `@use`.

## Defining a Template

```
@define contact_card
@card padding:16
| {label:name}
| Email: {label:email}
| {button:msg "Message" primary}
@end card
@end define
```

The template is stored at parse time and not rendered until `@use` is called.

## Using a Template

```
@use contact_card scope=alice
@use contact_card scope=bob
```

Each `@use` expands the template with all state keys scoped by the `scope` parameter. `name` becomes `alice.name`, `email` becomes `alice.email`, etc.

## Setting Scoped State

```javascript
app.set('alice.name', 'Alice Johnson');
app.set('alice.email', 'alice@example.com');
app.set('bob.name', 'Bob Smith');
app.set('bob.email', 'bob@example.com');
```

Each instance is fully independent — changing `alice.name` doesn't affect `bob.name`.

## Auto-Scoping

If you omit the `scope` parameter, Outconceive generates a unique scope automatically:

```
@use contact_card
@use contact_card
```

The first use gets scope `contact_card_0`, the second gets `contact_card_1`.

## Multi-Line Templates

Templates can contain any Markout — containers, components, validation, animations:

```
@define form_field
| {label:label_text col-4}  {input:value validate:required col-8}
@end define

@use form_field scope=field_name
@use form_field scope=field_email
@use form_field scope=field_phone
```

```javascript
app.set('field_name.label_text', 'Full Name');
app.set('field_email.label_text', 'Email');
app.set('field_phone.label_text', 'Phone');
```

:::tip
Templates are expanded at parse time, not render time. They're a compile-time convenience, not a runtime feature. This means zero overhead.
:::

## Templates vs @each

| Feature | `@define`/`@use` | `@each` |
|---------|-----------------|---------|
| Count | Fixed at parse time | Dynamic from state |
| Scoping | Explicit `scope=` param | Automatic index-based |
| Use case | Reusable UI patterns | Data-driven lists |
| Overhead | Zero (parse-time expansion) | Per-render (list iteration) |
