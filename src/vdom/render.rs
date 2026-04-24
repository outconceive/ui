use std::collections::HashMap;
use super::node::VNode;
use crate::component::{self, ComponentSpan};
use crate::style;
use crate::block;
use crate::line::Line;
use crate::state::StateStore;
use crate::parametric;

pub fn render_document(lines: &[Line], state: &StateStore) -> VNode {
    let mut root_children = Vec::new();
    let mut container_stack: Vec<(String, HashMap<String, String>, Vec<VNode>)> = Vec::new();
    let mut each_stack: Vec<(String, Vec<Line>)> = Vec::new();
    let mut parametric_stack: Vec<(HashMap<String, String>, Vec<Line>)> = Vec::new();
    let mut line_counter = 0usize;

    for (raw_index, line) in lines.iter().enumerate() {
        // Collecting template lines inside @each
        if !each_stack.is_empty() {
            if line.is_each_end() {
                let (list_key, template_lines) = each_stack.pop().unwrap();
                let count = state.get_list_count(&list_key);
                for item_idx in 0..count {
                    let scope = format!("{}.{}", list_key, item_idx);
                    for tpl_line in &template_lines {
                        let row_vnode = render_line_scoped(tpl_line, line_counter, state, &scope, &list_key, item_idx);
                        if let Some(parent) = container_stack.last_mut() {
                            parent.2.push(row_vnode);
                        } else {
                            root_children.push(row_vnode);
                        }
                        line_counter += 1;
                    }
                }
            } else if line.is_each_start() {
                let key = line.meta.tag.as_deref().unwrap_or("").to_string();
                each_stack.push((key, Vec::new()));
            } else {
                each_stack.last_mut().unwrap().1.push(line.clone());
            }
            continue;
        }

        // Collecting lines inside @parametric
        if !parametric_stack.is_empty() {
            if line.is_container_end() {
                if let Some(tag) = &line.meta.tag {
                    if tag == "parametric" {
                        let (attrs, collected_lines) = parametric_stack.pop().unwrap();
                        let layout = parametric::solve_layout(&collected_lines);

                        let container_style = format!(
                            "position:relative;width:{:.1}px;height:{:.1}px",
                            layout.container_width, layout.container_height
                        );
                        let mut container_attrs = attrs;
                        container_attrs.insert("style".to_string(), container_style);

                        let mut children = Vec::new();
                        for (el_name, rect) in &layout.elements {
                            let span_line = collected_lines.iter()
                                .find(|l| l.spans().iter().any(|s| s.state_key.as_deref() == Some(el_name) || format!("_anon_{}", 0) == *el_name));

                            if let Some(sl) = span_line {
                                let spans = sl.spans();
                                if let Some(span) = spans.iter().find(|s| s.state_key.as_deref() == Some(el_name.as_str())) {
                                    let mut wrapper_attrs = HashMap::new();
                                    wrapper_attrs.insert("style".to_string(), format!(
                                        "position:absolute;left:{:.1}px;top:{:.1}px;width:{:.1}px;height:{:.1}px",
                                        rect.x, rect.y, rect.width, rect.height
                                    ));
                                    wrapper_attrs.insert("data-parametric".to_string(), el_name.clone());

                                    let inner = render_span(span, state);
                                    children.push(VNode::element_with_attrs("div", wrapper_attrs, vec![inner]));
                                }
                            }
                        }

                        let container = VNode::element_with_attrs("div", container_attrs, children);
                        if let Some(parent) = container_stack.last_mut() {
                            parent.2.push(container);
                        } else {
                            root_children.push(container);
                        }
                        continue;
                    }
                }
            }
            parametric_stack.last_mut().unwrap().1.push(line.clone());
            continue;
        }

        if line.is_each_start() {
            let key = line.meta.tag.as_deref().unwrap_or("").to_string();
            each_stack.push((key, Vec::new()));
            continue;
        }

        if line.is_container_start() {
            let tag = line.meta.tag.as_deref().unwrap_or("div");
            let mut attrs = HashMap::new();
            attrs.insert("class".to_string(), format!("mc-{}", tag));

            if tag == "parametric" {
                let mut attrs = HashMap::new();
                attrs.insert("class".to_string(), "mc-parametric".to_string());
                if let Some(config) = &line.meta.config {
                    let style = config_to_style(config);
                    if !style.is_empty() {
                        attrs.insert("data-config".to_string(), config.clone());
                    }
                }
                parametric_stack.push((attrs, Vec::new()));
                continue;
            }

            if tag == "editor" {
                if let Some(config) = &line.meta.config {
                    let (features, bind_key) = parse_editor_config(config);
                    if !features.is_empty() {
                        attrs.insert("data-features".to_string(), features.join(","));
                    }
                    if let Some(key) = bind_key {
                        attrs.insert("data-bind".to_string(), key);
                    }
                }
                attrs.insert("data-editor".to_string(), "true".to_string());
            } else {
                if let Some(config) = &line.meta.config {
                    attrs.insert("data-config".to_string(), config.clone());
                    let style = config_to_style(config);
                    if !style.is_empty() {
                        attrs.insert("style".to_string(), style);
                    }
                }
            }

            container_stack.push((tag.to_string(), attrs, Vec::new()));
            continue;
        }

        if line.is_container_end() {
            if let Some((tag, attrs, children)) = container_stack.pop() {
                let inner = if tag == "editor" { Vec::new() } else { children };
                let container = VNode::element_with_attrs(&tag_for_container(&tag), attrs, inner);
                if let Some(parent) = container_stack.last_mut() {
                    parent.2.push(container);
                } else {
                    root_children.push(container);
                }
            }
            continue;
        }

        let row_vnode = render_line(line, raw_index, state);
        if let Some(parent) = container_stack.last_mut() {
            parent.2.push(row_vnode);
        } else {
            root_children.push(row_vnode);
        }
        line_counter += 1;
    }

    while let Some((tag, attrs, children)) = container_stack.pop() {
        let container = VNode::element_with_attrs(&tag_for_container(&tag), attrs, children);
        if let Some(parent) = container_stack.last_mut() {
            parent.2.push(container);
        } else {
            root_children.push(container);
        }
    }

    let mut root_attrs = HashMap::new();
    root_attrs.insert("class".to_string(), "mc-app".to_string());
    VNode::element_with_attrs("div", root_attrs, root_children)
}

