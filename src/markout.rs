use crate::component;
use crate::document::Document;
use crate::line::Line;

pub fn parse(input: &str) -> Document {
    // Pass 1: collect @define blocks
    let mut templates: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut defining: Option<(String, Vec<String>)> = None;
    let mut raw_lines: Vec<String> = Vec::new();

    for raw in input.lines() {
        let trimmed = raw.trim();

        if let Some((ref name, ref mut body)) = defining {
            if trimmed == "@end define" || trimmed == &format!("@end define {}", name) {
                templates.insert(name.clone(), body.clone());
                defining = None;
            } else {
                body.push(raw.to_string());
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@define ") {
            defining = Some((rest.trim().to_string(), Vec::new()));
            continue;
        }

        raw_lines.push(raw.to_string());
    }

    // Pass 2: expand @use and parse normally
    let mut expanded = Vec::new();
    let mut use_counter = 0usize;

    for raw in &raw_lines {
        let trimmed = raw.trim();

        if let Some(rest) = trimmed.strip_prefix("@use ") {
            let (name, params) = parse_use_directive(rest);
            if let Some(template) = templates.get(&name) {
                let scope = params.get("scope")
                    .cloned()
                    .unwrap_or_else(|| format!("{}_{}", name, use_counter));
                use_counter += 1;

                for tpl_line in template {
                    let scoped = scope_markout_line(tpl_line, &scope);
                    expanded.push(scoped);
                }
            }
            continue;
        }

        expanded.push(raw.clone());
    }

    // Pass 3: parse expanded lines
    let mut lines: Vec<Line> = Vec::new();

    for raw in &expanded {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "@end each" {
            lines.push(Line::each_end());
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@end ") {
            let tag = rest.trim();
            lines.push(Line::container_end(tag));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@each ") {
            lines.push(Line::each_start(rest.trim()));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('@') {
            let (tag, config) = parse_container_directive(rest);
            let cfg = if config.is_empty() { None } else { Some(config.as_str()) };
            lines.push(Line::container_start(&tag, cfg));
            continue;
        }

        let content_str = if let Some(rest) = trimmed.strip_prefix("| ") {
            rest
        } else if trimmed == "|" {
            ""
        } else {
            trimmed
        };

        lines.push(parse_content_line(content_str));
    }

    if lines.is_empty() {
        Document::new()
    } else {
        Document::from_lines(lines)
    }
}

fn parse_use_directive(rest: &str) -> (String, std::collections::HashMap<String, String>) {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let name = parts.first().map(|s| s.to_string()).unwrap_or_default();
    let mut params = std::collections::HashMap::new();

    for part in &parts[1..] {
        if let Some((k, v)) = part.split_once('=') {
            params.insert(k.to_string(), v.to_string());
        }
    }

    (name, params)
}

fn scope_markout_line(line: &str, scope: &str) -> String {
    // Replace {component:key with {component:scope.key in the line
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            result.push('{');
            i += 1;
            // Find the component:key part and scope the key
            let mut inner = String::new();
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '{' { depth += 1; }
                if chars[i] == '}' { depth -= 1; }
                if depth > 0 {
                    inner.push(chars[i]);
                }
                i += 1;
            }
            result.push_str(&scope_component_spec(&inner, scope));
            result.push('}');
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn scope_component_spec(spec: &str, scope: &str) -> String {
    let parts = shell_split(spec);
    if parts.is_empty() { return spec.to_string(); }

    let mut result_parts = Vec::new();

    for (idx, part) in parts.iter().enumerate() {
        if idx == 0 {
            // component:key → component:scope.key
            if let Some(colon) = part.find(':') {
                let comp = &part[..colon];
                let key = &part[colon + 1..];
                result_parts.push(format!("{}:{}.{}", comp, scope, key));
            } else {
                result_parts.push(part.clone());
            }
        } else {
            result_parts.push(part.clone());
        }
    }

    result_parts.join(" ")
}

pub fn emit(doc: &Document) -> String {
    let mut output = Vec::new();

    for line in &doc.lines {
        if line.is_each_start() {
            let key = line.meta.tag.as_deref().unwrap_or("items");
            output.push(format!("@each {}", key));
            continue;
        }

        if line.is_each_end() {
            output.push("@end each".to_string());
            continue;
        }

        if line.is_container_start() {
            let tag = line.meta.tag.as_deref().unwrap_or("div");
            if let Some(ref config) = line.meta.config {
                output.push(format!("@{} {}", tag, config));
            } else {
                output.push(format!("@{}", tag));
            }
            continue;
        }

        if line.is_container_end() {
            let tag = line.meta.tag.as_deref().unwrap_or("div");
            output.push(format!("@end {}", tag));
            continue;
        }

        let spans = line.spans();
        if spans.is_empty() {
            output.push("| ".to_string());
            continue;
        }

        let mut parts = Vec::new();
        for span in &spans {
            parts.push(span_to_markout(span, &line.logic));
        }

        output.push(format!("| {}", parts.join("  ")));
    }

    output.join("\n")
}

fn parse_container_directive(rest: &str) -> (String, String) {
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    let tag = parts[0].to_string();
    let config = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
    (tag, config)
}

fn parse_content_line(input: &str) -> Line {
    if input.is_empty() {
        return Line::new();
    }

    let tokens = tokenize(input);

    if tokens.is_empty() {
        return Line::label(input);
    }

    let mut content = String::new();
    let mut components = String::new();
    let mut state_keys = String::new();
    let mut styles = String::new();
    let mut logic: Option<std::collections::HashMap<usize, String>> = None;
    let mut cols: Option<std::collections::HashMap<usize, (u8, u8)>> = None;
    let mut validates: Option<std::collections::HashMap<usize, String>> = None;
    let mut responsives: Option<std::collections::HashMap<usize, Vec<(String, u8, u8)>>> = None;
    let mut animates: Option<std::collections::HashMap<usize, String>> = None;
    let mut popovers: Option<std::collections::HashMap<usize, String>> = None;
    let mut line_constraints: Option<std::collections::HashMap<usize, Vec<String>>> = None;

    for (ti, token) in tokens.iter().enumerate() {
        match token {
            Token::Text(text) => {
                let len = text.chars().count();
                content.push_str(text);
                components.extend(std::iter::repeat(component::LABEL).take(len));
                state_keys.extend(std::iter::repeat('_').take(len));
                styles.extend(std::iter::repeat(' ').take(len));

                // If next token is a {label:...} component, insert separator
                // so group_spans doesn't merge adjacent L spans
                if let Some(Token::Component { kind, .. }) = tokens.get(ti + 1) {
                    if kind == "label" {
                        content.push(' ');
                        components.push(component::EMPTY);
                        state_keys.push('_');
                        styles.push(' ');
                    }
                }
            }
            Token::Component { kind, binding, label, style, href, col, validate, responsive, animate, popover, constraints: comp_constraints } => {
                let (comp_char, computed_width) = component_char_and_width(
                    kind,
                    binding.as_deref(),
                    label.as_deref(),
                );
                let default_display = "_".repeat(computed_width);
                let display = label.as_deref().unwrap_or(&default_display);
                let len = display.chars().count().max(computed_width).max(1);
                let pos = content.chars().count();

                // Pad display to match len if binding is longer than label
                let display_len = display.chars().count();
                content.push_str(display);
                if display_len < len {
                    content.extend(std::iter::repeat(' ').take(len - display_len));
                }
                components.extend(std::iter::repeat(comp_char).take(len));

                if let Some(b) = binding {
                    let key_padded = pad_key(b, len);
                    state_keys.push_str(&key_padded);
                } else {
                    state_keys.extend(std::iter::repeat('_').take(len));
                }

                if let Some(s) = style {
                    let s_char = style_char(s);
                    styles.extend(std::iter::repeat(s_char).take(len));
                } else {
                    styles.extend(std::iter::repeat(' ').take(len));
                }

                if let Some(h) = href {
                    logic.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, h.clone());
                }

                if let Some(c) = col {
                    cols.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, *c);
                }

                if let Some(v) = validate {
                    validates.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, v.clone());
                }

                if let Some(ref r) = responsive {
                    responsives.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, r.clone());
                }

                if let Some(ref a) = animate {
                    animates.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, a.clone());
                }

                if let Some(ref p) = popover {
                    popovers.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, p.clone());
                }

                if let Some(ref cc) = comp_constraints {
                    line_constraints.get_or_insert_with(std::collections::HashMap::new)
                        .insert(pos, cc.clone());
                }
            }
            Token::Spacing(n) => {
                content.extend(std::iter::repeat(' ').take(*n));
                components.extend(std::iter::repeat(component::EMPTY).take(*n));
                state_keys.extend(std::iter::repeat('_').take(*n));
                styles.extend(std::iter::repeat(' ').take(*n));
            }
        }
    }

    let mut line = Line::content_row(&content, &components, &state_keys, &styles);
    line.logic = logic;
    line.cols = cols;
    line.validates = validates;
    line.responsives = responsives;
    line.animates = animates;
    line.popovers = popovers;
    line.constraints = line_constraints;
    line
}

