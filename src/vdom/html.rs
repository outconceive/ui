use super::node::VNode;

pub fn vnode_to_html(node: &VNode) -> String {
    match node {
        VNode::Text(t) => escape_html(&t.content),
        VNode::Element(el) => {
            let mut html = String::new();
            html.push('<');
            html.push_str(&el.tag);

            let mut keys: Vec<&String> = el.attrs.keys().collect();
            keys.sort();
            for key in keys {
                let value = &el.attrs[key];
                html.push(' ');
                html.push_str(key);
                html.push_str("=\"");
                html.push_str(&escape_attr(value));
                html.push('"');
            }

            if is_void_element(&el.tag) {
                html.push_str("/>");
                return html;
            }

            html.push('>');

            for child in &el.children {
                html.push_str(&vnode_to_html(child));
            }

            html.push_str("</");
            html.push_str(&el.tag);
            html.push('>');
            html
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed"
            | "source" | "track" | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdom::node::VNode;
    use std::collections::HashMap;

    #[test]
    fn test_text_node() {
        assert_eq!(vnode_to_html(&VNode::text("Hello")), "Hello");
    }

    #[test]
    fn test_text_escaping() {
        assert_eq!(vnode_to_html(&VNode::text("<b>bold</b>")), "&lt;b&gt;bold&lt;/b&gt;");
    }

    #[test]
    fn test_simple_element() {
        let node = VNode::element("div", vec![VNode::text("Hello")]);
        assert_eq!(vnode_to_html(&node), "<div>Hello</div>");
    }

    #[test]
    fn test_element_with_attrs() {
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "mc-row".to_string());
        attrs.insert("data-line".to_string(), "0".to_string());
        let node = VNode::element_with_attrs("div", attrs, vec![]);
        let html = vnode_to_html(&node);
        assert!(html.contains("class=\"mc-row\""));
        assert!(html.contains("data-line=\"0\""));
    }

    #[test]
    fn test_void_element() {
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "text".to_string());
        let node = VNode::element_with_attrs("input", attrs, vec![]);
        let html = vnode_to_html(&node);
        assert!(html.ends_with("/>"));
        assert!(!html.contains("</input>"));
    }

    #[test]
    fn test_br_element() {
        let node = VNode::element("br", vec![]);
        assert_eq!(vnode_to_html(&node), "<br/>");
    }

    #[test]
    fn test_nested() {
        let node = VNode::element("div", vec![
            VNode::element("span", vec![VNode::text("Hello")]),
            VNode::text(" "),
            VNode::element("span", vec![VNode::text("World")]),
        ]);
        assert_eq!(vnode_to_html(&node), "<div><span>Hello</span> <span>World</span></div>");
    }

    #[test]
    fn test_attr_escaping() {
        let mut attrs = HashMap::new();
        attrs.insert("title".to_string(), "A \"quoted\" value".to_string());
        let node = VNode::element_with_attrs("div", attrs, vec![]);
        assert!(vnode_to_html(&node).contains("A &quot;quoted&quot; value"));
    }
}
