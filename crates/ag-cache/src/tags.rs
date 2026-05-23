//! Indice de tags para invalidacion agrupada de entradas de cache.

use std::collections::{HashMap, HashSet};

/// Mapea tags a conjuntos de keys para invalidacion por grupo.
#[derive(Default)]
pub struct TagIndex {
    tag_to_keys: HashMap<String, HashSet<String>>,
}

impl TagIndex {
    /// Registra que `key` pertenece a los `tags` dados.
    pub fn insert(&mut self, key: &str, tags: &[&str]) {
        for tag in tags {
            self.tag_to_keys
                .entry(tag.to_string())
                .or_default()
                .insert(key.to_string());
        }
    }

    /// Elimina `key` de todos los tags donde aparezca.
    pub fn remove(&mut self, key: &str) {
        for keys in self.tag_to_keys.values_mut() {
            keys.remove(key);
        }
    }

    /// Retorna todos los keys asociados a `tag`.
    pub fn keys_for_tag(&self, tag: &str) -> Vec<String> {
        self.tag_to_keys
            .get(tag)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
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
}
