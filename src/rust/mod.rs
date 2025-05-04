mod doc_comments;

use pest::Parser;
use pest_derive::Parser;

use crate::{rust::doc_comments::*, spacing};
use anyhow::Result;
use pest::iterators::Pair;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar/rust.pest"]
struct RustParser;

pub fn process(input: &str) -> Result<String> {
    let r = RustParser::parse(Rule::program, input)?;
    let mut result: Vec<String> = Vec::new();

    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::line_outer_doc_comment_block => {
                let raw_lines: Vec<String> =
                    pair.into_inner().map(|p| p.as_str().to_string()).collect();
                let block = DocCommentBlock::new(CommentStyle::LineOuter, raw_lines);
                let spaced_content = block.spacing();
                result.push(spaced_content);
            }
            Rule::line_inner_doc_comment_block => {
                let raw_lines: Vec<String> =
                    pair.into_inner().map(|p| p.as_str().to_string()).collect();
                let block = DocCommentBlock::new(CommentStyle::LineInner, raw_lines);
                let spaced_content = block.spacing();
                result.push(spaced_content);
            }
            Rule::block_outer_doc_comment => {
                let inner_pairs = pair.into_inner();
                let raw_lines: Vec<String> = inner_pairs.map(|p| p.as_str().to_string()).collect();

                let block = DocCommentBlock::new(CommentStyle::BlockOuter, raw_lines);
                let spaced_inner_content = block.spacing();

                let formatted_comment = format!("{}{}{}", "/**", spaced_inner_content, "*/");
                result.push(formatted_comment);
            }
            Rule::block_inner_doc_comment => {
                let inner_pairs = pair.into_inner();

                let raw_lines: Vec<String> = inner_pairs.map(|p| p.as_str().to_string()).collect();

                let block = DocCommentBlock::new(CommentStyle::BlockInner, raw_lines);
                let spaced_inner_content = block.spacing();

                let formatted_comment = format!("{}{}{}", "/*!", spaced_inner_content, "*/");
                result.push(formatted_comment);
            }
            Rule::line_comment | Rule::block_comment => {
                result.push(spacing(pair.as_str()));
            }
            Rule::comment => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair);
                }
            }
            Rule::string => {
                result.push(spacing(pair.as_str()));
            }
            Rule::program => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair);
                }
            }
            _ => {
                result.push(pair.as_str().to_owned());
            }
        }
    }
    for pair in r {
        parse_pair(&mut result, pair);
    }

    Ok(result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_basic() {
        let text = "let x = 1; // 这是一个comment";
        let result = process(text).unwrap();
        assert_eq!(result, "let x = 1; // 这是一个 comment");

        let text = "let x = 1; //This is a comment";
        let result = process(text).unwrap();
        assert_eq!(result, "let x = 1; //This is a comment");
    }

    #[test]
    fn test_block_comment() {
        let text = r#"/*This is a block comment*/"#;
        let result = process(text).unwrap();
        assert_eq!(result, r#"/*This is a block comment*/"#);

        let text = r#"/*This is a block comment
        /*This is a nested block comment*/
        */"#;
        let result = process(text).unwrap();
        assert_eq!(
            result,
            r#"/*This is a block comment
        /*This is a nested block comment*/
        */"#
        );

        let text = r#"/*This is a 块注释
        This is另一行
        */"#;
        let result = process(text).unwrap();
        assert_eq!(
            result,
            r#"/*This is a 块注释
        This is 另一行
        */"#
        );
    }

    #[test]
    fn test_block_doc_comment() {
        let text = r#"
/**
 * This is a block doc comment
 * This is another line
*/
"#;
        let result = process(text).unwrap();
        assert_eq!(
            result,
            r#"
/**
 * This is a block doc comment
 * This is another line
*/
"#
        );
    }

    #[test]
    fn test_block_inner_doc_comment() {
        let input = "/*!
Inner comment中文
Another line 行*/";
        let expected = "/*!
Inner comment 中文
Another line 行*/";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_line_doc_comment_block() {
        let input = "/// Line doc comment中文
/// Another line 行
/// ```rust
/// let x = 1;
/// ```";
        let expected = "/// Line doc comment 中文
/// Another line 行
/// ```rust
/// let x = 1;
/// ```";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_multi_line_doc_comment() {
        let input = r#"/** Outer block doc comment
 * 函数的块文档注释
 */"#;
        let expected = r#"/** Outer block doc comment
 * 函数的块文档注释
 */"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_line_inner_doc_comment_block() {
        let input = "//! Inner line doc comment中文\n//! Another line 行";
        let expected = "//! Inner line doc comment 中文\n//! Another line 行";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_raw_string() {
        let input = r##"let s = r#"Raw string content中文 with "quotes" and \escapes"#;"##;
        let expected = r##"let s = r#"Raw string content 中文 with "quotes" and \escapes"#;"##;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_string_with_escapes() {
        let input = r#"let msg = "Hello\n world世界\t!";"#;
        let expected = r#"let msg = "Hello\n world 世界\t!";"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_byte_string() {
        let input = r#"let bytes = b"byte string内容";"#;
        let expected = r#"let bytes = b"byte string 内容";"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_raw_byte_string() {
        let input = r##"let raw_bytes = br#"raw byte内容"#;"##;
        let expected = r##"let raw_bytes = br#"raw byte 内容"#;"##;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_escapes() {
        let input = r#"let message = "Hello, world!你好，世界！ Contains escapes: \n\t\""; // Regular string 普通字符串"#;
        let expected = r#"let message = "Hello, world! 你好，世界！ Contains escapes: \n\t\""; // Regular string 普通字符串"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_integrated() {
        let source = std::fs::read_to_string("test-data/source.rs").unwrap();
        let expected = std::fs::read_to_string("test-data/expect.rs").unwrap();
        let result = process(&source).unwrap();
        // std::fs::write("test-data/.result.rs", result.clone()).unwrap();

        assert_eq!(result, expected);
    }
}
