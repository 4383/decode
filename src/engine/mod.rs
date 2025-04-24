use anyhow::{Context, Result};
use serde_json::Value;
use crate::parser::{Query, PathSegment, FilterExpression, ComparisonOperator, LiteralValue};

/// Apply a query to a JSON value and return the resulting JSON
pub fn apply_query(json: &Value, query: &Query) -> Result<Value> {
    // Start with the root JSON value
    let mut result = json.clone();
    
    // Apply each path segment in the main path
    for segment in &query.path_segments {
        result = apply_path_segment(&result, segment)?;
    }
    
    // Apply recursive paths if any
    if !query.recursive_paths.is_empty() {
        let mut recursive_results = Vec::new();
        
        // For each recursive path defined
        for recursive_path in &query.recursive_paths {
            // Apply recursive search starting from current result
            collect_recursive(&result, recursive_path, &mut recursive_results)?;
        }
        
        // If we found any results, replace the current result with the collection
        if !recursive_results.is_empty() {
            result = Value::Array(recursive_results);
        }
    }
    
    Ok(result)
}

/// Recursively collect values that match a given path pattern
fn collect_recursive(json: &Value, path_segments: &[PathSegment], results: &mut Vec<Value>) -> Result<()> {
    match json {
        Value::Object(obj) => {
            // For each property in the object
            for (key, value) in obj {
                // If this property matches the field we're looking for
                if path_segments.len() == 1 {
                    if let PathSegment::Field(field_name) = &path_segments[0] {
                        // Check if this key matches the field name
                        if key == field_name {
                            results.push(value.clone());
                        }
                    }
                }
                
                // Then recursively search this property
                collect_recursive(value, path_segments, results)?;
            }
        },
        Value::Array(arr) => {
            // For each item in the array
            for item in arr {
                // Recursively search this item
                collect_recursive(item, path_segments, results)?;
            }
        },
        _ => {}
    }
    
    Ok(())
}

/// Apply a single path segment to a JSON value
fn apply_path_segment(json: &Value, segment: &PathSegment) -> Result<Value> {
    match segment {
        PathSegment::Field(name) => {
            if let Value::Object(obj) = json {
                obj.get(name)
                   .cloned()
                   .context(format!("Field '{}' not found", name))
            } else {
                Err(anyhow::anyhow!("Cannot access field on non-object value"))
            }
        },
        PathSegment::Index(idx) => {
            if let Value::Array(arr) = json {
                let idx = if *idx < 0 {
                    // Handle negative indices (counting from the end)
                    (arr.len() as i64 + idx) as usize
                } else {
                    *idx as usize
                };
                
                arr.get(idx)
                   .cloned()
                   .context(format!("Index {} out of bounds", idx))
            } else {
                Err(anyhow::anyhow!("Cannot access index on non-array value"))
            }
        },
        PathSegment::MultiIndex(indices) => {
            // Create a new array with the selected indices
            if let Value::Array(arr) = json {
                let mut result = Vec::new();
                
                for &idx in indices {
                    let usize_idx = if idx < 0 {
                        // Handle negative indices (counting from the end)
                        (arr.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };
                    
                    if let Some(value) = arr.get(usize_idx) {
                        result.push(value.clone());
                    }
                }
                
                Ok(Value::Array(result))
            } else {
                Err(anyhow::anyhow!("Cannot access indices on non-array value"))
            }
        },
        PathSegment::Filter(filter_expr) => {
            // Apply filter expression
            match json {
                Value::Array(arr) => {
                    let mut result = Vec::new();
                    for item in arr {
                        if evaluate_filter(item, filter_expr)? {
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::Array(result))
                },
                _ => Err(anyhow::anyhow!("Cannot filter non-array value"))
            }
        },
        PathSegment::RecursiveWildcard => {
            // For recursive wildcard [*], we need to collect all elements in an array
            match json {
                Value::Array(arr) => {
                    // Return a copy of the entire array
                    Ok(Value::Array(arr.clone()))
                },
                Value::Object(obj) => {
                    // Collect all field values into an array
                    let values: Vec<Value> = obj.values().cloned().collect();
                    Ok(Value::Array(values))
                },
                _ => Err(anyhow::anyhow!("Cannot apply wildcard to primitive value"))
            }
        }
    }
}

/// Evaluate a filter expression against a JSON value
fn evaluate_filter(json: &Value, filter: &FilterExpression) -> Result<bool> {
    // Extract the value at the path specified in the filter
    let mut current = json.clone();
    
    // Try to apply each path segment
    for segment in &filter.path {
        match apply_path_segment(&current, segment) {
            Ok(value) => current = value,
            Err(_) => {
                // Field doesn't exist - when checking for null equality,
                // missing fields should NOT be treated the same as explicit nulls
                return Ok(false);
            }
        }
    }
    
    // Compare the value with the filter literal
    match (&current, &filter.operator, &filter.value) {
        // String comparisons
        (Value::String(s), ComparisonOperator::Equal, LiteralValue::String(val)) => Ok(s == val),
        (Value::String(s), ComparisonOperator::NotEqual, LiteralValue::String(val)) => Ok(s != val),
        
        // Number comparisons
        (Value::Number(n), ComparisonOperator::Equal, LiteralValue::Integer(val)) => {
            // Convert both to f64 for proper float comparison
            if let Some(num) = n.as_f64() {
                Ok((num - *val as f64).abs() < f64::EPSILON)
            } else {
                Ok(false)
            }
        },
        (Value::Number(n), ComparisonOperator::NotEqual, LiteralValue::Integer(val)) => {
            if let Some(num) = n.as_f64() {
                Ok((num - *val as f64).abs() > f64::EPSILON)
            } else {
                Ok(true)
            }
        },
        (Value::Number(n), ComparisonOperator::GreaterThan, LiteralValue::Integer(val)) => {
            if let Some(num) = n.as_f64() {
                Ok(num > *val as f64)
            } else {
                Ok(false)
            }
        },
        (Value::Number(n), ComparisonOperator::GreaterThanOrEqual, LiteralValue::Integer(val)) => {
            if let Some(num) = n.as_f64() {
                Ok(num >= *val as f64)
            } else {
                Ok(false)
            }
        },
        (Value::Number(n), ComparisonOperator::LessThan, LiteralValue::Integer(val)) => {
            if let Some(num) = n.as_f64() {
                Ok(num < *val as f64)
            } else {
                Ok(false)
            }
        },
        (Value::Number(n), ComparisonOperator::LessThanOrEqual, LiteralValue::Integer(val)) => {
            if let Some(num) = n.as_f64() {
                Ok(num <= *val as f64)
            } else {
                Ok(false)
            }
        },
        
        // Boolean comparisons
        (Value::Bool(b), ComparisonOperator::Equal, LiteralValue::Boolean(val)) => Ok(b == val),
        (Value::Bool(b), ComparisonOperator::NotEqual, LiteralValue::Boolean(val)) => Ok(b != val),
        
        // Null comparisons
        (Value::Null, ComparisonOperator::Equal, LiteralValue::Null) => Ok(true),
        (Value::Null, ComparisonOperator::NotEqual, LiteralValue::Null) => Ok(false),
        (_, ComparisonOperator::Equal, LiteralValue::Null) => Ok(false),
        (_, ComparisonOperator::NotEqual, LiteralValue::Null) => Ok(true),
        
        // Other combinations are not supported
        _ => Err(anyhow::anyhow!("Unsupported comparison: {:?} {:?} {:?}", current, filter.operator, filter.value)),
    }
}