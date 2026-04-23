use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Text(String),
    Number(f64),
    Bool(bool),
    List(Vec<StateValue>),
    Null,
}

impl StateValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            StateValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StateValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn display(&self) -> String {
        match self {
            StateValue::Text(s) => s.clone(),
            StateValue::Number(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            StateValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            StateValue::List(_) => "[...]".to_string(),
            StateValue::Null => "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct StateStore {
    values: HashMap<String, StateValue>,
    dirty_keys: HashSet<String>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            dirty_keys: HashSet::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&StateValue> {
        self.values.get(key)
    }

    pub fn get_text(&self, key: &str) -> String {
        self.values
            .get(key)
            .map(|v| v.display())
            .unwrap_or_default()
    }

    pub fn get_number(&self, key: &str) -> f64 {
        self.values
            .get(key)
            .and_then(|v| v.as_number())
            .unwrap_or(0.0)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.values
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set(&mut self, key: &str, value: StateValue) {
        let changed = self.values.get(key) != Some(&value);
        if changed {
            self.values.insert(key.to_string(), value);
            self.dirty_keys.insert(key.to_string());
        }
    }

    pub fn set_text(&mut self, key: &str, value: &str) {
        self.set(key, StateValue::Text(value.to_string()));
    }

    pub fn set_number(&mut self, key: &str, value: f64) {
        self.set(key, StateValue::Number(value));
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.set(key, StateValue::Bool(value));
    }

    pub fn toggle(&mut self, key: &str) {
        let current = self.get_bool(key);
        self.set_bool(key, !current);
    }

    pub fn set_list_item(&mut self, list_key: &str, index: usize, fields: &[(String, StateValue)]) {
        for (field, value) in fields {
            let key = format!("{}.{}.{}", list_key, index, field);
            self.set(&key, value.clone());
        }
        self.set(&format!("{}._count", list_key),
            StateValue::Number((index + 1).max(self.get_list_count(list_key)) as f64));
        self.dirty_keys.insert(list_key.to_string());
    }

    pub fn add_list_item(&mut self, list_key: &str, fields: &[(String, StateValue)]) {
        let count = self.get_list_count(list_key);
        self.set_list_item(list_key, count, fields);
    }

    pub fn remove_list_item(&mut self, list_key: &str, index: usize) {
        let count = self.get_list_count(list_key);
        if index >= count { return; }

        // Shift items down
        for i in index..count - 1 {
            let next_keys: Vec<(String, StateValue)> = self.values.iter()
                .filter(|(k, _)| k.starts_with(&format!("{}.{}.", list_key, i + 1)))
                .map(|(k, v)| {
                    let suffix = &k[format!("{}.{}.", list_key, i + 1).len()..];
                    (suffix.to_string(), v.clone())
                })
                .collect();

            // Clear current index
            let prefix = format!("{}.{}.", list_key, i);
            let to_remove: Vec<String> = self.values.keys()
                .filter(|k| k.starts_with(&prefix))
                .cloned()
                .collect();
            for k in to_remove {
                self.values.remove(&k);
            }

            // Write shifted values
            for (field, value) in next_keys {
                let key = format!("{}.{}.{}", list_key, i, field);
                self.values.insert(key, value);
            }
        }

        // Remove last index
        let prefix = format!("{}.{}.", list_key, count - 1);
        let to_remove: Vec<String> = self.values.keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in to_remove {
            self.values.remove(&k);
        }

        self.set(&format!("{}._count", list_key),
            StateValue::Number((count - 1) as f64));
        self.dirty_keys.insert(list_key.to_string());
    }

    pub fn get_list_count(&self, list_key: &str) -> usize {
        self.values
            .get(&format!("{}._count", list_key))
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(0)
    }

    pub fn get_scoped(&self, scope: &str, key: &str) -> Option<&StateValue> {
        self.values.get(&format!("{}.{}", scope, key))
    }

    pub fn get_scoped_text(&self, scope: &str, key: &str) -> String {
        self.get_scoped(scope, key)
            .map(|v| v.display())
            .unwrap_or_default()
    }

    pub fn get_scoped_bool(&self, scope: &str, key: &str) -> bool {
        self.get_scoped(scope, key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn take_dirty_keys(&mut self) -> HashSet<String> {
        std::mem::take(&mut self.dirty_keys)
    }

    pub fn has_dirty_keys(&self) -> bool {
        !self.dirty_keys.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_store() {
        let store = StateStore::new();
        assert!(!store.has_dirty_keys());
        assert_eq!(store.get("anything"), None);
    }

    #[test]
    fn test_set_and_get_text() {
        let mut store = StateStore::new();
        store.set_text("name", "Alice");
        assert_eq!(store.get_text("name"), "Alice");
    }

    #[test]
    fn test_set_and_get_number() {
        let mut store = StateStore::new();
        store.set_number("count", 42.0);
        assert_eq!(store.get_number("count"), 42.0);
    }

    #[test]
    fn test_set_and_get_bool() {
        let mut store = StateStore::new();
        store.set_bool("active", true);
        assert_eq!(store.get_bool("active"), true);
    }

    #[test]
    fn test_toggle() {
        let mut store = StateStore::new();
        store.set_bool("flag", false);
        store.toggle("flag");
        assert_eq!(store.get_bool("flag"), true);
        store.toggle("flag");
        assert_eq!(store.get_bool("flag"), false);
    }

    #[test]
    fn test_dirty_tracking() {
        let mut store = StateStore::new();
        assert!(!store.has_dirty_keys());

        store.set_text("name", "Alice");
        assert!(store.has_dirty_keys());

        let dirty = store.take_dirty_keys();
        assert!(dirty.contains("name"));
        assert!(!store.has_dirty_keys());
    }

    #[test]
    fn test_no_dirty_on_same_value() {
        let mut store = StateStore::new();
        store.set_text("name", "Alice");
        store.take_dirty_keys();

        store.set_text("name", "Alice");
        assert!(!store.has_dirty_keys());
    }

    #[test]
    fn test_display() {
        assert_eq!(StateValue::Text("hello".into()).display(), "hello");
        assert_eq!(StateValue::Number(42.0).display(), "42");
        assert_eq!(StateValue::Number(3.14).display(), "3.14");
        assert_eq!(StateValue::Bool(true).display(), "true");
        assert_eq!(StateValue::Null.display(), "");
    }

    #[test]
    fn test_get_missing_key_defaults() {
        let store = StateStore::new();
        assert_eq!(store.get_text("missing"), "");
        assert_eq!(store.get_number("missing"), 0.0);
        assert_eq!(store.get_bool("missing"), false);
    }

    #[test]
    fn test_contains() {
        let mut store = StateStore::new();
        assert!(!store.contains("key"));
        store.set_text("key", "val");
        assert!(store.contains("key"));
    }
}
