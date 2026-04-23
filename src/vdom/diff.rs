use super::node::{VNode, VElement};
use super::patch::Patch;

pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_node(old, new, &mut Vec::new(), &mut patches);
    patches
}

fn diff_node(old: &VNode, new: &VNode, path: &mut Vec<usize>, patches: &mut Vec<Patch>) {
    if old == new {
        return;
    }

    match (old, new) {
        (VNode::Text(old_t), VNode::Text(new_t)) => {
            if old_t.content != new_t.content {
                patches.push(Patch::UpdateText {
                    path: path.clone(),
                    text: new_t.content.clone(),
                });
            }
        }

        (VNode::Element(old_el), VNode::Element(new_el)) if old_el.tag == new_el.tag => {
            diff_attrs(old_el, new_el, path, patches);
            diff_children(old_el, new_el, path, patches);
        }

        _ => {
            patches.push(Patch::Replace {
                path: path.clone(),
                node: new.clone(),
            });
        }
    }
}

fn diff_attrs(old: &VElement, new: &VElement, path: &[usize], patches: &mut Vec<Patch>) {
    for (key, new_val) in &new.attrs {
        match old.attrs.get(key) {
            Some(old_val) if old_val == new_val => {}
            _ => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    key: key.clone(),
                    value: new_val.clone(),
                });
            }
        }
    }

    for key in old.attrs.keys() {
        if !new.attrs.contains_key(key) {
            patches.push(Patch::RemoveAttribute {
                path: path.to_vec(),
                key: key.clone(),
            });
        }
    }
}

fn diff_children(
    old: &VElement,
    new: &VElement,
    path: &mut Vec<usize>,
    patches: &mut Vec<Patch>,
) {
    let old_children = &old.children;
    let new_children = &new.children;

    let old_keyed = build_key_map(old_children);
    let new_keyed = build_key_map(new_children);

    if !old_keyed.is_empty() || !new_keyed.is_empty() {
        diff_keyed_children(old_children, new_children, &old_keyed, &new_keyed, path, patches);
    } else {
        diff_indexed_children(old_children, new_children, path, patches);
    }
}

fn build_key_map(children: &[VNode]) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for (i, child) in children.iter().enumerate() {
        if let VNode::Element(el) = child {
            if let Some(key) = el.attrs.get("data-line") {
                map.insert(key.clone(), i);
            }
        }
    }
    map
}

fn diff_keyed_children(
    old_children: &[VNode],
    new_children: &[VNode],
    old_keyed: &std::collections::HashMap<String, usize>,
    new_keyed: &std::collections::HashMap<String, usize>,
    path: &mut Vec<usize>,
    patches: &mut Vec<Patch>,
) {
    let new_keys: Vec<Option<String>> = new_children
        .iter()
        .map(|c| {
            if let VNode::Element(el) = c {
                el.attrs.get("data-line").cloned()
            } else {
                None
            }
        })
        .collect();

    for (new_idx, new_child) in new_children.iter().enumerate() {
        let new_key = new_keys[new_idx].as_deref();

        if let Some(key) = new_key {
            if let Some(&old_idx) = old_keyed.get(key) {
                path.push(new_idx);
                diff_node(&old_children[old_idx], new_child, path, patches);
                path.pop();
            } else {
                patches.push(Patch::Insert {
                    path: {
                        let mut p = path.clone();
                        p.push(new_idx);
                        p
                    },
                    node: new_child.clone(),
                });
            }
        } else {
            path.push(new_idx);
            if new_idx < old_children.len() {
                diff_node(&old_children[new_idx], new_child, path, patches);
            } else {
                patches.push(Patch::Insert {
                    path: path.clone(),
                    node: new_child.clone(),
                });
            }
            path.pop();
        }
    }

    for (key, &old_idx) in old_keyed {
        if !new_keyed.contains_key(key) {
            patches.push(Patch::Remove {
                path: {
                    let mut p = path.clone();
                    p.push(old_idx);
                    p
                },
            });
        }
    }
}

fn diff_indexed_children(
    old_children: &[VNode],
    new_children: &[VNode],
    path: &mut Vec<usize>,
    patches: &mut Vec<Patch>,
) {
    let min_len = old_children.len().min(new_children.len());

    for i in 0..min_len {
        path.push(i);
        diff_node(&old_children[i], &new_children[i], path, patches);
        path.pop();
    }

    for i in min_len..new_children.len() {
        patches.push(Patch::Insert {
            path: {
                let mut p = path.clone();
                p.push(i);
                p
            },
            node: new_children[i].clone(),
        });
    }

    for i in (min_len..old_children.len()).rev() {
        patches.push(Patch::Remove {
            path: {
                let mut p = path.clone();
                p.push(i);
                p
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_trees() {
        let tree = VNode::element("div", vec![VNode::text("Hello")]);
        assert!(diff(&tree, &tree).is_empty());
    }

    #[test]
    fn test_text_update() {
        let old = VNode::text("Hello");
        let new = VNode::text("World");
        let patches = diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::UpdateText { text, .. } if text == "World"));
    }

    #[test]
    fn test_replace_different_tags() {
        let old = VNode::element("p", vec![]);
        let new = VNode::element("h1", vec![]);
        let patches = diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Replace { .. }));
    }

    #[test]
    fn test_child_added() {
        let old = VNode::element("div", vec![VNode::text("A")]);
        let new = VNode::element("div", vec![VNode::text("A"), VNode::text("B")]);
        let patches = diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Insert { .. }));
    }

    #[test]
    fn test_child_removed() {
        let old = VNode::element("div", vec![VNode::text("A"), VNode::text("B")]);
        let new = VNode::element("div", vec![VNode::text("A")]);
        let patches = diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Remove { .. }));
    }
}
