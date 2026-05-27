//! Tag index for grouped invalidation of cache entries.

use std::collections::{HashMap, HashSet};

/// Maps tags to sets of keys for group invalidation.
#[derive(Default)]
pub struct TagIndex {
    tag_to_keys: HashMap<String, HashSet<String>>,
}

impl TagIndex {
    /// Registers that `key` belongs to the given `tags`.
    pub fn insert(&mut self, key: &str, tags: &[&str]) {
        for tag in tags {
            self.tag_to_keys
                .entry(tag.to_string())
                .or_default()
                .insert(key.to_string());
        }
    }

    /// Removes `key` from all tags where it appears.
    pub fn remove(&mut self, key: &str) {
        for keys in self.tag_to_keys.values_mut() {
            keys.remove(key);
        }
    }

    /// Returns all keys associated with `tag`.
    pub fn keys_for_tag(&self, tag: &str) -> Vec<String> {
        self.tag_to_keys
            .get(tag)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Removes all tag to key mappings.
    pub fn clear(&mut self) {
        self.tag_to_keys.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_retrieve() {
        let mut idx = TagIndex::default();
        idx.insert("user:1", &["users", "admin"]);
        idx.insert("user:2", &["users"]);
        let mut keys = idx.keys_for_tag("users");
        keys.sort();
        assert_eq!(keys, vec!["user:1", "user:2"]);
    }

    #[test]
    fn remove_clears_key_from_tags() {
        let mut idx = TagIndex::default();
        idx.insert("user:1", &["users"]);
        idx.remove("user:1");
        assert!(idx.keys_for_tag("users").is_empty());
    }

    #[test]
    fn unknown_tag_returns_empty() {
        let idx = TagIndex::default();
        assert!(idx.keys_for_tag("nonexistent").is_empty());
    }

    #[test]
    fn multiple_tags_per_key() {
        let mut idx = TagIndex::default();
        idx.insert("user:1", &["users", "admin"]);
        assert!(idx.keys_for_tag("admin").contains(&"user:1".to_string()));
        assert!(idx.keys_for_tag("users").contains(&"user:1".to_string()));
    }

    #[test]
    fn clear_removes_all_tags() {
        let mut idx = TagIndex::default();
        idx.insert("k1", &["t1"]);
        idx.insert("k2", &["t1", "t2"]);
        idx.clear();
        assert!(idx.keys_for_tag("t1").is_empty());
        assert!(idx.keys_for_tag("t2").is_empty());
    }
}
