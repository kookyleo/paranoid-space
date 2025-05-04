use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use anyhow::Result;

use crate::spacing;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar/json5.pest"]
struct JSON5Parser;

pub fn process(input: &str) -> Result<String> {
    let pairs = JSON5Parser::parse(Rule::json, input)?;
    let mut result: Vec<String> = Vec::new();

    // Helper function to recursively process pairs
    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>) {
        match pair.as_rule() {
            // Handle rules that need spacing
            Rule::string => {
                let s = pair.as_str();
                if s.len() >= 2 {
                    let quote = &s[0..1]; // " or '
                    let content = &s[1..s.len() - 1];
                    // Apply spacing to the inner content
                    let spaced_content = spacing(content);
                    // Reconstruct the string with original quotes
                    result.push(format!("{}{}{}", quote, spaced_content, quote));
                } else {
                    // Handle potentially empty or malformed strings
                    result.push(s.to_owned());
                }
            }
            Rule::LINE_COMMENT => {
                let content = pair
                    .into_inner()
                    .map(|p| p.as_str())
                    .collect::<Vec<&str>>()
                    .join("");
                result.push(format!("//{}", spacing(&content)));
            }
            Rule::BLOCK_COMMENT => {
                let content = pair
                    .into_inner()
                    .map(|p| p.as_str())
                    .collect::<Vec<&str>>()
                    .join("");
                result.push(format!("/*{}*/", spacing(&content)));
            }
            // Handle rules that contain nested structures; recurse into them
            Rule::json | Rule::value | Rule::object | Rule::array | Rule::pair | Rule::COMMENT => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair);
                }
            }
            // Catch-all for any unexpected rules encountered
            _ => {
                // For debugging, print unexpected rules
                // println!("Unhandled rule: {:?}", pair.as_rule());
                // Default behavior: append the text representation
                result.push(pair.as_str().to_owned());
            }
        }
    }

    for pair in pairs {
        parse_pair(&mut result, pair);
    }

    Ok(result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let json5 = r#"{
            "key1":"value1",
            "键2": "值2",
            key3: "v3",
        }"#;
        let expected = r#"{
            "key1":"value1",
            "键2": "值 2",
            key3: "v3",
        }"#;
        assert_eq!(process(json5).unwrap(), expected);
    }

    #[test]
    fn test_comments() {
        let json5 = r#"// 注释 before
        {}"#;
        let expected = r#"// 注释 before
        {}"#;
        assert_eq!(process(json5).unwrap(), expected);

        let json5 = r#"/*block comment*/{}"#;
        let expected = r#"/*block comment*/{}"#;
        assert_eq!(process(json5).unwrap(), expected);

        let json5 = r#"/*block注释 */{}"#;
        let expected = r#"/*block 注释 */{}"#;
        assert_eq!(process(json5).unwrap(), expected);
    }

    #[test]
    fn test_spacing_strings_and_comments() {
        let json5 = r#"
        {
            // 注释before
            "key1": "value 1", //注释beside
            "key2": "值2", /* block注释
                             * multi line */
            "key 3": "value3",
            /* another block */"key4": "值 4" //comment最终
        }
        "#;
        // Expected output (without spacing applied)
        let expected = r#"
        {
            // 注释 before
            "key1": "value 1", //注释 beside
            "key2": "值 2", /* block 注释
                             * multi line */
            "key 3": "value3",
            /* another block */"key4": "值 4" //comment 最终
        }
        "#;
        assert_eq!(process(json5).unwrap(), expected);
    }

    #[test]
    fn test_identifiers_and_other_types() {
        let json5 = r#"
         {
             // Using identifiers as keys
             key_iden: "string value",
             anotherKey: 123.45, // Number
             isActive: true, /* Boolean */
             items: [ null, 'single quoted string' ] // Array, Null, Single quotes
         }
         "#;
        // Expected output (without spacing applied)
        let expected = r#"
         {
             // Using identifiers as keys
             key_iden: "string value",
             anotherKey: 123.45, // Number
             isActive: true, /* Boolean */
             items: [ null, 'single quoted string' ] // Array, Null, Single quotes
         }
         "#;
        assert_eq!(process(json5).unwrap(), expected);
    }

    #[test]
    fn test_trailing_commas() {
        let json5 = r#"
         {
             "a": 1,
             "b": "B b", // trailing comma in object
         } // trailing comma allowed by grammar
         "#;
        let expected = r#"
         {
             "a": 1,
             "b": "B b", // trailing comma in object
         } // trailing comma allowed by grammar
         "#;
        assert_eq!(process(json5).unwrap(), expected);

        let json5_array = r#"
         [
             "item 1",
             "项 2", // trailing comma in array
         ]
         "#;
        let expected_array = r#"
         [
             "item 1",
             "项 2", // trailing comma in array
         ]
         "#;
        assert_eq!(process(json5_array).unwrap(), expected_array);
    }

    #[test]
    fn test_empty_structures() {
        assert_eq!(process("{}").unwrap(), "{}");
        assert_eq!(process("[]").unwrap(), "[]");
        // With comments (without spacing)
        assert_eq!(process("{ /* empty */ }").unwrap(), "{ /* empty */ }");
        assert_eq!(
            process(
                "[ // empty
         ]"
            )
            .unwrap(),
            "[ // empty
         ]"
        );
    }

    #[test]
    fn test_malformed_json5() {
        // Example: Missing closing brace
        let json5 = r#"{ "key": "value" "#;
        assert!(process(json5).is_err());

        // Example: Invalid syntax (e.g., colon instead of comma)
        let json5_invalid = r#"{ "a": 1, "b": 2, }"#;
        assert!(!process(json5_invalid).is_err());
    }

    #[test]
    fn test_line_continuation() {
        // 测试 JSON5 中的反斜杠换行特性
        let json5_with_line_continuation = r#"{
            "description": "这是一个很长的描述，\
可以使用反斜杠进行换行"
        }"#;
        let expected_json5 = r#"{
            "description": "这是一个很长的描述，\
可以使用反斜杠进行换行"
        }"#;
        let processed_json5 = process(json5_with_line_continuation).expect("处理带有反斜杠换行的 JSON5 失败");
        assert_eq!(processed_json5, expected_json5);
    }
}
