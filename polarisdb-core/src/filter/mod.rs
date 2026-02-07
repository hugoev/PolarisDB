//! Filter expressions for metadata-based vector filtering.
//!
//! Filters allow combining vector similarity search with metadata conditions,
//! enabling queries like "find similar vectors where category = 'docs' AND year >= 2024".

pub mod bitmap_index;

pub use bitmap_index::BitmapIndex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::payload::Payload;

/// A filter expression that can be evaluated against a payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub(crate) condition: FilterCondition,
}

impl Filter {
    /// Creates a filter for a specific field.
    ///
    /// # Example
    ///
    /// ```
    /// use polarisdb_core::Filter;
    ///
    /// let filter = Filter::field("category").eq("documents");
    /// ```
    pub fn field(name: &str) -> FieldFilter {
        FieldFilter {
            field_name: name.to_string(),
        }
    }

    /// Creates a filter from a condition.
    pub fn from_condition(condition: FilterCondition) -> Self {
        Self { condition }
    }

    /// Combines this filter with another using AND.
    pub fn and(self, other: Filter) -> Self {
        Self {
            condition: FilterCondition::And(Box::new(self.condition), Box::new(other.condition)),
        }
    }

    /// Combines this filter with another using OR.
    pub fn or(self, other: Filter) -> Self {
        Self {
            condition: FilterCondition::Or(Box::new(self.condition), Box::new(other.condition)),
        }
    }

    /// Negates this filter.
    #[allow(clippy::should_implement_trait)]
    pub fn negate(self) -> Self {
        Self {
            condition: FilterCondition::Not(Box::new(self.condition)),
        }
    }

    /// Evaluates the filter against a payload. Returns true if the payload matches.
    pub fn matches(&self, payload: &Payload) -> bool {
        self.condition.matches(payload)
    }
}

/// Builder for field-specific filter conditions.
#[derive(Debug)]
pub struct FieldFilter {
    field_name: String,
}

impl FieldFilter {
    /// Field equals value.
    pub fn eq<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Eq(self.field_name, value.into()))
    }

    /// Field not equals value.
    pub fn ne<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Ne(self.field_name, value.into()))
    }

    /// Field greater than value.
    pub fn gt<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Gt(self.field_name, value.into()))
    }

    /// Field greater than or equal to value.
    pub fn gte<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Gte(self.field_name, value.into()))
    }

    /// Field less than value.
    pub fn lt<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Lt(self.field_name, value.into()))
    }

    /// Field less than or equal to value.
    pub fn lte<V: Into<Value>>(self, value: V) -> Filter {
        Filter::from_condition(FilterCondition::Lte(self.field_name, value.into()))
    }

    /// Field value is in the given list.
    pub fn contained_in<V: Into<Value>>(self, values: Vec<V>) -> Filter {
        let values: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        Filter::from_condition(FilterCondition::In(self.field_name, values))
    }

    /// Field (as string) contains the given substring.
    pub fn contains(self, substring: &str) -> Filter {
        Filter::from_condition(FilterCondition::Contains(
            self.field_name,
            substring.to_string(),
        ))
    }

    /// Field exists (is not null/missing).
    pub fn exists(self) -> Filter {
        Filter::from_condition(FilterCondition::Exists(self.field_name))
    }
}

/// The actual filter condition variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Field equals value
    Eq(String, Value),
    /// Field not equals value
    Ne(String, Value),
    /// Field greater than value
    Gt(String, Value),
    /// Field greater than or equal to value
    Gte(String, Value),
    /// Field less than value
    Lt(String, Value),
    /// Field less than or equal to value
    Lte(String, Value),
    /// Field value is in list
    In(String, Vec<Value>),
    /// String field contains substring
    Contains(String, String),
    /// Field exists
    Exists(String),
    /// Logical AND
    And(Box<FilterCondition>, Box<FilterCondition>),
    /// Logical OR
    Or(Box<FilterCondition>, Box<FilterCondition>),
    /// Logical NOT
    Not(Box<FilterCondition>),
}

