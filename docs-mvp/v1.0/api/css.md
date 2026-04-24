# CSS Reference

All Outconceive CSS classes use the `mc-` prefix. The stylesheet is built on CSS custom properties for theming.

## Layout

| Class | Description |
|-------|-------------|
| `.mc-app` | Root container (`max-width: 800px`) |
| `.mc-row` | Flex row (one line of components) |
| `.mc-row.mc-grid` | Grid layout row |
| `.mc-row.mc-flex` | Explicit flex row |

## Containers

| Class | Description |
|-------|-------------|
| `.mc-card` | Rounded card with shadow |
| `.mc-modal` | Modal dialog |
| `.mc-sidebar` | Sidebar with border |
| `.mc-columns` | CSS columns container |

## Components

| Class | Element |
|-------|---------|
| `.mc-label` | Text label |
| `.mc-input` | Text input |
| `.mc-input-password` | Password input (wider letter spacing) |
| `.mc-button` | Button |
| `.mc-checkbox` | Checkbox |
| `.mc-radio` | Radio button |
| `.mc-select` | Select dropdown |
| `.mc-textarea` | Textarea |
| `.mc-link` | Hyperlink |
| `.mc-image` | Image |
| `.mc-divider` | Horizontal rule |
| `.mc-spacer` | Flex spacer |
| `.mc-pill` | Rounded pill/tag |
| `.mc-badge` | Compact notification badge |
| `.mc-progress` | Progress bar container |
| `.mc-progress-bar` | Progress bar fill (inner) |
| `.mc-sparkline` | Inline SVG sparkline chart |
| `.mc-sparkline-path` | Sparkline stroke path |
| `.mc-sparkline-fill` | Sparkline area fill |
| `.mc-has-popover` | Element with hover tooltip |

## Button Styles

| Class | Appearance |
|-------|-----------|
| `.mc-primary` | Blue filled |
| `.mc-secondary` | Gray filled |
| `.mc-danger` | Red filled |
| `.mc-warning` | Yellow filled |
| `.mc-info` | Cyan text |
| `.mc-outline` | Transparent with accent border |
| `.mc-ghost` | Transparent, no border |

## Column Grid

| Class | Width |
|-------|-------|
| `.mc-col-1` through `.mc-col-12` | N/12 of row width |

Responsive variants: `.mc-sm-col-N`, `.mc-md-col-N`, `.mc-lg-col-N`, `.mc-xl-col-N`

Breakpoints: `sm` 576px, `md` 768px, `lg` 992px, `xl` 1200px.

## Animations

| Class | Effect |
|-------|--------|
| `.mc-animate-fade` | Fade in |
| `.mc-animate-slide` | Slide from left |
| `.mc-animate-slide-up` | Slide from bottom |
| `.mc-animate-scale` | Scale in |
| `.mc-animate-bounce` | Elastic bounce |
| `.mc-animate-pulse` | Continuous pulse |
| `.mc-animate-shake` | Horizontal shake |
| `.mc-animate-glow` | Continuous glow |

## Validation

| Class | Description |
|-------|-------------|
| `.mc-invalid` | Red border + shadow |
| `.mc-valid` | Green border |
| `.mc-error` | Error message text |

## Parametric

| Class | Description |
|-------|-------------|
| `.mc-parametric` | Constraint-based layout container (`position: relative`) |
| `[data-parametric]` | Solved element wrapper (`position: absolute`) |

## Editor

| Class | Description |
|-------|-------------|
| `.mc-editor` | Editor container |
| `.mc-editor-active` | Editor initialized |
| `.mc-editor-toolbar` | Toolbar row |
| `.mc-editor-btn` | Toolbar button |
| `.mc-editor-btn-active` | Active format state |
| `.mc-editor-content` | Contenteditable area |

## Routing

| Class | Description |
|-------|-------------|
| `.route-active` | Applied to active route's nav element |

## Size Scale

`.mc-size-1` (10px) through `.mc-size-9` (48px).

## Custom Properties

See [Theming](/v1.0/guide/theming) for the full list of `--mc-*` CSS variables.

## Data Attributes

| Attribute | Set By | Used By |
|-----------|--------|---------|
| `data-line` | Renderer | Row identification |
| `data-bind` | Renderer | State key binding |
| `data-action` | Renderer | Button action name |
| `data-validate` | Renderer | Validation rules |
| `data-route` | Renderer | Route path |
| `data-fetch` | Renderer | Fetch URL |
| `data-logic` | Renderer | Logic function ref |
| `data-config` | Renderer | Container config |
| `data-scope` | Renderer | List item scope |
| `data-span` | Renderer | Component character width |
| `data-spacer` | Renderer | Spacer mode (end, evenly, col-N-end) |
| `data-popover` | Renderer | Popover tooltip text |
| `data-value` | Renderer | Progress bar percentage |
| `data-theme` | JS API | Active theme name |
| `data-editor` | Renderer | Editor container marker |
| `data-features` | Renderer | Comma-separated editor features |
| `data-parametric` | Renderer | Parametric element name |