pub fn render_line(line: &Line, index: usize, state: &StateStore) -> VNode {
    let spans = line.spans();
    let mut children = Vec::new();

    if spans.is_empty() {
        children.push(VNode::element("br", vec![]));
    } else {
        for span in &spans {
            children.push(render_span(span, state));
        }
    }

    let row_class = match line.meta.format {
        block::GRID_ROW => "mc-row mc-grid",
        block::FLEX_ROW => "mc-row mc-flex",
        _ => "mc-row",
    };

    let mut attrs = HashMap::new();
    attrs.insert("class".to_string(), row_class.to_string());
    attrs.insert("data-line".to_string(), index.to_string());
    if let Some(config) = &line.meta.config {
        attrs.insert("data-config".to_string(), config.clone());
    }

    VNode::element_with_attrs("div", attrs, children)
}

pub fn render_line_scoped(
    line: &Line,
    index: usize,
    state: &StateStore,
    scope: &str,
    list_key: &str,
    item_index: usize,
) -> VNode {
    let spans = line.spans();
    let mut children = Vec::new();

    if spans.is_empty() {
        children.push(VNode::element("br", vec![]));
    } else {
        for span in &spans {
            children.push(render_span_scoped(span, state, scope, list_key, item_index));
        }
    }

    let mut attrs = HashMap::new();
    attrs.insert("class".to_string(), "mc-row".to_string());
    attrs.insert("data-line".to_string(), index.to_string());
    attrs.insert("data-scope".to_string(), scope.to_string());

    VNode::element_with_attrs("div", attrs, children)
}

fn render_span_scoped(
    span: &ComponentSpan,
    state: &StateStore,
    scope: &str,
    list_key: &str,
    item_index: usize,
) -> VNode {
    let scoped_key = span.state_key.as_ref()
        .map(|k| format!("{}.{}", scope, k));

    let mut scoped_span = span.clone();
    scoped_span.state_key = scoped_key.clone();

    if let Some(ref key) = scoped_span.state_key {
        if key.ends_with("remove") || key.ends_with("delete") {
            scoped_span.logic_ref = Some(format!("remove:{}:{}", list_key, item_index));
        }
    }

    render_span(&scoped_span, state)
}

