use std::collections::{HashMap, HashSet};
use crate::line::Line;
use crate::state::StateStore;

#[derive(Debug)]
pub struct Document {
    pub lines: Vec<Line>,
    pub state: StateStore,
    state_to_lines: HashMap<String, HashSet<usize>>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![Line::new()],
            state: StateStore::new(),
            state_to_lines: HashMap::new(),
        }
    }

    pub fn from_lines(lines: Vec<Line>) -> Self {
        let mut doc = Self {
            lines,
            state: StateStore::new(),
            state_to_lines: HashMap::new(),
        };
        doc.rebuild_state_index();
        doc
    }

    pub fn add_line(&mut self, line: Line) {
        let idx = self.lines.len();
        self.index_line(idx, &line);
        self.lines.push(line);
    }

    pub fn insert_line(&mut self, index: usize, line: Line) {
        self.lines.insert(index, line);
        self.rebuild_state_index();
    }

    pub fn remove_line(&mut self, index: usize) -> Line {
        let line = self.lines.remove(index);
        self.rebuild_state_index();
        line
    }

    pub fn container_start(&mut self, tag: &str, config: Option<&str>) {
        self.add_line(Line::container_start(tag, config));
    }

    pub fn container_end(&mut self, tag: &str) {
        self.add_line(Line::container_end(tag));
    }

    pub fn row(
        &mut self,
        content: &str,
        components: &str,
        state_keys: &str,
        styles: &str,
    ) {
        self.add_line(Line::content_row(content, components, state_keys, styles));
    }

    pub fn label(&mut self, text: &str) {
        self.add_line(Line::label(text));
    }

    pub fn lines_for_state_key(&self, key: &str) -> Vec<usize> {
        self.state_to_lines
            .get(key)
            .map(|set| {
                let mut v: Vec<usize> = set.iter().copied().collect();
                v.sort();
                v
            })
            .unwrap_or_default()
    }

    pub fn dirty_lines(&mut self) -> Vec<usize> {
        let dirty_keys = self.state.take_dirty_keys();
        let mut lines = HashSet::new();
        for key in &dirty_keys {
            if let Some(line_set) = self.state_to_lines.get(key) {
                lines.extend(line_set);
            }
        }
        let mut result: Vec<usize> = lines.into_iter().collect();
        result.sort();
        result
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn rebuild_index(&mut self) {
        self.rebuild_state_index();
    }

    fn rebuild_state_index(&mut self) {
        self.state_to_lines.clear();
        for idx in 0..self.lines.len() {
            let spans = self.lines[idx].spans();
            for span in spans {
                if let Some(ref key) = span.state_key {
                    self.state_to_lines
                        .entry(key.clone())
                        .or_insert_with(HashSet::new)
                        .insert(idx);
                }
            }
        }
    }

    fn index_line(&mut self, idx: usize, line: &Line) {
        for span in line.spans() {
            if let Some(ref key) = span.state_key {
                self.state_to_lines
                    .entry(key.clone())
                    .or_insert_with(HashSet::new)
                    .insert(idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = Document::new();
        assert_eq!(doc.line_count(), 1);
    }

    #[test]
    fn test_from_lines() {
        let doc = Document::from_lines(vec![
            Line::label("Hello"),
            Line::label("World"),
        ]);
        assert_eq!(doc.line_count(), 2);
    }

    #[test]
    fn test_add_line() {
        let mut doc = Document::new();
        doc.add_line(Line::label("Added"));
        assert_eq!(doc.line_count(), 2);
    }

    #[test]
    fn test_insert_line() {
        let mut doc = Document::from_lines(vec![
            Line::label("First"),
            Line::label("Third"),
        ]);
        doc.insert_line(1, Line::label("Second"));
        assert_eq!(doc.line_count(), 3);
        assert_eq!(doc.lines[1].content, "Second");
    }

    #[test]
    fn test_remove_line() {
        let mut doc = Document::from_lines(vec![
            Line::label("A"),
            Line::label("B"),
            Line::label("C"),
        ]);
        let removed = doc.remove_line(1);
        assert_eq!(removed.content, "B");
        assert_eq!(doc.line_count(), 2);
    }

    #[test]
    fn test_state_index() {
        let doc = Document::from_lines(vec![
            Line::content_row("________", "IIIIIIII", "username", "        "),
            Line::label("Hello"),
            Line::content_row("________", "IIIIIIII", "username", "        "),
        ]);

        let lines = doc.lines_for_state_key("username");
        assert_eq!(lines, vec![0, 2]);
    }

    #[test]
    fn test_dirty_lines() {
        let mut doc = Document::from_lines(vec![
            Line::content_row("________", "IIIIIIII", "username", "        "),
            Line::label("Static"),
            Line::content_row("________", "LLLLLLLL", "username", "        "),
        ]);

        doc.state.set_text("username", "Alice");
        let dirty = doc.dirty_lines();
        assert_eq!(dirty, vec![0, 2]);
    }

    #[test]
    fn test_dirty_lines_only_affected() {
        let mut doc = Document::from_lines(vec![
            Line::content_row("________", "IIIIIIII", "username", "        "),
            Line::content_row("________", "IIIIIIII", "password", "        "),
        ]);

        doc.state.set_text("username", "Alice");
        let dirty = doc.dirty_lines();
        assert_eq!(dirty, vec![0]);
    }

    #[test]
    fn test_container_helpers() {
        let mut doc = Document::new();
        doc.container_start("card", Some("shadow:md"));
        doc.label("Inside");
        doc.container_end("card");
        assert_eq!(doc.line_count(), 4);
        assert!(doc.lines[1].is_container_start());
        assert!(doc.lines[3].is_container_end());
    }

    #[test]
    fn test_row_helper() {
        let mut doc = Document::new();
        doc.row("Hello", "LLLLL", "_____", "     ");
        assert_eq!(doc.lines[1].content, "Hello");
    }
}