#[derive(Debug, PartialEq)]
enum Token {
    Text(String),
    Component {
        kind: String,
        binding: Option<String>,
        label: Option<String>,
        style: Option<String>,
        href: Option<String>,
        col: Option<(u8, u8)>,
        validate: Option<String>,
        responsive: Option<Vec<(String, u8, u8)>>,
        animate: Option<String>,
        popover: Option<String>,
        constraints: Option<Vec<String>>,
    },
    Spacing(usize),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            if let Some((token, end)) = parse_component_token(&chars, i) {
                tokens.push(token);
                i = end;
                continue;
            }
        }

        let start = i;
        while i < chars.len() && chars[i] != '{' {
            i += 1;
        }

        if i > start {
            let text: String = chars[start..i].iter().collect();

            // Check if it's pure spacing between components
            let is_spacing = text.chars().all(|c| c == ' ')
                && !tokens.is_empty()
                && start > 0;

            if is_spacing && i < chars.len() && chars[i] == '{' {
                tokens.push(Token::Spacing(text.len()));
            } else {
                tokens.push(Token::Text(text));
            }
        }
    }

    tokens
}

fn parse_component_token(chars: &[char], start: usize) -> Option<(Token, usize)> {
    let mut i = start + 1; // skip '{'
    let mut depth = 1;

    while i < chars.len() && depth > 0 {
        if chars[i] == '{' { depth += 1; }
        if chars[i] == '}' { depth -= 1; }
        i += 1;
    }

    if depth != 0 {
        return None;
    }

    let inner: String = chars[start + 1..i - 1].iter().collect();
    let token = parse_component_spec(&inner)?;
    Some((token, i))
}

