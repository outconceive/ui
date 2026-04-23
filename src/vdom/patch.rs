use serde::Serialize;
use super::node::VNode;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Patch {
    Replace {
        path: Vec<usize>,
        node: VNode,
    },
    Insert {
        path: Vec<usize>,
        node: VNode,
    },
    Remove {
        path: Vec<usize>,
    },
    UpdateText {
        path: Vec<usize>,
        text: String,
    },
    SetAttribute {
        path: Vec<usize>,
        key: String,
        value: String,
    },
    RemoveAttribute {
        path: Vec<usize>,
        key: String,
    },
}
