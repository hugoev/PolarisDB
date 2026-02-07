//! Bitmap-based payload index for efficient pre-filtering.
//!
//! Uses roaring bitmaps to track which vector IDs match specific field values,
//! enabling O(1) set operations for filter evaluation.

use std::collections::HashMap;

use roaring::RoaringBitmap;
use serde_json::Value;

use crate::filter::{Filter, FilterCondition};
use crate::payload::Payload;

/// A bitmap-based index for fast payload filtering.
///
/// Maintains inverted indexes from field values to vector IDs using
/// roaring bitmaps, which are highly compressed for sparse ID sets.
///
/// # Example
///
/// ```
/// use polarisdb_core::{BitmapIndex, Filter, Payload};
///
/// let mut index = BitmapIndex::new();
///
/// // Index some payloads
/// index.insert(1, &Payload::new().with_field("category", "electronics"));
/// index.insert(2, &Payload::new().with_field("category", "books"));
/// index.insert(3, &Payload::new().with_field("category", "electronics"));
///
/// // Query for matching IDs
/// let filter = Filter::field("category").eq("electronics");
/// let matches = index.query(&filter);
///
/// assert!(matches.contains(1));
/// assert!(matches.contains(3));
/// assert!(!matches.contains(2));
/// ```
#[derive(Debug, Default)]
pub struct BitmapIndex {
    /// field_name -> field_value (as string) -> bitmap of vector IDs
    field_indexes: HashMap<String, HashMap<String, RoaringBitmap>>,
    /// Tracks all indexed IDs (for NOT operations)
    all_ids: RoaringBitmap,
}

impl BitmapIndex {
    /// Creates a new empty bitmap index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of indexed vectors.
    pub fn len(&self) -> u64 {
        self.all_ids.len()
    }

    /// Returns true if no vectors are indexed.
    pub fn is_empty(&self) -> bool {
        self.all_ids.is_empty()
    }

    /// Indexes a vector's payload.
    pub fn insert(&mut self, id: u64, payload: &Payload) {
        self.all_ids.insert(id as u32);

        for (field, value) in payload.iter() {
            let value_str = value_to_string(value);
            self.field_indexes
                .entry(field.clone())
                .or_default()
                .entry(value_str)
                .or_default()
                .insert(id as u32);
        }
    }

    /// Removes a vector's payload from the index.
    pub fn delete(&mut self, id: u64, payload: &Payload) {
        self.all_ids.remove(id as u32);

        for (field, value) in payload.iter() {
            let value_str = value_to_string(value);
            if let Some(field_map) = self.field_indexes.get_mut(field) {
                if let Some(bitmap) = field_map.get_mut(&value_str) {
                    bitmap.remove(id as u32);
                }
            }
        }
    }

    /// Evaluates a filter and returns matching vector IDs as a bitmap.
    pub fn query(&self, filter: &Filter) -> RoaringBitmap {
        self.eval_condition(&filter.condition)
    }

    /// Evaluates a filter condition recursively.
    fn eval_condition(&self, condition: &FilterCondition) -> RoaringBitmap {
        match condition {
            FilterCondition::Eq(field, value) => self.get_bitmap(field, &value_to_string(value)),
            FilterCondition::Ne(field, value) => {
                let eq_bitmap = self.get_bitmap(field, &value_to_string(value));
                &self.all_ids - &eq_bitmap
            }
            FilterCondition::In(field, values) => {
                let mut result = RoaringBitmap::new();
                for value in values {
                    result |= self.get_bitmap(field, &value_to_string(value));
                }
                result
            }
            FilterCondition::Gt(field, value) | FilterCondition::Gte(field, value) => {
                self.range_query(field, value, condition)
            }
            FilterCondition::Lt(field, value) | FilterCondition::Lte(field, value) => {
                self.range_query(field, value, condition)
            }
            FilterCondition::Contains(field, _value) => {
                // Contains is for substring matching - fallback to checking all values
                if let Some(field_map) = self.field_indexes.get(field) {
                    field_map
                        .values()
                        .fold(RoaringBitmap::new(), |acc, bm| &acc | bm)
                } else {
                    RoaringBitmap::new()
                }
            }
            FilterCondition::Exists(field) => {
                // Union of all values for this field
                if let Some(field_map) = self.field_indexes.get(field) {
                    field_map
                        .values()
                        .fold(RoaringBitmap::new(), |acc, bm| &acc | bm)
                } else {
                    RoaringBitmap::new()
                }
            }
            FilterCondition::And(left, right) => {
                self.eval_condition(left) & self.eval_condition(right)
            }
            FilterCondition::Or(left, right) => {
                self.eval_condition(left) | self.eval_condition(right)
            }
            FilterCondition::Not(inner) => {
                let inner_result = self.eval_condition(inner);
                &self.all_ids - &inner_result
            }
        }
    }