fn parse_component_spec(spec: &str) -> Option<Token> {
    let parts = shell_split(spec);
    if parts.is_empty() {
        return None;
    }

    let first = &parts[0];
    let (kind, binding) = if let Some(idx) = first.find(':') {
        let k = first[..idx].to_string();
        let b = first[idx + 1..].to_string();
        (k, if b.is_empty() { None } else { Some(b) })
    } else {
        (first.clone(), None)
    };

    let mut label = None;
    let mut style = None;
    let mut href = None;
    let mut col = None;
    let mut route = None;
    let mut validate = None;
    let mut fetch = None;
    let mut responsive_cols: Vec<(String, u8, u8)> = Vec::new();
    let mut animate = None;
    let mut popover = None;
    let mut constraints: Vec<String> = Vec::new();

    for part in &parts[1..] {
        if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
            label = Some(part[1..part.len() - 1].to_string());
        } else if part.starts_with("href=") {
            href = Some(part[5..].to_string());
        } else if part.starts_with("route:") {
            route = Some(part[6..].to_string());
        } else if part.starts_with("validate:") {
            validate = Some(part[9..].to_string());
        } else if part.starts_with("fetch:") {
            fetch = Some(part[6..].to_string());
        } else if part.starts_with("animate:") {
            animate = Some(part[8..].to_string());
        } else if part.starts_with("popover:") {
            popover = Some(part[8..].trim_matches('"').to_string());
        } else if crate::constraint::is_constraint_token(part) {
            constraints.push(part.clone());
        } else if part.starts_with("col-") || part.starts_with("sm:") || part.starts_with("md:") || part.starts_with("lg:") || part.starts_with("xl:") {
            if let Some(bp_col) = parse_responsive_col(part) {
                responsive_cols.push(bp_col);
            } else if part.starts_with("col-") {
                col = crate::component::parse_col(part);
            }
        } else {
            style = Some(part.clone());
        }
    }

    let resp = if responsive_cols.is_empty() { None } else { Some(responsive_cols) };

    Some(Token::Component {
        kind,
        binding,
        label,
        style,
        href: href
            .or(route.map(|r| format!("route:{}", r)))
            .or(fetch.map(|f| format!("fetch:{}", f))),
        col,
        validate,
        responsive: resp,
        animate,
        popover,
        constraints: if constraints.is_empty() { None } else { Some(constraints) },
    })
}

fn parse_responsive_col(part: &str) -> Option<(String, u8, u8)> {
    // Parses "sm:col-6", "md:col-4", "lg:col-3[5]", etc.
    let colon = part.find(':')?;
    let breakpoint = part[..colon].to_string();
    let col_part = &part[colon + 1..];
    let (n, total) = crate::component::parse_col(col_part)?;
    Some((breakpoint, n, total))
}

fn shell_split(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '"' {
            in_quotes = !in_quotes;
            current.push('"');
        } else if chars[i] == ' ' && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(chars[i]);
        }
        i += 1;
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn component_char_and_width(kind: &str, binding: Option<&str>, label: Option<&str>) -> (char, usize) {
    let base_char = match kind {
        "input" => component::TEXT_INPUT,
        "password" => component::PASSWORD_INPUT,
        "button" => component::BUTTON,
        "checkbox" => component::CHECKBOX,
        "radio" => component::RADIO,
        "select" => component::SELECT,
        "textarea" => component::TEXTAREA,
        "image" | "img" => component::IMAGE,
        "link" => component::LINK,
        "label" => component::LABEL,
        "divider" | "hr" => component::DIVIDER,
        "spacer" => component::SPACER,
        "pill" => component::PILL,
        "badge" => component::BADGE,
        "progress" => component::PROGRESS,
        "sparkline" => component::SPARKLINE,
        _ => component::CUSTOM,
    };

    if let Some(l) = label {
        let label_len = l.chars().count().max(1);
        let key_len = binding.map(|b| b.chars().count()).unwrap_or(0);
        return (base_char, label_len.max(key_len));
    }

    let default_width = match kind {
        "input" => 16,
        "password" => 16,
        "button" => 8,
        "select" => 12,
        "textarea" => 24,
        "image" | "img" => 8,
        "link" => 8,
        "label" => 8,
        "divider" | "hr" => 8,
        "pill" => 8,
        "badge" => 4,
        "progress" => 12,
        "sparkline" => 16,
        _ => 8,
        // checkbox/radio/spacer get sized by binding key
    };

    let min_width = match kind {
        "checkbox" | "radio" | "spacer" => 1,
        _ => default_width,
    };

    let key_len = binding.map(|b| b.chars().count()).unwrap_or(0);
    (base_char, key_len.max(min_width))
}

