use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum VNode {
    Element(VElement),
    Text(VText),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VElement {
    pub tag: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attrs: HashMap<String, String>,
    pub children: Vec<VNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VText {
    pub content: String,
}

impl VNode {
    pub fn element(tag: &str, children: Vec<VNode>) -> Self {
        VNode::Element(VElement {
            tag: tag.to_string(),
            attrs: HashMap::new(),
            children,
        })
    }

    pub fn element_with_attrs(
        tag: &str,
        attrs: HashMap<String, String>,
        children: Vec<VNode>,
    ) -> Self {
        VNode::Element(VElement {
            tag: tag.to_string(),
            attrs,
            children,
        })
    }

    pub fn text(content: &str) -> Self {
        VNode::Text(VText {
            content: content.to_string(),
        })
    }

    pub fn is_text(&self) -> bool {
        matches!(self, VNode::Text(_))
    }

    pub fn is_element(&self) -> bool {
        matches!(self, VNode::Element(_))
    }

    pub fn tag(&self) -> Option<&str> {
        match self {
            VNode::Element(el) => Some(&el.tag),
            VNode::Text(_) => None,
        }
    }

    pub fn children(&self) -> &[VNode] {
        match self {
            VNode::Element(el) => &el.children,
            VNode::Text(_) => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_node() {
        let node = VNode::text("Hello");
        assert!(node.is_text());
        assert!(!node.is_element());
    }

    #[test]
    fn test_element_node() {
        let node = VNode::element("div", vec![VNode::text("child")]);
        assert!(node.is_element());
        assert_eq!(node.tag(), Some("div"));
        assert_eq!(node.children().len(), 1);
    }

    #[test]
    fn test_element_with_attrs() {
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "mc-row".to_string());
        let node = VNode::element_with_attrs("div", attrs, vec![]);
        match &node {
            VNode::Element(el) => assert_eq!(el.attrs.get("class").unwrap(), "mc-row"),
            _ => panic!("expected element"),
        }
    }
}