fn render_span(span: &ComponentSpan, state: &StateStore) -> VNode {
    let tag = component::tag_for(span.component);
    let mut attrs = HashMap::new();

    let mut classes = Vec::new();
    classes.push(component::css_class_for(span.component).to_string());
    if let Some(s) = span.style {
        let style_class = style::css_class_for(s);
        if !style_class.is_empty() {
            classes.push(style_class.to_string());
        }
    }
    if let Some(col) = span.col {
        if col.1 == 12 {
            classes.push(format!("mc-col-{}", col.0));
        } else {
            classes.push(format!("mc-col-{}-{}", col.0, col.1));
        }
    }
    if let Some(ref resp) = span.responsive {
        for (bp, n, total) in resp {
            if *total == 12 {
                classes.push(format!("mc-{}-col-{}", bp, n));
            } else {
                classes.push(format!("mc-{}-col-{}-{}", bp, n, total));
            }
        }
    }
    if let Some(ref a) = span.animate {
        classes.push(format!("mc-animate-{}", a));
    }
    if let Some(ref p) = span.popover {
        classes.push("mc-has-popover".to_string());
    }

    let class_str = classes.join(" ");
    if !class_str.trim().is_empty() {
        attrs.insert("class".to_string(), class_str.trim().to_string());
    }

    if let Some(ref key) = span.state_key {
        attrs.insert("data-bind".to_string(), key.clone());
    }

    if let Some(ref logic) = span.logic_ref {
        if let Some(route_path) = logic.strip_prefix("route:") {
            attrs.insert("data-route".to_string(), route_path.to_string());
        } else if let Some(fetch_url) = logic.strip_prefix("fetch:") {
            attrs.insert("data-fetch".to_string(), fetch_url.to_string());
        } else {
            attrs.insert("data-logic".to_string(), logic.clone());
        }
    }

    if let Some(ref v) = span.validate {
        attrs.insert("data-validate".to_string(), v.clone());
    }

    if let Some(ref p) = span.popover {
        attrs.insert("data-popover".to_string(), p.clone());
    }

    let width_pct = span.end - span.start;
    attrs.insert("data-span".to_string(), width_pct.to_string());

    match span.component {
        component::TEXT_INPUT | component::PASSWORD_INPUT => {
            if let Some(input_type) = component::input_type_for(span.component) {
                attrs.insert("type".to_string(), input_type.to_string());
            }
            if let Some(ref key) = span.state_key {
                let value = state.get_text(key);
                if !value.is_empty() {
                    attrs.insert("value".to_string(), value);
                }
            }
            let placeholder = span.content.trim().trim_matches('_');
            if !placeholder.is_empty() {
                attrs.insert("placeholder".to_string(), placeholder.to_string());
            }
            VNode::element_with_attrs(tag, attrs, vec![])
        }

        component::CHECKBOX | component::RADIO => {
            if let Some(input_type) = component::input_type_for(span.component) {
                attrs.insert("type".to_string(), input_type.to_string());
            }
            if let Some(ref key) = span.state_key {
                if state.get_bool(key) {
                    attrs.insert("checked".to_string(), "checked".to_string());
                }
            }
            VNode::element_with_attrs(tag, attrs, vec![])
        }

        component::BUTTON => {
            if let Some(ref key) = span.state_key {
                attrs.insert("data-action".to_string(), key.clone());
            }
            let label = span.content.trim();
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(label)])
        }

        component::LABEL => {
            let display = if let Some(ref key) = span.state_key {
                let val = state.get_text(key);
                if val.is_empty() { span.content.clone() } else { val }
            } else {
                span.content.clone()
            };
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(&display)])
        }

        component::LINK => {
            if let Some(ref logic) = span.logic_ref {
                attrs.insert("href".to_string(), logic.clone());
            }
            let label = span.content.trim();
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(label)])
        }

        component::IMAGE => {
            if let Some(ref logic) = span.logic_ref {
                attrs.insert("src".to_string(), logic.clone());
            }
            let alt = span.content.trim();
            if !alt.is_empty() {
                attrs.insert("alt".to_string(), alt.to_string());
            }
            VNode::element_with_attrs(tag, attrs, vec![])
        }

        component::DIVIDER => {
            VNode::element_with_attrs(tag, attrs, vec![])
        }

        component::SELECT => {
            VNode::element_with_attrs(tag, attrs, vec![])
        }

        component::TEXTAREA => {
            let value = span.state_key.as_ref()
                .map(|k| state.get_text(k))
                .unwrap_or_default();
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(&value)])
        }

        component::PILL => {
            let display = if let Some(ref key) = span.state_key {
                let val = state.get_text(key);
                if val.is_empty() { span.content.trim().to_string() } else { val }
            } else {
                span.content.trim().to_string()
            };
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(&display)])
        }

        component::BADGE => {
            let display = if let Some(ref key) = span.state_key {
                let val = state.get_text(key);
                if val.is_empty() { span.content.trim().to_string() } else { val }
            } else {
                span.content.trim().to_string()
            };
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(&display)])
        }

        component::PROGRESS => {
            let value = span.state_key.as_ref()
                .map(|k| state.get_text(k))
                .unwrap_or_else(|| span.content.trim().to_string());
            let pct: f64 = value.parse::<f64>().unwrap_or(0.0).min(100.0).max(0.0);
            attrs.insert("data-value".to_string(), format!("{}", pct));

            let bar_style = format!("width:{:.1}%", pct);
            let mut bar_attrs = HashMap::new();
            bar_attrs.insert("class".to_string(), "mc-progress-bar".to_string());
            bar_attrs.insert("style".to_string(), bar_style);

            let bar = VNode::element_with_attrs("div", bar_attrs, vec![]);
            VNode::element_with_attrs(tag, attrs, vec![bar])
        }

        component::SPARKLINE => {
            let data_str = span.state_key.as_ref()
                .map(|k| state.get_text(k))
                .unwrap_or_else(|| span.content.trim().to_string());
            let svg_content = render_sparkline_svg(&data_str);
            attrs.insert("viewBox".to_string(), "0 0 100 30".to_string());
            attrs.insert("preserveAspectRatio".to_string(), "none".to_string());

            let fill_d = format!("{}L100,30 L0,30 Z", &svg_content);

            let mut path_attrs = HashMap::new();
            path_attrs.insert("d".to_string(), svg_content);
            path_attrs.insert("class".to_string(), "mc-sparkline-path".to_string());
            path_attrs.insert("fill".to_string(), "none".to_string());
            path_attrs.insert("stroke".to_string(), "currentColor".to_string());
            path_attrs.insert("stroke-width".to_string(), "1.5".to_string());
            let path = VNode::element_with_attrs("path", path_attrs, vec![]);

            let mut fill_attrs = HashMap::new();
            fill_attrs.insert("d".to_string(), fill_d);
            fill_attrs.insert("class".to_string(), "mc-sparkline-fill".to_string());
            let fill = VNode::element_with_attrs("path", fill_attrs, vec![]);

            VNode::element_with_attrs(tag, attrs, vec![fill, path])
        }

        component::SPACER => {
            let mode = span.state_key.as_deref().unwrap_or("end");
            let style = spacer_style(mode);
            attrs.insert("class".to_string(), "mc-spacer".to_string());
            if !style.is_empty() {
                attrs.insert("style".to_string(), style);
            }
            attrs.insert("data-spacer".to_string(), mode.to_string());
            VNode::element_with_attrs("span", attrs, vec![])
        }

        component::EMPTY => {
            attrs.insert("class".to_string(), "mc-spacer".to_string());
            VNode::element_with_attrs("span", attrs, vec![])
        }

        _ => {
            VNode::element_with_attrs(tag, attrs, vec![VNode::text(&span.content)])
        }
    }
}

