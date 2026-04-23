pub mod block;
pub mod component;
pub mod document;
pub mod line;
pub mod markout;
pub mod state;
pub mod style;
pub mod vdom;

use wasm_bindgen::prelude::*;
use serde::Serialize;

use document::Document;
use line::Line;
use vdom::diff::diff;
use vdom::node::VNode;
use vdom::patch::Patch;
use vdom::render;

pub struct OutconceiveCore {
    pub document: Document,
    old_vdom: Option<VNode>,
    old_line_vnodes: std::collections::HashMap<usize, VNode>,
}

impl OutconceiveCore {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            old_vdom: None,
            old_line_vnodes: std::collections::HashMap::new(),
        }
    }

    pub fn from_document(document: Document) -> Self {
        Self {
            document,
            old_vdom: None,
            old_line_vnodes: std::collections::HashMap::new(),
        }
    }

    pub fn initial_render(&mut self) -> VNode {
        let vdom = render::render_document(&self.document.lines, &self.document.state);
        self.cache_line_vnodes();
        self.old_vdom = Some(vdom.clone());
        vdom
    }

    pub fn render_and_diff(&mut self) -> Vec<Patch> {
        let new_vdom = render::render_document(&self.document.lines, &self.document.state);

        let patches = if let Some(ref old) = self.old_vdom {
            let old_count = self.old_line_vnodes.len();
            let new_count = self.content_line_count();
            if old_count != new_count {
                vec![Patch::Replace {
                    path: vec![],
                    node: new_vdom.clone(),
                }]
            } else {
                diff(old, &new_vdom)
            }
        } else {
            vec![Patch::Replace {
                path: vec![],
                node: new_vdom.clone(),
            }]
        };

        self.cache_line_vnodes();
        self.old_vdom = Some(new_vdom);
        patches
    }


    pub fn render_dirty_lines(&mut self) -> Vec<Patch> {
        let dirty = self.document.dirty_lines();
        if dirty.is_empty() {
            return Vec::new();
        }
        self.render_and_diff()
    }

    pub fn render_lines_incremental(&mut self, line_indices: &[usize]) -> Vec<Patch> {
        let mut patches = Vec::new();

        for &idx in line_indices {
            if idx >= self.document.lines.len() {
                continue;
            }

            let new_line_vnode = render::render_line(
                &self.document.lines[idx],
                idx,
                &self.document.state,
            );

            if let Some(old_vnode) = self.old_line_vnodes.get(&idx) {
                let line_patches = diff(old_vnode, &new_line_vnode);
                for patch in line_patches {
                    patches.push(prepend_path(patch, idx));
                }
            }
            self.old_line_vnodes.insert(idx, new_line_vnode.clone());

            self.update_vdom_child(idx, new_line_vnode);
        }

        patches
    }

    pub fn update_state(&mut self, key: &str, value: state::StateValue) -> Vec<Patch> {
        self.document.state.set(key, value);
        self.render_dirty_lines()
    }

    pub fn update_state_text(&mut self, key: &str, value: &str) -> Vec<Patch> {
        self.document.state.set_text(key, value);
        self.render_dirty_lines()
    }

    pub fn toggle_state(&mut self, key: &str) -> Vec<Patch> {
        self.document.state.toggle(key);
        self.render_dirty_lines()
    }

    pub fn add_line(&mut self, line: Line) -> Vec<Patch> {
        self.document.add_line(line);
        self.render_and_diff()
    }

    pub fn insert_line(&mut self, index: usize, line: Line) -> Vec<Patch> {
        self.document.insert_line(index, line);
        self.render_and_diff()
    }

    pub fn remove_line(&mut self, index: usize) -> Vec<Patch> {
        self.document.remove_line(index);
        self.render_and_diff()
    }

    fn content_line_count(&self) -> usize {
        self.document.lines.iter()
            .filter(|l| !l.is_container_start() && !l.is_container_end())
            .count()
    }

    fn cache_line_vnodes(&mut self) {
        self.old_line_vnodes.clear();
        for (i, line) in self.document.lines.iter().enumerate() {
            if !line.is_container_start() && !line.is_container_end() {
                self.old_line_vnodes.insert(
                    i,
                    render::render_line(line, i, &self.document.state),
                );
            }
        }
    }

    fn update_vdom_child(&mut self, child_index: usize, new_child: VNode) {
        if let Some(VNode::Element(ref mut root)) = self.old_vdom {
            fn update_at(children: &mut Vec<VNode>, target_line: usize, new_child: VNode) -> bool {
                for child in children.iter_mut() {
                    if let VNode::Element(el) = child {
                        if el.attrs.get("data-line").map(|s| s.as_str())
                            == Some(&target_line.to_string())
                        {
                            *child = new_child;
                            return true;
                        }
                        if update_at(&mut el.children, target_line, new_child.clone()) {
                            return true;
                        }
                    }
                }
                false
            }
            update_at(&mut root.children, child_index, new_child);
        }
    }
}