impl FilterCondition {
    /// Evaluates this condition against a payload.
    pub fn matches(&self, payload: &Payload) -> bool {
        match self {
            FilterCondition::Eq(field, value) => {
                payload.get(field).map(|v| v == value).unwrap_or(false)
            }
            FilterCondition::Ne(field, value) => {
                payload.get(field).map(|v| v != value).unwrap_or(true)
            }
            FilterCondition::Gt(field, value) => {
                compare_values(payload.get(field), value, |a, b| a > b)
            }
            FilterCondition::Gte(field, value) => {
                compare_values(payload.get(field), value, |a, b| a >= b)
            }
            FilterCondition::Lt(field, value) => {
                compare_values(payload.get(field), value, |a, b| a < b)
            }
            FilterCondition::Lte(field, value) => {
                compare_values(payload.get(field), value, |a, b| a <= b)
            }
            FilterCondition::In(field, values) => payload
                .get(field)
                .map(|v| values.contains(v))
                .unwrap_or(false),
            FilterCondition::Contains(field, substring) => payload
                .get_str(field)
                .map(|s| s.contains(substring))
                .unwrap_or(false),
            FilterCondition::Exists(field) => payload.contains_key(field),
            FilterCondition::And(a, b) => a.matches(payload) && b.matches(payload),
            FilterCondition::Or(a, b) => a.matches(payload) || b.matches(payload),
            FilterCondition::Not(c) => !c.matches(payload),
        }
    }
}

/// Helper to compare numeric values.
fn compare_values<F>(field_value: Option<&Value>, target: &Value, cmp: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    match (field_value, target) {
        (Some(Value::Number(a)), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(av), Some(bv)) => cmp(av, bv),
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_payload() -> Payload {
        Payload::new()
            .with_field("category", "documents")
            .with_field("year", 2024)
            .with_field("score", 0.95)
            .with_field("active", true)
            .with_field("name", "test document")
    }

    #[test]
    fn test_filter_eq() {
        let p = test_payload();
        let filter = Filter::field("category").eq("documents");
        assert!(filter.matches(&p));

        let filter = Filter::field("category").eq("images");
        assert!(!filter.matches(&p));
    }

    #[test]
    fn test_filter_ne() {
        let p = test_payload();
        let filter = Filter::field("category").ne("images");
        assert!(filter.matches(&p));
    }

    #[test]
    fn test_filter_numeric_comparisons() {
        let p = test_payload();

        assert!(Filter::field("year").gt(2020).matches(&p));
        assert!(Filter::field("year").gte(2024).matches(&p));
        assert!(Filter::field("year").lt(2025).matches(&p));
        assert!(Filter::field("year").lte(2024).matches(&p));
    }

    #[test]
    fn test_filter_in() {
        let p = test_payload();
        let filter = Filter::field("category").contained_in(vec!["documents", "images"]);
        assert!(filter.matches(&p));

        let filter = Filter::field("category").contained_in(vec!["audio", "video"]);
        assert!(!filter.matches(&p));
    }

    #[test]
    fn test_filter_contains() {
        let p = test_payload();
        let filter = Filter::field("name").contains("document");
        assert!(filter.matches(&p));
    }

    #[test]
    fn test_filter_exists() {
        let p = test_payload();
        assert!(Filter::field("category").exists().matches(&p));
        assert!(!Filter::field("nonexistent").exists().matches(&p));
    }

    #[test]
    fn test_filter_and() {
        let p = test_payload();
        let filter = Filter::field("category")
            .eq("documents")
            .and(Filter::field("year").gte(2024));
        assert!(filter.matches(&p));

        let filter = Filter::field("category")
            .eq("documents")
            .and(Filter::field("year").gt(2024));
        assert!(!filter.matches(&p));
    }

    #[test]
    fn test_filter_or() {
        let p = test_payload();
        let filter = Filter::field("category")
            .eq("images")
            .or(Filter::field("year").eq(2024));
        assert!(filter.matches(&p));
    }

    #[test]
    fn test_filter_not() {
        let p = test_payload();
        let filter = Filter::field("category").eq("images").negate();
        assert!(filter.matches(&p));
    }

    #[test]
    fn test_filter_complex() {
        let p = test_payload();
        // (category = 'documents' AND year >= 2024) OR active = true
        let filter = Filter::field("category")
            .eq("documents")
            .and(Filter::field("year").gte(2024))
            .or(Filter::field("active").eq(true));
        assert!(filter.matches(&p));
    }
}
