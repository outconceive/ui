use std::collections::HashMap;
use crate::block;
use crate::component::{self, ComponentSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct MetaLine {
    pub format: char,
    pub level: u8,
    pub tag: Option<String>,
    pub config: Option<String>,
}

impl MetaLine {
    pub fn new() -> Self {
        Self {
            format: block::PLAIN,
            level: 0,
            tag: None,
            config: None,
        }
    }

    pub fn container_start(tag: &str, config: Option<&str>) -> Self {
        Self {
            format: block::CONTAINER_START,
            level: 0,
            tag: Some(tag.to_string()),
            config: config.map(|s| s.to_string()),
        }
    }

    pub fn container_end(tag: &str) -> Self {
        Self {
            format: block::CONTAINER_END,
            level: 0,
            tag: Some(tag.to_string()),
            config: None,
        }
    }

    pub fn row() -> Self {
        Self {
            format: block::ROW,
            level: 0,
            tag: None,
            config: None,
        }
    }

    pub fn each_start(list_key: &str) -> Self {
        Self {
            format: block::EACH_START,
            level: 0,
            tag: Some(list_key.to_string()),
            config: None,
        }
    }

    pub fn each_end() -> Self {
        Self {
            format: block::EACH_END,
            level: 0,
            tag: None,
            config: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub content: String,
    pub components: String,
    pub state_keys: String,
    pub styles: String,
    pub meta: MetaLine,
    pub logic: Option<HashMap<usize, String>>,
    pub cols: Option<HashMap<usize, (u8, u8)>>,
    pub validates: Option<HashMap<usize, String>>,
    pub responsives: Option<HashMap<usize, Vec<(String, u8, u8)>>>,
    pub animates: Option<HashMap<usize, String>>,
    pub popovers: Option<HashMap<usize, String>>,
}

impl Line {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            components: String::new(),
            state_keys: String::new(),
            styles: String::new(),
            meta: MetaLine::new(),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn content_row(
        content: &str,
        components: &str,
        state_keys: &str,
        styles: &str,
    ) -> Self {
        let len = content.chars().count();
        Self {
            content: content.to_string(),
            components: pad_or_trim(components, len, component::EMPTY),
            state_keys: pad_or_trim(state_keys, len, '_'),
            styles: pad_or_trim(styles, len, ' '),
            meta: MetaLine::row(),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn label(text: &str) -> Self {
        let len = text.chars().count();
        Self {
            content: text.to_string(),
            components: std::iter::repeat(component::LABEL).take(len).collect(),
            state_keys: std::iter::repeat('_').take(len).collect(),
            styles: std::iter::repeat(' ').take(len).collect(),
            meta: MetaLine::row(),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn container_start(tag: &str, config: Option<&str>) -> Self {
        Self {
            content: String::new(),
            components: String::new(),
            state_keys: String::new(),
            styles: String::new(),
            meta: MetaLine::container_start(tag, config),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn each_start(list_key: &str) -> Self {
        Self {
            content: String::new(),
            components: String::new(),
            state_keys: String::new(),
            styles: String::new(),
            meta: MetaLine::each_start(list_key),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn each_end() -> Self {
        Self {
            content: String::new(),
            components: String::new(),
            state_keys: String::new(),
            styles: String::new(),
            meta: MetaLine::each_end(),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn is_each_start(&self) -> bool {
        self.meta.format == block::EACH_START
    }

    pub fn is_each_end(&self) -> bool {
        self.meta.format == block::EACH_END
    }

    pub fn container_end(tag: &str) -> Self {
        Self {
            content: String::new(),
            components: String::new(),
            state_keys: String::new(),
            styles: String::new(),
            meta: MetaLine::container_end(tag),
            logic: None,
            cols: None,
            validates: None,
            responsives: None,
            animates: None,
            popovers: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && !block::is_container_boundary(self.meta.format)
    }

    pub fn is_container_start(&self) -> bool {
        self.meta.format == block::CONTAINER_START
    }

    pub fn is_container_end(&self) -> bool {
        self.meta.format == block::CONTAINER_END
    }

    pub fn spans(&self) -> Vec<ComponentSpan> {
        let mut spans = component::group_spans(
            &self.content,
            &self.components,
            &self.state_keys,
            &self.styles,
            &self.logic,
        );
        if let Some(ref cols) = self.cols {
            for span in &mut spans {
                if let Some(&col) = cols.get(&span.start) {
                    span.col = Some(col);
                }
            }
        }
        if let Some(ref validates) = self.validates {
            for span in &mut spans {
                if let Some(v) = validates.get(&span.start) {
                    span.validate = Some(v.clone());
                }
            }
        }
        if let Some(ref responsives) = self.responsives {
            for span in &mut spans {
                if let Some(r) = responsives.get(&span.start) {
                    span.responsive = Some(r.clone());
                }
            }
        }
        if let Some(ref animates) = self.animates {
            for span in &mut spans {
                if let Some(a) = animates.get(&span.start) {
                    span.animate = Some(a.clone());
                }
            }
        }
        if let Some(ref popovers) = self.popovers {
            for span in &mut spans {
                if let Some(p) = popovers.get(&span.start) {
                    span.popover = Some(p.clone());
                }
            }
        }
        spans
    }

    pub fn set_logic(&mut self, position: usize, func_name: &str) {
        self.logic
            .get_or_insert_with(HashMap::new)
            .insert(position, func_name.to_string());
    }

    pub fn set_col(&mut self, position: usize, n: u8, total: u8) {
        self.cols
            .get_or_insert_with(HashMap::new)
            .insert(position, (n, total));
    }

    pub fn width(&self) -> usize {
        self.content.chars().count()
    }
}

fn pad_or_trim(s: &str, target_len: usize, pad_char: char) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= target_len {
        chars[..target_len].iter().collect()
    } else {
        let mut result: String = chars.into_iter().collect();
        result.extend(std::iter::repeat(pad_char).take(target_len - result.chars().count()));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_line() {
        let line = Line::new();
        assert!(line.is_empty());
        assert_eq!(line.meta.format, block::PLAIN);
    }

    #[test]
    fn test_content_row() {
        let line = Line::content_row(
            "Hello",
            "LLLLL",
            "_____",
            "     ",
        );
        assert_eq!(line.content, "Hello");
        assert_eq!(line.components, "LLLLL");
        assert_eq!(line.width(), 5);
    }

    #[test]
    fn test_content_row_pads_short_strings() {
        let line = Line::content_row("Hello", "LL", "_", "");
        assert_eq!(line.components.chars().count(), 5);
        assert_eq!(line.state_keys.chars().count(), 5);
        assert_eq!(line.styles.chars().count(), 5);
    }

    #[test]
    fn test_label() {
        let line = Line::label("Username");
        assert_eq!(line.content, "Username");
        assert!(line.components.chars().all(|c| c == component::LABEL));
        assert_eq!(line.width(), 8);
    }

    #[test]
    fn test_container_start() {
        let line = Line::container_start("card", Some("shadow:md"));
        assert!(line.is_container_start());
        assert!(!line.is_container_end());
        assert_eq!(line.meta.tag, Some("card".to_string()));
        assert_eq!(line.meta.config, Some("shadow:md".to_string()));
    }

    #[test]
    fn test_container_end() {
        let line = Line::container_end("card");
        assert!(line.is_container_end());
        assert!(!line.is_container_start());
        assert_eq!(line.meta.tag, Some("card".to_string()));
    }

    #[test]
    fn test_spans() {
        let line = Line::content_row(
            "Name  ______",
            "LLLLLLIIIIIII",
            "______name___",
            "             ",
        );
        let spans = line.spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].component, component::LABEL);
        assert_eq!(spans[1].component, component::TEXT_INPUT);
    }

    #[test]
    fn test_set_logic() {
        let mut line = Line::content_row("Go", "BB", "do", "pp");
        line.set_logic(0, "submit_handler");
        assert_eq!(
            line.logic.as_ref().unwrap().get(&0),
            Some(&"submit_handler".to_string())
        );
    }

    #[test]
    fn test_pad_or_trim() {
        assert_eq!(pad_or_trim("AB", 5, '_'), "AB___");
        assert_eq!(pad_or_trim("ABCDE", 3, '_'), "ABC");
        assert_eq!(pad_or_trim("ABC", 3, '_'), "ABC");
    }
}