fn prepend_path(patch: Patch, parent_index: usize) -> Patch {
    match patch {
        Patch::Replace { mut path, node } => {
            path.insert(0, parent_index);
            Patch::Replace { path, node }
        }
        Patch::Insert { mut path, node } => {
            path.insert(0, parent_index);
            Patch::Insert { path, node }
        }
        Patch::Remove { mut path } => {
            path.insert(0, parent_index);
            Patch::Remove { path }
        }
        Patch::UpdateText { mut path, text } => {
            path.insert(0, parent_index);
            Patch::UpdateText { path, text }
        }
        Patch::SetAttribute { mut path, key, value } => {
            path.insert(0, parent_index);
            Patch::SetAttribute { path, key, value }
        }
        Patch::RemoveAttribute { mut path, key } => {
            path.insert(0, parent_index);
            Patch::RemoveAttribute { path, key }
        }
    }
}

// === WASM Bindings ===

#[wasm_bindgen]
pub struct OutconceiveApp {
    core: OutconceiveCore,
}

#[wasm_bindgen]
impl OutconceiveApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: OutconceiveCore::new(),
        }
    }

    #[wasm_bindgen]
    pub fn from_markout(&mut self, input: &str) {
        self.core = OutconceiveCore::from_document(markout::parse(input));
    }

    #[wasm_bindgen]
    pub fn to_markout(&self) -> String {
        markout::emit(&self.core.document)
    }

    #[wasm_bindgen]
    pub fn demo_login_form(&mut self) {
        let lines = vec![
            Line::container_start("card", Some("shadow:md,padding:24,max-width:400px")),
            Line::content_row(
                "Login",
                "LLLLL",
                "_____",
                "     ",
            ),
            Line::content_row(
                "Username  ________________",
                "LLLLLLLLLLIIIIIIIIIIIIIIIII",
                "__________username________",
                "                          ",
            ),
            Line::content_row(
                "Password  ________________",
                "LLLLLLLLLLPPPPPPPPPPPPPPPP",
                "__________password________",
                "                          ",
            ),
            Line::content_row(
                "  Remember me",
                "CCCCCCCCCCCCC",
                "remember_____",
                "             ",
            ),
            Line::content_row(
                "  Sign In  ",
                "BBBBBBBBBBB",
                "submit_____",
                "ppppppppppp",
            ),
            Line::content_row(
                "Status: Ready",
                "LLLLLLLLLLLLL",
                "____status___",
                "             ",
            ),
            Line::container_end("card"),
        ];
        self.core = OutconceiveCore::from_document(Document::from_lines(lines));
    }

    #[wasm_bindgen]
    pub fn add_row(
        &mut self,
        content: &str,
        components: &str,
        state_keys: &str,
        styles: &str,
    ) -> JsValue {
        let patches = self.core.add_line(
            Line::content_row(content, components, state_keys, styles),
        );
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn add_container_start(&mut self, tag: &str, config: &str) -> JsValue {
        let cfg = if config.is_empty() { None } else { Some(config) };
        let patches = self.core.add_line(Line::container_start(tag, cfg));
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn add_container_end(&mut self, tag: &str) -> JsValue {
        let patches = self.core.add_line(Line::container_end(tag));
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn initial_render(&mut self) -> JsValue {
        let vdom = self.core.initial_render();
        self.to_js(&vdom)
    }

    #[wasm_bindgen]
    pub fn update_state(&mut self, key: &str, value: &str) -> JsValue {
        let patches = self.core.update_state_text(key, value);
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn toggle_state(&mut self, key: &str) -> JsValue {
        let patches = self.core.toggle_state(key);
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn get_state(&self, key: &str) -> JsValue {
        let value = self.core.document.state.get_text(key);
        JsValue::from_str(&value)
    }

    #[wasm_bindgen]
    pub fn get_state_bool(&self, key: &str) -> bool {
        self.core.document.state.get_bool(key)
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> JsValue {
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    // === SSR Methods ===

    #[wasm_bindgen]
    pub fn render_to_html(&mut self) -> String {
        let vdom = self.core.initial_render();
        vdom::html::vnode_to_html(&vdom)
    }

    #[wasm_bindgen]
    pub fn markout_to_html(input: &str) -> String {
        let doc = markout::parse(input);
        let core_state = &doc.state;
        let vdom = render::render_document(&doc.lines, core_state);
        vdom::html::vnode_to_html(&vdom)
    }

    // === List Methods ===

    #[wasm_bindgen]
    pub fn add_list_item(&mut self, list_key: &str, fields_json: &str) -> JsValue {
        let fields = parse_fields_json(fields_json);
        self.core.document.state.add_list_item(list_key, &fields);
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn remove_list_item(&mut self, list_key: &str, index: usize) -> JsValue {
        self.core.document.state.remove_list_item(list_key, index);
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn set_list_item(&mut self, list_key: &str, index: usize, fields_json: &str) -> JsValue {
        let fields = parse_fields_json(fields_json);
        self.core.document.state.set_list_item(list_key, index, &fields);
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn get_list_count(&self, list_key: &str) -> usize {
        self.core.document.state.get_list_count(list_key)
    }

    // === IDE Methods ===

    #[wasm_bindgen]
    pub fn insert_component(
        &mut self,
        at_line: usize,
        comp_type: &str,
        label: &str,
        state_key: &str,
        style_name: &str,
    ) -> JsValue {
        let markout_line = build_markout_component(comp_type, label, state_key, style_name);
        let line = markout::parse(&format!("| {}", markout_line))
            .lines.into_iter().next().unwrap_or_else(Line::new);
        let patches = self.core.insert_line(at_line, line);
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn insert_container(&mut self, at_line: usize, tag: &str, config: &str) -> JsValue {
        let cfg = if config.is_empty() { None } else { Some(config) };
        self.core.document.insert_line(at_line, Line::container_start(tag, cfg));
        self.core.document.insert_line(at_line + 1, Line::new());
        self.core.document.insert_line(at_line + 2, Line::container_end(tag));
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn update_line_component(
        &mut self,
        line_index: usize,
        comp_type: &str,
        label: &str,
        state_key: &str,
        style_name: &str,
    ) -> JsValue {
        if line_index >= self.core.document.lines.len() {
            return JsValue::NULL;
        }
        let markout_line = build_markout_component(comp_type, label, state_key, style_name);
        let new_line = markout::parse(&format!("| {}", markout_line))
            .lines.into_iter().next().unwrap_or_else(Line::new);
        self.core.document.lines[line_index] = new_line;
        self.core.document.rebuild_index();
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn remove_line_at(&mut self, index: usize) -> JsValue {
        if index >= self.core.document.lines.len() {
            return JsValue::NULL;
        }
        let patches = self.core.remove_line(index);
        self.to_js(&patches)
    }

    #[wasm_bindgen]
    pub fn get_line_count(&self) -> usize {
        self.core.document.line_count()
    }

    #[wasm_bindgen]
    pub fn get_line_info(&self, index: usize) -> JsValue {
        if index >= self.core.document.lines.len() {
            return JsValue::NULL;
        }
        let line = &self.core.document.lines[index];

        #[derive(Serialize)]
        struct LineInfo {
            content: String,
            components: String,
            state_keys: String,
            styles: String,
            is_container_start: bool,
            is_container_end: bool,
            tag: Option<String>,
            config: Option<String>,
        }

        let info = LineInfo {
            content: line.content.clone(),
            components: line.components.clone(),
            state_keys: line.state_keys.clone(),
            styles: line.styles.clone(),
            is_container_start: line.is_container_start(),
            is_container_end: line.is_container_end(),
            tag: line.meta.tag.clone(),
            config: line.meta.config.clone(),
        };
        self.to_js(&info)
    }

    #[wasm_bindgen]
    pub fn move_line(&mut self, from: usize, to: usize) -> JsValue {
        let len = self.core.document.lines.len();
        if from >= len || to >= len || from == to {
            return JsValue::NULL;
        }
        let line = self.core.document.remove_line(from);
        let target = if to > from { to - 1 } else { to };
        self.core.document.insert_line(target, line);
        let patches = self.core.render_and_diff();
        self.to_js(&patches)
    }

    fn to_js<T: Serialize>(&self, value: &T) -> JsValue {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        value.serialize(&serializer).unwrap_or(JsValue::NULL)
    }
}

fn parse_fields_json(json: &str) -> Vec<(String, state::StateValue)> {
    let mut fields = Vec::new();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                let sv = match val {
                    serde_json::Value::String(s) => state::StateValue::Text(s.clone()),
                    serde_json::Value::Number(n) => {
                        state::StateValue::Number(n.as_f64().unwrap_or(0.0))
                    }
                    serde_json::Value::Bool(b) => state::StateValue::Bool(*b),
                    serde_json::Value::Null => state::StateValue::Null,
                    _ => state::StateValue::Text(val.to_string()),
                };
                fields.push((key.clone(), sv));
            }
        }
    }
    fields
}

fn build_markout_component(comp_type: &str, label: &str, state_key: &str, style_name: &str) -> String {
    let mut parts = Vec::new();
    let mut type_part = comp_type.to_string();
    if !state_key.is_empty() {
        type_part.push(':');
        type_part.push_str(state_key);
    }
    parts.push(type_part);

    if !label.is_empty() {
        parts.push(format!("\"{}\"", label));
    }

    if !style_name.is_empty() {
        parts.push(style_name.to_string());
    }

    format!("{{{}}}", parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_core() {
        let mut core = OutconceiveCore::new();
        let vdom = core.initial_render();
        assert_eq!(vdom.tag(), Some("div"));
    }

    #[test]
    fn test_initial_render_with_content() {
        let mut doc = Document::new();
        doc.label("Hello World");

        let mut core = OutconceiveCore::from_document(doc);
        let vdom = core.initial_render();
        assert_eq!(vdom.children().len(), 2);
    }

    #[test]
    fn test_state_update_produces_patches() {
        let mut doc = Document::from_lines(vec![
            Line::content_row("________", "IIIIIIII", "username", "        "),
        ]);
        doc.state.set_text("username", "");

        let mut core = OutconceiveCore::from_document(doc);
        core.initial_render();

        let patches = core.update_state_text("username", "Alice");
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_no_patches_when_no_change() {
        let doc = Document::from_lines(vec![Line::label("Static")]);
        let mut core = OutconceiveCore::from_document(doc);
        core.initial_render();

        let patches = core.render_dirty_lines();
        assert!(patches.is_empty());
    }

    #[test]
    fn test_add_line_produces_patches() {
        let doc = Document::from_lines(vec![Line::label("First")]);
        let mut core = OutconceiveCore::from_document(doc);
        core.initial_render();

        let patches = core.add_line(Line::label("Second"));
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_toggle_state() {
        let mut doc = Document::from_lines(vec![
            Line::content_row("check", "CCCCC", "agree", "     "),
        ]);
        doc.state.set_bool("agree", false);

        let mut core = OutconceiveCore::from_document(doc);
        core.initial_render();

        let patches = core.toggle_state("agree");
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_full_login_form() {
        let doc = Document::from_lines(vec![
            Line::container_start("card", Some("shadow:md,padding:16")),
            Line::content_row(
                "Username  ________________  ",
                "LLLLLLLLLLIIIIIIIIIIIIIIIIIII",
                "__________username__________",
                "                            ",
            ),
            Line::content_row(
                "Password  ________________  ",
                "LLLLLLLLLLPPPPPPPPPPPPPPPPPP",
                "__________password__________",
                "                            ",
            ),
            Line::content_row(
                "  Login   ",
                "BBBBBBBBBB",
                "submit____",
                "pppppppppp",
            ),
            Line::container_end("card"),
        ]);

        let mut core = OutconceiveCore::from_document(doc);
        let vdom = core.initial_render();

        let card = &vdom.children()[0];
        assert_eq!(card.children().len(), 3);

        let patches = core.update_state_text("username", "admin");
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_incremental_rendering() {
        let doc = Document::from_lines(vec![
            Line::content_row("________", "IIIIIIII", "field_a_", "        "),
            Line::label("Static text"),
            Line::content_row("________", "IIIIIIII", "field_b_", "        "),
        ]);

        let mut core = OutconceiveCore::from_document(doc);
        core.initial_render();

        let patches = core.update_state_text("field_a", "changed");
        for patch in &patches {
            match patch {
                Patch::SetAttribute { path, .. } | Patch::UpdateText { path, .. } => {
                    assert_eq!(path[0], 0, "should only patch line 0");
                }
                _ => {}
            }
        }
    }
}
