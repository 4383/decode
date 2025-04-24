#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::engine;
    use serde_json::json;

    #[test]
    fn test_simple_field_access() {
        let json = json!({
            "name": "John Doe",
            "age": 30,
            "isActive": true
        });
        
        let query = parser::parse_query("$.name").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!("John Doe"));
    }

    #[test]
    fn test_array_index_access() {
        let json = json!({
            "items": [10, 20, 30, 40, 50]
        });
        
        let query = parser::parse_query("$.items[2]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!(30));
    }

    #[test]
    fn test_negative_array_index() {
        let json = json!({
            "items": [10, 20, 30, 40, 50]
        });
        
        let query = parser::parse_query("$.items[-1]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!(50));
    }

    #[test]
    fn test_multi_index_access() {
        let json = json!({
            "items": [10, 20, 30, 40, 50]
        });
        
        let query = parser::parse_query("$.items[0,2,4]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!([10, 30, 50]));
    }

    #[test]
    fn test_filter_expression() {
        let json = json!({
            "users": [
                {"name": "Alice", "age": 25, "active": true},
                {"name": "Bob", "age": 30, "active": false},
                {"name": "Charlie", "age": 35, "active": true}
            ]
        });
        
        let query = parser::parse_query("$.users[?(@.age > 28)]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!([
            {"name": "Bob", "age": 30, "active": false},
            {"name": "Charlie", "age": 35, "active": true}
        ]));
    }

    #[test]
    fn test_recursive_wildcard() {
        let json = json!({
            "store": {
                "book": [
                    {
                        "title": "The Great Gatsby",
                        "price": 9.99
                    },
                    {
                        "title": "Moby Dick",
                        "price": 12.99
                    }
                ],
                "bicycle": {
                    "color": "red",
                    "price": 199.99
                }
            }
        });
        
        let query = parser::parse_query("$.store[*]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        // Should return an array containing the book array and bicycle object
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_recursive_descent() {
        let json = json!({
            "store": {
                "book": [
                    {
                        "title": "The Great Gatsby",
                        "price": 9.99
                    },
                    {
                        "title": "Moby Dick",
                        "price": 12.99
                    }
                ]
            }
        });
        
        let query = parser::parse_query("$..title").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        let expected = json!(["The Great Gatsby", "Moby Dick"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chained_access() {
        let json = json!({
            "users": [
                {"name": {"first": "John", "last": "Doe"}, "age": 30},
                {"name": {"first": "Jane", "last": "Smith"}, "age": 25}
            ]
        });
        
        let query = parser::parse_query("$.users[1].name.first").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result, json!("Jane"));
    }

    #[test]
    fn test_null_comparison() {
        let json = json!({
            "users": [
                {"name": "Alice", "email": "alice@example.com"},
                {"name": "Bob", "email": null},
                {"name": "Charlie"}
            ]
        });
        
        let query = parser::parse_query("$.users[?(@.email == null)]").unwrap();
        let result = engine::apply_query(&json, &query).unwrap();
        
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result.as_array().unwrap()[0]["name"], "Bob");
    }
}