fn pad_key(key: &str, target_len: usize) -> String {
    let key_len = key.chars().count();
    if key_len >= target_len {
        key.chars().take(target_len).collect()
    } else {
        let mut result = key.to_string();
        result.extend(std::iter::repeat('_').take(target_len - key_len));
        result
    }
}

fn style_char(style_name: &str) -> char {
    match style_name {
        "primary" => 'p',
        "secondary" => 's',
        "danger" => 'd',
        "warning" => 'w',
        "info" => 'i',
        "dark" => 'k',
        "light" => 'l',
        "outline" => 'o',
        "ghost" => 'g',
        _ => ' ',
    }
}

fn style_name(style_char: char) -> Option<&'static str> {
    match style_char {
        'p' => Some("primary"),
        's' => Some("secondary"),
        'd' => Some("danger"),
        'w' => Some("warning"),
        'i' => Some("info"),
        'k' => Some("dark"),
        'l' => Some("light"),
        'o' => Some("outline"),
        'g' => Some("ghost"),
        _ => None,
    }
}

fn span_to_markout(
    span: &crate::component::ComponentSpan,
    logic: &Option<std::collections::HashMap<usize, String>>,
) -> String {
    let kind = match span.component {
        component::LABEL => {
            if span.state_key.is_some() {
                let binding = span.state_key.as_deref().unwrap_or("");
                let mut extras = Vec::new();
                if let Some(ref a) = span.animate {
                    extras.push(format!("animate:{}", a));
                }
                if let Some((n, total)) = span.col {
                    if total == 12 { extras.push(format!("col-{}", n)); }
                    else { extras.push(format!("col-{}[{}]", n, total)); }
                }
                if let Some(ref resp) = span.responsive {
                    for (bp, n, total) in resp {
                        if *total == 12 { extras.push(format!("{}:col-{}", bp, n)); }
                        else { extras.push(format!("{}:col-{}[{}]", bp, n, total)); }
                    }
                }
                if extras.is_empty() {
                    return format!("{{label:{}}}", binding);
                } else {
                    return format!("{{label:{} {}}}", binding, extras.join(" "));
                }
            } else {
                return span.content.clone();
            }
        }
        component::SPACER => {
            let mode = span.state_key.as_deref().unwrap_or("end");
            return format!("{{spacer:{}}}", mode);
        }
        component::PILL => "pill",
        component::BADGE => "badge",
        component::PROGRESS => "progress",
        component::SPARKLINE => "sparkline",
        component::TEXT_INPUT => "input",
        component::PASSWORD_INPUT => "password",
        component::BUTTON => "button",
        component::CHECKBOX => "checkbox",
        component::RADIO => "radio",
        component::SELECT => "select",
        component::TEXTAREA => "textarea",
        component::IMAGE => "image",
        component::LINK => "link",
        component::DIVIDER => "divider",
        component::CUSTOM => "custom",
        _ => return span.content.clone(),
    };

    let mut parts = Vec::new();

    let mut type_binding = kind.to_string();
    if let Some(ref key) = span.state_key {
        type_binding.push(':');
        type_binding.push_str(key);
    }
    parts.push(type_binding);

    let content_trimmed = span.content.trim().trim_matches('_');
    if !content_trimmed.is_empty() && span.component == component::BUTTON {
        parts.push(format!("\"{}\"", content_trimmed));
    }

    if let Some(s) = span.style {
        if let Some(name) = style_name(s) {
            parts.push(name.to_string());
        }
    }

    if let Some(ref href) = logic.as_ref().and_then(|l| l.get(&span.start)) {
        if let Some(route_path) = href.strip_prefix("route:") {
            parts.push(format!("route:{}", route_path));
        } else if let Some(fetch_url) = href.strip_prefix("fetch:") {
            parts.push(format!("fetch:{}", fetch_url));
        } else {
            parts.push(format!("href={}", href));
        }
    }

    if let Some(ref v) = span.validate {
        parts.push(format!("validate:{}", v));
    }

    if let Some(ref p) = span.popover {
        parts.push(format!("popover:\"{}\"", p));
    }

    if let Some((n, total)) = span.col {
        if total == 12 {
            parts.push(format!("col-{}", n));
        } else {
            parts.push(format!("col-{}[{}]", n, total));
        }
    }

    if let Some(ref a) = span.animate {
        parts.push(format!("animate:{}", a));
    }

    if let Some(ref resp) = span.responsive {
        for (bp, n, total) in resp {
            if *total == 12 {
                parts.push(format!("{}:col-{}", bp, n));
            } else {
                parts.push(format!("{}:col-{}[{}]", bp, n, total));
            }
        }
    }

    if let Some(ref cc) = span.constraints {
        for c in cc {
            parts.push(c.clone());
        }
    }

    format!("{{{}}}", parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component;

    #[test]
    fn test_parse_empty() {
        let doc = parse("");
        assert_eq!(doc.line_count(), 1);
    }

    #[test]
    fn test_parse_label() {
        let doc = parse("| Hello World");
        assert_eq!(doc.lines[0].content, "Hello World");
        assert!(doc.lines[0].components.chars().all(|c| c == component::LABEL));
    }

    #[test]
    fn test_parse_input() {
        let doc = parse("| {input:username}");
        let spans = doc.lines[0].spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].component, component::TEXT_INPUT);
        assert_eq!(spans[0].state_key, Some("username".to_string()));
    }

    #[test]
    fn test_parse_button_with_label() {
        let doc = parse(r#"| {button:submit "Sign In" primary}"#);
        let spans = doc.lines[0].spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].component, component::BUTTON);
        assert_eq!(spans[0].state_key, Some("submit".to_string()));
        assert_eq!(spans[0].content.trim(), "Sign In");
        assert_eq!(spans[0].style, Some('p'));
    }

    #[test]
    fn test_parse_checkbox() {
        let doc = parse("| {checkbox:agree}");
        let spans = doc.lines[0].spans();
        assert_eq!(spans[0].component, component::CHECKBOX);
        assert_eq!(spans[0].state_key, Some("agree".to_string()));
    }

    #[test]
    fn test_parse_password() {
        let doc = parse("| {password:pass}");
        let spans = doc.lines[0].spans();
        assert_eq!(spans[0].component, component::PASSWORD_INPUT);
        assert_eq!(spans[0].state_key, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_mixed_line() {
        let doc = parse("| Username  {input:username}");
        let spans = doc.lines[0].spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].component, component::LABEL);
        assert_eq!(spans[0].content, "Username  ");
        assert_eq!(spans[1].component, component::TEXT_INPUT);
        assert_eq!(spans[1].state_key, Some("username".to_string()));
    }

    #[test]
    fn test_parse_container() {
        let doc = parse("@card shadow:md\n| Hello\n@end card");
        assert_eq!(doc.line_count(), 3);
        assert!(doc.lines[0].is_container_start());
        assert_eq!(doc.lines[0].meta.tag, Some("card".to_string()));
        assert_eq!(doc.lines[0].meta.config, Some("shadow:md".to_string()));
        assert_eq!(doc.lines[1].content, "Hello");
        assert!(doc.lines[2].is_container_end());
    }

    #[test]
    fn test_parse_container_no_config() {
        let doc = parse("@card\n| Inside\n@end card");
        assert!(doc.lines[0].is_container_start());
        assert_eq!(doc.lines[0].meta.config, None);
    }

    #[test]
    fn test_parse_counter_app() {
        let doc = parse(
            r#"| {button:decrement "-" outline}  {label:count}  {button:increment "+" outline}"#,
        );
        let spans = doc.lines[0].spans();
        assert_eq!(spans.len(), 5);

        assert_eq!(spans[0].component, component::BUTTON);
        assert_eq!(spans[0].state_key, Some("decrement".to_string()));
        assert_eq!(spans[0].content.trim(), "-");

        assert_eq!(spans[2].component, component::LABEL);
        assert_eq!(spans[2].state_key, Some("count".to_string()));

        assert_eq!(spans[4].component, component::BUTTON);
        assert_eq!(spans[4].state_key, Some("increment".to_string()));
        assert_eq!(spans[4].content.trim(), "+");
    }

    #[test]
    fn test_parse_login_form() {
        let input = r#"@card shadow:md padding:24
| Login
| Username  {input:username}
| Password  {password:password}
| {checkbox:remember} Remember me
| {button:submit "Sign In" primary}
| Status: {label:status}
@end card"#;

        let doc = parse(input);
        assert!(doc.lines[0].is_container_start());

        let login_label = &doc.lines[1];
        assert_eq!(login_label.content, "Login");

        let username_spans = doc.lines[2].spans();
        assert_eq!(username_spans[0].component, component::LABEL);
        assert_eq!(username_spans[1].component, component::TEXT_INPUT);

        let password_spans = doc.lines[3].spans();
        assert_eq!(password_spans[1].component, component::PASSWORD_INPUT);

        let checkbox_spans = doc.lines[4].spans();
        assert_eq!(checkbox_spans[0].component, component::CHECKBOX);

        let button_spans = doc.lines[5].spans();
        assert_eq!(button_spans[0].component, component::BUTTON);
        assert_eq!(button_spans[0].style, Some('p'));

        assert!(doc.lines[7].is_container_end());
    }

    #[test]
    fn test_parse_nested_containers() {
        let input = "@page\n@sidebar\n| Nav\n@end sidebar\n@main\n| Content\n@end main\n@end page";
        let doc = parse(input);
        assert_eq!(doc.line_count(), 8);
        assert!(doc.lines[0].is_container_start());
        assert!(doc.lines[1].is_container_start());
        assert!(doc.lines[3].is_container_end());
        assert!(doc.lines[7].is_container_end());
    }

    #[test]
    fn test_parse_reactive_label() {
        let doc = parse("| Count: {label:count}");
        let spans = doc.lines[0].spans();
        // Text "Count: " + separator space + bound label
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].component, component::LABEL);
        assert_eq!(spans[0].content, "Count: ");
        assert!(spans[0].state_key.is_none());
        assert_eq!(spans[1].component, component::EMPTY);
        assert_eq!(spans[2].component, component::LABEL);
        assert_eq!(spans[2].state_key, Some("count".to_string()));
    }

    #[test]
    fn test_parse_outline_style() {
        let doc = parse(r#"| {button:go "Go" outline}"#);
        let spans = doc.lines[0].spans();
        assert_eq!(spans[0].style, Some('o'));
    }

    #[test]
    fn test_parse_danger_style() {
        let doc = parse(r#"| {button:del "Delete" danger}"#);
        let spans = doc.lines[0].spans();
        assert_eq!(spans[0].style, Some('d'));
    }

    // === Emit tests ===

    #[test]
    fn test_emit_label() {
        let doc = Document::from_lines(vec![Line::label("Hello")]);
        let output = emit(&doc);
        assert_eq!(output, "| Hello");
    }

    #[test]
    fn test_emit_container() {
        let doc = Document::from_lines(vec![
            Line::container_start("card", Some("shadow:md")),
            Line::label("Inside"),
            Line::container_end("card"),
        ]);
        let output = emit(&doc);
        assert!(output.contains("@card shadow:md"));
        assert!(output.contains("| Inside"));
        assert!(output.contains("@end card"));
    }

    #[test]
    fn test_emit_button() {
        let doc = Document::from_lines(vec![
            Line::content_row("Submit", "BBBBBB", "submit", "pppppp"),
        ]);
        let output = emit(&doc);
        assert!(output.contains("{button:submit"));
        assert!(output.contains("primary"));
    }

    #[test]
    fn test_emit_input() {
        let doc = Document::from_lines(vec![
            Line::content_row("________________", "IIIIIIIIIIIIIIII", "username________", "                "),
        ]);
        let output = emit(&doc);
        assert!(output.contains("{input:username}"));
    }

    // === Round-trip tests ===

    #[test]
    fn test_roundtrip_counter() {
        let input = r#"| {button:decrement "-" outline}  {label:count}  {button:increment "+" outline}"#;
        let doc = parse(input);

        let spans = doc.lines[0].spans();
        assert!(spans.iter().any(|s| s.component == component::BUTTON && s.state_key == Some("decrement".to_string())));
        assert!(spans.iter().any(|s| s.component == component::LABEL && s.state_key == Some("count".to_string())));
        assert!(spans.iter().any(|s| s.component == component::BUTTON && s.state_key == Some("increment".to_string())));
    }

    #[test]
    fn test_roundtrip_container() {
        let input = "@card shadow:md\n| Hello\n@end card";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("@card shadow:md"));
        assert!(output.contains("@end card"));
    }

    // === Col tests ===

    #[test]
    fn test_parse_col_12_grid() {
        let doc = parse("| {input:name col-8}  {button:go col-4}");
        let spans = doc.lines[0].spans();
        let input_span = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input_span.col, Some((8, 12)));
        let btn_span = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn_span.col, Some((4, 12)));
    }

    #[test]
    fn test_parse_col_custom_denominator() {
        let doc = parse("| {input:name col-3[5]}");
        let spans = doc.lines[0].spans();
        let input_span = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input_span.col, Some((3, 5)));
    }

    #[test]
    fn test_parse_col_with_style() {
        let doc = parse(r#"| {button:go "Go" primary col-6}"#);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn.col, Some((6, 12)));
        assert_eq!(btn.style, Some('p'));
    }

    #[test]
    fn test_emit_col_12() {
        let input = "| {input:name col-8}  {button:go col-4}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("col-8"));
        assert!(output.contains("col-4"));
    }

    #[test]
    fn test_emit_col_custom() {
        let input = "| {input:name col-3[5]}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("col-3[5]"));
    }

    #[test]
    fn test_parse_col_utility() {
        assert_eq!(component::parse_col("col-6"), Some((6, 12)));
        assert_eq!(component::parse_col("col-3[5]"), Some((3, 5)));
        assert_eq!(component::parse_col("col-12"), Some((12, 12)));
        assert_eq!(component::parse_col("col-0"), None);
        assert_eq!(component::parse_col("col-13"), None);
        assert_eq!(component::parse_col("col-4[3]"), None);
        assert_eq!(component::parse_col("notcol"), None);
    }

    // === Route tests ===

    #[test]
    fn test_parse_route() {
        let doc = parse(r#"| {button:nav "Dashboard" ghost route:/dashboard}"#);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn.logic_ref, Some("route:/dashboard".to_string()));
    }

    #[test]
    fn test_emit_route() {
        let input = r#"| {button:nav "Dashboard" ghost route:/dashboard}"#;
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("route:/dashboard"));
    }

    #[test]
    fn test_route_with_col() {
        let doc = parse(r#"| {button:nav "Home" primary route:/home col-6}"#);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn.logic_ref, Some("route:/home".to_string()));
        assert_eq!(btn.col, Some((6, 12)));
        assert_eq!(btn.style, Some('p'));
    }

    // === Spacer tests ===

    #[test]
    fn test_parse_spacer_end() {
        let doc = parse("| Logo  {spacer:end}  {button:login \"Sign In\" primary}");
        let spans = doc.lines[0].spans();
        let spacer = spans.iter().find(|s| s.component == component::SPACER).unwrap();
        assert_eq!(spacer.state_key, Some("end".to_string()));
    }

    #[test]
    fn test_parse_spacer_evenly() {
        let doc = parse("| {button:a \"A\"}  {spacer:evenly}  {button:b \"B\"}  {spacer:evenly}  {button:c \"C\"}");
        let spans = doc.lines[0].spans();
        let spacers: Vec<_> = spans.iter().filter(|s| s.component == component::SPACER).collect();
        assert_eq!(spacers.len(), 2);
        assert_eq!(spacers[0].state_key, Some("evenly".to_string()));
    }

    #[test]
    fn test_parse_spacer_col() {
        let doc = parse("| {label:id col-1}  {spacer:col-4-end}  {label:name col-4}");
        let spans = doc.lines[0].spans();
        let spacer = spans.iter().find(|s| s.component == component::SPACER).unwrap();
        assert_eq!(spacer.state_key, Some("col-4-end".to_string()));
    }

    #[test]
    fn test_emit_spacer() {
        let input = "| Logo  {spacer:end}  {button:go \"Go\" primary}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("{spacer:end}"));
    }

    #[test]
    fn test_spacer_render() {
        use crate::state::StateStore;
        use crate::vdom::render::render_document;

        let doc = parse("| {label:a}  {spacer:evenly}  {label:b}");
        let state = StateStore::new();
        let vdom = render_document(&doc.lines, &state);

        let row = &vdom.children()[0];
        // Should have label, spacer separator, spacer, spacer separator, label = 5 spans
        // (separators are EMPTY between label and spacer)
        let children = row.children();
        let spacer = children.iter().find(|c| {
            if let crate::vdom::node::VNode::Element(el) = c {
                el.attrs.get("data-spacer").is_some()
            } else {
                false
            }
        });
        assert!(spacer.is_some());
    }

    // === Animate tests ===

    #[test]
    fn test_parse_animate() {
        let doc = parse("| {label:status animate:fade}");
        let spans = doc.lines[0].spans();
        let label = spans.iter().find(|s| s.component == component::LABEL && s.state_key == Some("status".to_string())).unwrap();
        assert_eq!(label.animate, Some("fade".to_string()));
    }

    #[test]
    fn test_parse_animate_with_style() {
        let doc = parse(r#"| {button:go "Go" primary animate:bounce}"#);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn.animate, Some("bounce".to_string()));
        assert_eq!(btn.style, Some('p'));
    }

    #[test]
    fn test_emit_animate() {
        let input = "| {label:status animate:slide}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("animate:slide"));
    }

    // === Responsive tests ===

    #[test]
    fn test_parse_responsive_cols() {
        let doc = parse("| {input:name col-12 md:col-8 lg:col-6}");
        let spans = doc.lines[0].spans();
        let input = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input.col, Some((12, 12)));
        let resp = input.responsive.as_ref().unwrap();
        assert_eq!(resp.len(), 2);
        assert!(resp.contains(&("md".to_string(), 8, 12)));
        assert!(resp.contains(&("lg".to_string(), 6, 12)));
    }

    #[test]
    fn test_parse_responsive_custom_denominator() {
        let doc = parse("| {input:name col-5[5] md:col-3[5]}");
        let spans = doc.lines[0].spans();
        let input = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input.col, Some((5, 5)));
        let resp = input.responsive.as_ref().unwrap();
        assert_eq!(resp[0], ("md".to_string(), 3, 5));
    }

    #[test]
    fn test_emit_responsive() {
        let input = "| {input:name col-12 md:col-6 lg:col-4}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("col-12"));
        assert!(output.contains("md:col-6"));
        assert!(output.contains("lg:col-4"));
    }

    // === Fetch tests ===

    #[test]
    fn test_parse_fetch() {
        let doc = parse(r#"| {button:load "Load" primary fetch:/api/users}"#);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert_eq!(btn.logic_ref, Some("fetch:/api/users".to_string()));
    }

    #[test]
    fn test_emit_fetch() {
        let input = r#"| {button:load "Load" primary fetch:/api/users}"#;
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("fetch:/api/users"));
    }

    // === Validate tests ===

    #[test]
    fn test_parse_validate() {
        let doc = parse("| {input:email validate:required,email}");
        let spans = doc.lines[0].spans();
        let input = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input.validate, Some("required,email".to_string()));
    }

    #[test]
    fn test_parse_validate_with_style() {
        let doc = parse("| {input:name primary validate:required,min:3}");
        let spans = doc.lines[0].spans();
        let input = spans.iter().find(|s| s.component == component::TEXT_INPUT).unwrap();
        assert_eq!(input.validate, Some("required,min:3".to_string()));
        assert_eq!(input.style, Some('p'));
    }

    #[test]
    fn test_emit_validate() {
        let input = "| {input:email validate:required,email}";
        let doc = parse(input);
        let output = emit(&doc);
        assert!(output.contains("validate:required,email"));
    }

    // === Each tests ===

    #[test]
    fn test_parse_each() {
        let doc = parse("@each todos\n| {checkbox:done} {label:text}\n@end each");
        assert_eq!(doc.line_count(), 3);
        assert!(doc.lines[0].is_each_start());
        assert_eq!(doc.lines[0].meta.tag, Some("todos".to_string()));
        assert!(doc.lines[2].is_each_end());
    }

    #[test]
    fn test_emit_each() {
        let doc = parse("@each items\n| {label:name}\n@end each");
        let output = emit(&doc);
        assert!(output.contains("@each items"));
        assert!(output.contains("@end each"));
    }

    #[test]
    fn test_each_render_expands() {
        use crate::state::StateStore;
        use crate::vdom::render::render_document;

        let doc = parse("@each todos\n| {label:text}\n@end each");
        let mut state = StateStore::new();
        state.set_list_item("todos", 0, &[
            ("text".to_string(), crate::state::StateValue::Text("Buy milk".to_string())),
        ]);
        state.set_list_item("todos", 1, &[
            ("text".to_string(), crate::state::StateValue::Text("Write code".to_string())),
        ]);

        let vdom = render_document(&doc.lines, &state);
        // Should have 2 rows (one per item)
        assert_eq!(vdom.children().len(), 2);
    }

    #[test]
    fn test_each_render_empty_list() {
        use crate::state::StateStore;
        use crate::vdom::render::render_document;

        let doc = parse("@each todos\n| {label:text}\n@end each");
        let state = StateStore::new();
        let vdom = render_document(&doc.lines, &state);
        assert_eq!(vdom.children().len(), 0);
    }

    // === Template/Slot tests ===

    #[test]
    fn test_define_and_use() {
        let input = "@define greeting\n| Hello, {label:name}!\n@end define\n\n@use greeting scope=alice\n@use greeting scope=bob";
        let doc = parse(input);
        assert_eq!(doc.line_count(), 2);

        let spans_0 = doc.lines[0].spans();
        let label_0 = spans_0.iter().find(|s| s.state_key.is_some()).unwrap();
        assert!(label_0.state_key.as_ref().unwrap().contains("alice"));

        let spans_1 = doc.lines[1].spans();
        let label_1 = spans_1.iter().find(|s| s.state_key.is_some()).unwrap();
        assert!(label_1.state_key.as_ref().unwrap().contains("bob"));
    }

    #[test]
    fn test_define_multiline() {
        let input = "@define card_tpl\n@card padding:16\n| {label:title}\n| {input:value}\n@end card\n@end define\n\n@use card_tpl scope=a\n@use card_tpl scope=b";
        let doc = parse(input);
        // Each use: container_start + label_row + input_row + container_end = 4 lines
        // Two uses = 8 lines
        assert_eq!(doc.line_count(), 8);
    }

    #[test]
    fn test_define_auto_scope() {
        let input = "@define widget\n| {label:val}\n@end define\n\n@use widget\n@use widget";
        let doc = parse(input);
        assert_eq!(doc.line_count(), 2);

        let key_0 = doc.lines[0].spans().iter()
            .find(|s| s.state_key.is_some())
            .and_then(|s| s.state_key.clone()).unwrap();
        let key_1 = doc.lines[1].spans().iter()
            .find(|s| s.state_key.is_some())
            .and_then(|s| s.state_key.clone()).unwrap();

        // Auto-scoped keys should be different
        assert_ne!(key_0, key_1);
    }

    #[test]
    fn test_define_with_button() {
        let input = "@define action_row\n| {button:go \"Click\" primary}\n@end define\n\n@use action_row scope=form1";
        let doc = parse(input);
        let spans = doc.lines[0].spans();
        let btn = spans.iter().find(|s| s.component == component::BUTTON).unwrap();
        assert!(btn.state_key.as_ref().unwrap().starts_with("form1."));
    }

    #[test]
    fn test_define_not_in_output() {
        let input = "@define hidden\n| Secret\n@end define\n\n| Visible";
        let doc = parse(input);
        assert_eq!(doc.line_count(), 1);
        assert_eq!(doc.lines[0].content, "Visible");
    }
}