    /// Gets the bitmap for a specific field/value pair.
    fn get_bitmap(&self, field: &str, value: &str) -> RoaringBitmap {
        self.field_indexes
            .get(field)
            .and_then(|m| m.get(value))
            .cloned()
            .unwrap_or_default()
    }

    /// Evaluates range queries (gt, gte, lt, lte).
    fn range_query(
        &self,
        field: &str,
        value: &Value,
        condition: &FilterCondition,
    ) -> RoaringBitmap {
        let Some(field_map) = self.field_indexes.get(field) else {
            return RoaringBitmap::new();
        };

        let target = value.as_f64().unwrap_or(0.0);
        let mut result = RoaringBitmap::new();

        for (stored_val, bitmap) in field_map {
            if let Ok(stored_num) = stored_val.parse::<f64>() {
                let matches = match condition {
                    FilterCondition::Gt(..) => stored_num > target,
                    FilterCondition::Gte(..) => stored_num >= target,
                    FilterCondition::Lt(..) => stored_num < target,
                    FilterCondition::Lte(..) => stored_num <= target,
                    _ => false,
                };
                if matches {
                    result |= bitmap;
                }
            }
        }

        result
    }

    /// Clears all indexed data.
    pub fn clear(&mut self) {
        self.field_indexes.clear();
        self.all_ids.clear();
    }
}

/// Converts a JSON value to a string for indexing.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query_eq() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("category", "electronics"));
        index.insert(2, &Payload::new().with_field("category", "books"));
        index.insert(3, &Payload::new().with_field("category", "electronics"));

        let filter = Filter::field("category").eq("electronics");
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(1));
        assert!(matches.contains(3));
        assert!(!matches.contains(2));
    }

    #[test]
    fn test_query_ne() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("status", "active"));
        index.insert(2, &Payload::new().with_field("status", "inactive"));
        index.insert(3, &Payload::new().with_field("status", "active"));

        let filter = Filter::field("status").ne("active");
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 1);
        assert!(matches.contains(2));
    }

    #[test]
    fn test_query_in() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("color", "red"));
        index.insert(2, &Payload::new().with_field("color", "blue"));
        index.insert(3, &Payload::new().with_field("color", "green"));

        let filter = Filter::field("color").contained_in(vec!["red", "blue"]);
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(1));
        assert!(matches.contains(2));
    }

    #[test]
    fn test_query_and() {
        let mut index = BitmapIndex::new();

        index.insert(
            1,
            &Payload::new()
                .with_field("category", "electronics")
                .with_field("brand", "sony"),
        );
        index.insert(
            2,
            &Payload::new()
                .with_field("category", "electronics")
                .with_field("brand", "lg"),
        );
        index.insert(
            3,
            &Payload::new()
                .with_field("category", "books")
                .with_field("brand", "sony"),
        );

        let filter = Filter::field("category")
            .eq("electronics")
            .and(Filter::field("brand").eq("sony"));
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 1);
        assert!(matches.contains(1));
    }

    #[test]
    fn test_query_or() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("category", "electronics"));
        index.insert(2, &Payload::new().with_field("category", "books"));
        index.insert(3, &Payload::new().with_field("category", "clothing"));

        let filter = Filter::field("category")
            .eq("electronics")
            .or(Filter::field("category").eq("books"));
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(1));
        assert!(matches.contains(2));
    }

    #[test]
    fn test_query_not() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("active", true));
        index.insert(2, &Payload::new().with_field("active", false));

        let filter = Filter::field("active").eq(true).negate();
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 1);
        assert!(matches.contains(2));
    }

    #[test]
    fn test_delete() {
        let mut index = BitmapIndex::new();

        let payload = Payload::new().with_field("category", "electronics");
        index.insert(1, &payload);
        index.insert(2, &payload);

        assert_eq!(index.len(), 2);

        index.delete(1, &payload);
        assert_eq!(index.len(), 1);

        let filter = Filter::field("category").eq("electronics");
        let matches = index.query(&filter);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains(2));
    }

    #[test]
    fn test_numeric_range() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("price", 10));
        index.insert(2, &Payload::new().with_field("price", 25));
        index.insert(3, &Payload::new().with_field("price", 50));

        let filter = Filter::field("price").gt(20);
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(2));
        assert!(matches.contains(3));
    }

    #[test]
    fn test_exists() {
        let mut index = BitmapIndex::new();

        index.insert(1, &Payload::new().with_field("name", "test"));
        index.insert(2, &Payload::new());

        let filter = Filter::field("name").exists();
        let matches = index.query(&filter);

        assert_eq!(matches.len(), 1);
        assert!(matches.contains(1));
    }
}
