use pest::Parser;
use pest_derive::Parser;

use crate::spacing;
use anyhow::Result;
use pest::iterators::Pair;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar/json.pest"]
struct JSONParser;

pub fn process(input: &str) -> Result<String> {
    let r = JSONParser::parse(Rule::json, input)?;
    let mut result: Vec<String> = Vec::new();
    fn parse_value(result: &mut Vec<String>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::object | Rule::array => {
                for p in pair.into_inner() {
                    parse_value(result, p);
                }
            }
            Rule::string => result.push(spacing(pair.as_str())),
            _ => result.push(pair.as_str().to_owned()),
        };
    }
    for pair in r {
        parse_value(&mut result, pair);
    }
    Ok(result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"a": "b", "c": 123, "d": true, "e": null, "f": ["g", 1]}"#;
        let expected = r#"{"a": "b", "c": 123, "d": true, "e": null, "f": ["g", 1]}"#;
        assert_eq!(process(json).unwrap(), expected);

        let json = r#"{"a": "甲b", "c": 123, "d": true, "e": null, "f": ["g", 1]}"#;
        let expected = r#"{"a": "甲 b", "c": 123, "d": true, "e": null, "f": ["g", 1]}"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    #[test]
    fn test_string_key() {
        let json = r#"{"甲a": "b"}"#;
        let expected = r#"{"甲a": "b"}"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    #[test]
    fn test_parse_string_only() {
        let json = r#"{"data": "hello world"}"#;
        let expected = r#"{"data": "hello world"}"#;
        assert_eq!(process(json).unwrap(), expected);

        let json = r#"{"data": "你好"}"#;
        let expected = r#"{"data": "你好"}"#;
        assert_eq!(process(json).unwrap(), expected);

        let json = r#"{"data": "你好world"}"#;
        let expected = r#"{"data": "你好 world"}"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    #[test]
    fn test_parse_array_with_strings() {
        let json = r#"["a", "b", "c"]"#;
        let expected = r#"["a", "b", "c"]"#;
        assert_eq!(process(json).unwrap(), expected);

        let json = r#"["甲", "乙", "丙"]"#;
        let expected = r#"["甲", "乙", "丙"]"#;
        assert_eq!(process(json).unwrap(), expected);

        let json = r#"["甲a", "乙b", "丙c"]"#;
        let expected = r#"["甲 a", "乙 b", "丙 c"]"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    #[test]
    fn test_parse_empty_object() {
        let json = r#"{}"#;
        let expected = r#"{}"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    #[test]
    fn test_parse_empty_array() {
        let json = r#"[]"#;
        let expected = r#"[]"#;
        assert_eq!(process(json).unwrap(), expected);
    }

    // Pest might handle some errors internally. This test checks basic well-formedness.
    #[test]
    fn test_malformed_json_missing_brace() {
        let json = r#"{"a": "b""#; // Missing closing brace
        assert!(process(json).is_err());
    }

    #[test]
    fn test_malformed_json_trailing_comma_object() {
        let json = r#"{"a": "b",}"#; // Trailing comma in object
        // Depending on the pest rule, this might be allowed or rejected.
        // Let's assume it's an error based on standard JSON.
        assert!(process(json).is_err());
    }

    #[test]
    fn test_malformed_json_trailing_comma_array() {
        let json = r#"["a",]"#; // Trailing comma in array
        // Depending on the pest rule, this might be allowed or rejected.
        // Let's assume it's an error based on standard JSON.
        assert!(process(json).is_err());
    }
}