fn render_sparkline_svg(data: &str) -> String {
    let values: Vec<f64> = data.split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();

    if values.is_empty() {
        return "M0,15 L100,15".to_string();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 0.001 { 1.0 } else { max - min };

    let mut path = String::new();
    for (i, &v) in values.iter().enumerate() {
        let x = if values.len() == 1 { 50.0 } else { (i as f64 / (values.len() - 1) as f64) * 100.0 };
        let y = 28.0 - ((v - min) / range) * 26.0 + 2.0;
        if i == 0 {
            path.push_str(&format!("M{:.1},{:.1}", x, y));
        } else {
            path.push_str(&format!(" L{:.1},{:.1}", x, y));
        }
    }

    path
}

fn spacer_style(mode: &str) -> String {
    if mode == "evenly" || mode == "even" {
        return "flex:1".to_string();
    }

    if mode == "end" {
        return "flex:1".to_string();
    }

    // col-N: fixed width of N/12
    if let Some(rest) = mode.strip_prefix("col-") {
        if let Some(end_rest) = rest.strip_suffix("-end") {
            // col-N-end: fill up to end of column N
            if let Ok(n) = end_rest.parse::<u8>() {
                if n > 0 && n <= 12 {
                    let pct = (n as f64 / 12.0) * 100.0;
                    return format!("flex:0 0 {:.4}%;max-width:{:.4}%", pct, pct);
                }
            }
        } else {
            // col-N: fixed N/12 width spacer
            if let Ok(n) = rest.parse::<u8>() {
                if n > 0 && n <= 12 {
                    let pct = (n as f64 / 12.0) * 100.0;
                    return format!("flex:0 0 {:.4}%;max-width:{:.4}%", pct, pct);
                }
            }
        }
    }

    "flex:1".to_string()
}

fn parse_editor_config(config: &str) -> (Vec<String>, Option<String>) {
    let valid_features = [
        "bold", "italic", "underline", "strikethrough", "code",
        "heading", "list", "ordered-list", "quote", "code-block",
        "link", "image", "divider", "hr",
    ];
    let mut features = Vec::new();
    let mut bind_key = None;

    for token in config.split_whitespace() {
        if let Some(key) = token.strip_prefix("bind:") {
            bind_key = Some(key.to_string());
        } else if valid_features.contains(&token) {
            features.push(token.to_string());
        }
    }

    (features, bind_key)
}

fn tag_for_container(tag_name: &str) -> String {
    match tag_name {
        "nav" => "nav".to_string(),
        "header" => "header".to_string(),
        "footer" => "footer".to_string(),
        "main" => "main".to_string(),
        "section" => "section".to_string(),
        "article" => "article".to_string(),
        "aside" => "aside".to_string(),
        "form" => "form".to_string(),
        _ => "div".to_string(),
    }
}

fn config_to_style(config: &str) -> String {
    let mut styles = Vec::new();

    for part in config.split(|c| c == ',' || c == ' ') {
        let part = part.trim();
        if part.is_empty() { continue; }

        if let Some((key, value)) = part.split_once(':') {
            match key {
                "cols" => {
                    styles.push(format!("column-count:{}", value));
                }
                "gap" => {
                    let v = ensure_unit(value);
                    styles.push(format!("column-gap:{};gap:{}", v, v));
                }
                "height" => {
                    styles.push(format!("height:{}", ensure_unit(value)));
                    styles.push("overflow:hidden".to_string());
                }
                "max-height" => {
                    styles.push(format!("max-height:{}", ensure_unit(value)));
                    styles.push("overflow:hidden".to_string());
                }
                "padding" => {
                    styles.push(format!("padding:{}",ensure_unit(value)));
                }
                "max-width" => {
                    styles.push(format!("max-width:{}", ensure_unit(value)));
                }
                "width" => {
                    styles.push(format!("width:{}", ensure_unit(value)));
                }
                _ => {}
            }
        }
    }

    styles.join(";")
}

fn ensure_unit(value: &str) -> String {
    if value.ends_with("px") || value.ends_with('%') || value.ends_with("em")
        || value.ends_with("rem") || value.ends_with("vh") || value.ends_with("vw")
        || value == "auto" || value == "0"
    {
        value.to_string()
    } else if value.parse::<f64>().is_ok() {
        format!("{}px", value)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_document() {
        let lines: Vec<Line> = vec![];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);
        assert_eq!(vdom.tag(), Some("div"));
        assert_eq!(vdom.children().len(), 0);
    }

    #[test]
    fn test_render_single_label() {
        let lines = vec![Line::label("Hello")];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        assert_eq!(vdom.children().len(), 1);
        let row = &vdom.children()[0];
        assert_eq!(row.tag(), Some("div"));

        let label = &row.children()[0];
        assert_eq!(label.tag(), Some("span"));
        match label {
            VNode::Element(el) => {
                assert!(el.attrs.get("class").unwrap().contains("mc-label"));
                match &el.children[0] {
                    VNode::Text(t) => assert_eq!(t.content, "Hello"),
                    _ => panic!("expected text"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_render_input_with_state() {
        let lines = vec![Line::content_row(
            "________",
            "IIIIIIII",
            "username",
            "        ",
        )];
        let mut state = StateStore::new();
        state.set_text("username", "Alice");
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        let input = &row.children()[0];
        match input {
            VNode::Element(el) => {
                assert_eq!(el.tag, "input");
                assert_eq!(el.attrs.get("type").unwrap(), "text");
                assert_eq!(el.attrs.get("value").unwrap(), "Alice");
                assert_eq!(el.attrs.get("data-bind").unwrap(), "username");
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_render_button_with_action() {
        let lines = vec![Line::content_row(
            "Submit",
            "BBBBBB",
            "submit",
            "pppppp",
        )];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        let button = &row.children()[0];
        match button {
            VNode::Element(el) => {
                assert_eq!(el.tag, "button");
                assert_eq!(el.attrs.get("data-action").unwrap(), "submit");
                assert!(el.attrs.get("class").unwrap().contains("mc-primary"));
                match &el.children[0] {
                    VNode::Text(t) => assert_eq!(t.content, "Submit"),
                    _ => panic!("expected text"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_render_checkbox() {
        let lines = vec![Line::content_row("check", "CCCCC", "agree", "     ")];
        let mut state = StateStore::new();
        state.set_bool("agree", true);
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        let checkbox = &row.children()[0];
        match checkbox {
            VNode::Element(el) => {
                assert_eq!(el.tag, "input");
                assert_eq!(el.attrs.get("type").unwrap(), "checkbox");
                assert_eq!(el.attrs.get("checked").unwrap(), "checked");
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_render_container() {
        let lines = vec![
            Line::container_start("card", Some("shadow:md")),
            Line::label("Inside card"),
            Line::container_end("card"),
        ];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        assert_eq!(vdom.children().len(), 1);
        let container = &vdom.children()[0];
        assert_eq!(container.tag(), Some("div"));
        match container {
            VNode::Element(el) => {
                assert!(el.attrs.get("class").unwrap().contains("mc-card"));
                assert_eq!(el.attrs.get("data-config").unwrap(), "shadow:md");
                assert_eq!(el.children.len(), 1);
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_render_nested_containers() {
        let lines = vec![
            Line::container_start("page", None),
            Line::container_start("sidebar", None),
            Line::label("Nav"),
            Line::container_end("sidebar"),
            Line::container_start("main", None),
            Line::label("Content"),
            Line::container_end("main"),
            Line::container_end("page"),
        ];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        assert_eq!(vdom.children().len(), 1);
        let page = &vdom.children()[0];
        assert_eq!(page.children().len(), 2);

        let sidebar = &page.children()[0];
        assert_eq!(sidebar.children().len(), 1);

        let main = &page.children()[1];
        assert_eq!(main.tag(), Some("main"));
        assert_eq!(main.children().len(), 1);
    }

    #[test]
    fn test_render_mixed_row() {
        let lines = vec![Line::content_row(
            "Name  ________  Go",
            "LLLLLLIIIIIIIIIIBBB",
            "______name______sub",
            "                ppp",
        )];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        assert_eq!(row.children().len(), 3);
        assert_eq!(row.children()[0].tag(), Some("span"));
        assert_eq!(row.children()[1].tag(), Some("input"));
        assert_eq!(row.children()[2].tag(), Some("button"));
    }

    #[test]
    fn test_render_empty_line() {
        let lines = vec![Line::new()];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        assert_eq!(row.children().len(), 1);
        assert_eq!(row.children()[0].tag(), Some("br"));
    }

    #[test]
    fn test_reactive_label() {
        let lines = vec![Line::content_row(
            "Count: 0",
            "LLLLLLLL",
            "___count",
            "        ",
        )];
        let mut state = StateStore::new();
        state.set_number("count", 42.0);
        let vdom = render_document(&lines, &state);

        let row = &vdom.children()[0];
        let label = &row.children()[0];
        match label {
            VNode::Element(el) => {
                match &el.children[0] {
                    VNode::Text(t) => assert_eq!(t.content, "42"),
                    _ => panic!("expected text"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_semantic_container_tags() {
        assert_eq!(tag_for_container("nav"), "nav");
        assert_eq!(tag_for_container("header"), "header");
        assert_eq!(tag_for_container("main"), "main");
        assert_eq!(tag_for_container("form"), "form");
        assert_eq!(tag_for_container("card"), "div");
        assert_eq!(tag_for_container("modal"), "div");
    }

    #[test]
    fn test_data_line_attribute() {
        let lines = vec![Line::label("A"), Line::label("B")];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        match &vdom.children()[0] {
            VNode::Element(el) => assert_eq!(el.attrs.get("data-line").unwrap(), "0"),
            _ => panic!("expected element"),
        }
        match &vdom.children()[1] {
            VNode::Element(el) => assert_eq!(el.attrs.get("data-line").unwrap(), "1"),
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_columns_container() {
        let lines = vec![
            Line::container_start("columns", Some("cols:3,gap:16,height:400px")),
            Line::label("Column content A"),
            Line::label("Column content B"),
            Line::label("Column content C"),
            Line::container_end("columns"),
        ];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let container = &vdom.children()[0];
        match container {
            VNode::Element(el) => {
                assert!(el.attrs.get("class").unwrap().contains("mc-columns"));
                let style = el.attrs.get("style").unwrap();
                assert!(style.contains("column-count:3"));
                assert!(style.contains("column-gap:16px"));
                assert!(style.contains("height:400px"));
                assert!(style.contains("overflow:hidden"));
                assert_eq!(el.children.len(), 3);
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_config_to_style() {
        let style = config_to_style("cols:2,gap:20,height:300px");
        assert!(style.contains("column-count:2"));
        assert!(style.contains("column-gap:20px"));
        assert!(style.contains("height:300px"));
    }

    #[test]
    fn test_config_to_style_with_units() {
        let style = config_to_style("padding:16,max-width:800px,width:50%");
        assert!(style.contains("padding:16px"));
        assert!(style.contains("max-width:800px"));
        assert!(style.contains("width:50%"));
    }

    #[test]
    fn test_render_editor_container() {
        let lines = vec![
            Line::container_start("editor", Some("bold italic code bind:content")),
            Line::label("This should be discarded"),
            Line::container_end("editor"),
        ];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let container = &vdom.children()[0];
        match container {
            VNode::Element(el) => {
                assert!(el.attrs.get("class").unwrap().contains("mc-editor"));
                assert_eq!(el.attrs.get("data-editor").unwrap(), "true");
                assert_eq!(el.attrs.get("data-features").unwrap(), "bold,italic,code");
                assert_eq!(el.attrs.get("data-bind").unwrap(), "content");
                assert_eq!(el.children.len(), 0, "editor container should be opaque");
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_editor_config() {
        let (features, bind) = parse_editor_config("bold italic heading bind:notes");
        assert_eq!(features, vec!["bold", "italic", "heading"]);
        assert_eq!(bind, Some("notes".to_string()));

        let (features2, bind2) = parse_editor_config("code quote");
        assert_eq!(features2, vec!["code", "quote"]);
        assert_eq!(bind2, None);
    }

    #[test]
    fn test_config_padding_applied() {
        let lines = vec![
            Line::container_start("card", Some("padding:24")),
            Line::label("Inside"),
            Line::container_end("card"),
        ];
        let state = StateStore::new();
        let vdom = render_document(&lines, &state);

        let container = &vdom.children()[0];
        match container {
            VNode::Element(el) => {
                let style = el.attrs.get("style").unwrap();
                assert!(style.contains("padding:24px"));
            }
            _ => panic!("expected element"),
        }
    }
}
