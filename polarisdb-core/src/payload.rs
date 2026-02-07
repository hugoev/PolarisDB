//! Payload storage for vector metadata.
//!
//! Payloads are JSON-like metadata attached to vectors, supporting
//! flexible schema and indexed field access for filtered searches.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Metadata payload attached to a vector.
///
/// Payloads store arbitrary JSON-like data and support indexed field access
/// for efficient filtered searches.
///
/// # Example
///
/// ```
/// use polarisdb_core::Payload;
///
/// let payload = Payload::new()
///     .with_field("category", "documentation")
///     .with_field("year", 2024)
///     .with_field("tags", vec!["rust", "database"]);
///
/// assert_eq!(payload.get_str("category"), Some("documentation"));
/// assert_eq!(payload.get_i64("year"), Some(2024));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    data: HashMap<String, Value>,
}

impl Payload {
    /// Creates a new empty payload.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Creates a payload from a HashMap.
    #[inline]
    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Adds a field to the payload. Chainable.
    pub fn with_field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Sets a field value.
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.data.insert(key.into(), value.into());
    }

    /// Gets a field value by key.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Gets a field as a string.
    #[inline]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }

    /// Gets a field as an i64.
    #[inline]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key).and_then(|v| v.as_i64())
    }

    /// Gets a field as an f64.
    #[inline]
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key).and_then(|v| v.as_f64())
    }

    /// Gets a field as a bool.
    #[inline]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    /// Gets a field as an array.
    #[inline]
    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.data.get(key).and_then(|v| v.as_array())
    }

    /// Removes a field and returns its value if present.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    /// Returns true if the payload contains the given key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the number of fields in the payload.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the payload has no fields.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an iterator over the payload fields.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.data.iter()
    }

    /// Returns the underlying data map.
    #[inline]
    pub fn into_inner(self) -> HashMap<String, Value> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_new() {
        let p = Payload::new();
        assert!(p.is_empty());
    }

    #[test]
    fn test_payload_with_field() {
        let p = Payload::new()
            .with_field("name", "test")
            .with_field("count", 42);

        assert_eq!(p.len(), 2);
        assert_eq!(p.get_str("name"), Some("test"));
        assert_eq!(p.get_i64("count"), Some(42));
    }

    #[test]
    fn test_payload_set_and_get() {
        let mut p = Payload::new();
        p.set("key", "value");
        assert_eq!(p.get_str("key"), Some("value"));
    }

    #[test]
    fn test_payload_get_typed() {
        let p = Payload::new()
            .with_field("str_field", "hello")
            .with_field("int_field", 123)
            .with_field("float_field", 3.14)
            .with_field("bool_field", true);

        assert_eq!(p.get_str("str_field"), Some("hello"));
        assert_eq!(p.get_i64("int_field"), Some(123));
        assert!((p.get_f64("float_field").unwrap() - 3.14).abs() < 1e-10);
        assert_eq!(p.get_bool("bool_field"), Some(true));
    }

    #[test]
    fn test_payload_remove() {
        let mut p = Payload::new().with_field("key", "value");
        assert!(p.contains_key("key"));

        let removed = p.remove("key");
        assert!(removed.is_some());
        assert!(!p.contains_key("key"));
    }

    #[test]
    fn test_payload_serialization() {
        let p = Payload::new()
            .with_field("name", "test")
            .with_field("count", 42);

        let json = serde_json::to_string(&p).unwrap();
        let deserialized: Payload = serde_json::from_str(&json).unwrap();
        assert_eq!(p, deserialized);
    }
}
