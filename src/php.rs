// src/php.rs
use anyhow::Result;

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

// Import the HTML processor
use crate::html; // Assuming html.rs provides a public `process` function

#[derive(Parser)]
#[grammar = "grammar/php.pest"]
struct PhpParser;

fn spacing(input: &str) -> String {
    dbg!(&input);
    if let Ok(parsed) = html::process(input) {
        dbg!(&parsed);
        return parsed;
    }
    input.to_string()
}

fn process_string(pair: Pair<Rule>) -> String {
    let mut result = String::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::heredoc_plain_chunk => {
                result.push_str(&spacing(inner_pair.as_str()));
            }
            Rule::nowdoc_body_content => {
                result.push_str(&spacing(inner_pair.as_str()));
            }
            Rule::php_double_quoted_string => {
                result.push('"');
                for dq_inner in inner_pair.into_inner() {
                    match dq_inner.as_rule() {
                        Rule::php_dq_normal_text => {
                            result.push_str(&spacing(dq_inner.as_str()));
                        }
                        _ => {
                            result.push_str(dq_inner.as_str());
                        }
                    }
                }
                result.push('"');
            }
            Rule::php_single_quoted_string => {
                result.push('\'');
                for sq_inner in inner_pair.into_inner() {
                    match sq_inner.as_rule() {
                        Rule::php_sq_normal_text => {
                            result.push_str(&spacing(sq_inner.as_str()));
                        }
                        _ => {
                            result.push_str(sq_inner.as_str());
                        }
                    }
                }
                result.push('\'');
            }
            // variable
            _ => {
                result.push_str(inner_pair.as_str());
            }
        }
    }
    result
}

/// Processes a PHP string, extracting comments and strings,
/// and delegating HTML sections to html::process.
pub fn process(input: &str) -> Result<String> {
    let pairs = PhpParser::parse(Rule::program, input)?;

    let mut result = Vec::new();
dbg!(&pairs);
    // Define a recursive helper function to process pairs
    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>) -> Result<()> {

        match pair.as_rule() {
            // Top-level structure: descend into inner chunks
            Rule::program => {
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() != Rule::EOI {
                        // Skip EOI for cleaner output
                        parse_pair(result, inner_pair)?;
                    }
                }
            }
            // HTML Chunk: Process using html::process
            Rule::html_chunk => {
                let html_str = pair.as_str();
                // Call the HTML processor from the html module
                // Assuming html::process correctly handles HTML fragments
                match html::process(html_str) {
                    Ok(processed_html) => result.push(processed_html),
                    Err(e) => {
                        eprintln!(
                            "Warning: HTML processing failed for chunk: {:?}. Error: {}",
                            html_str, e
                        );
                        // Fallback: push original HTML string if processing fails
                        result.push(html_str.to_string());
                    }
                }
            }
            // PHP Chunk: Process tags and inner body
            Rule::php_chunk => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair)?;
                }
            }
            // Keep PHP tags as they are
            Rule::php_start_tag | Rule::php_end_tag => {
                result.push(pair.as_str().to_string());
            }
            // Explicitly handle the body content matched by php_script_body
            Rule::php_script_body => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair)?;
                }
            }
            // Extract PHP Comments (keeping delimiters for context)
            Rule::php_comment => {
                // You might want different processing here, e.g., just extract content
                result.push(spacing(pair.as_str()));
            }
            Rule::php_string => result.push(process_string(pair)),
            // Catch unhandled rules during development
            _ => {
                result.push(pair.as_str().to_string());
            }
        }
        Ok(())
    }

    // Start processing from the top-level pairs
    for pair in pairs {
        parse_pair(&mut result, pair)?;
    }

    // Join the processed parts without extra spaces
    Ok(result.join(""))
}

// --- Basic Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_only() {
        let input = "<div><p>HTML测试</p></div>";
        // Expect html::process to handle this (assuming it preserves structure)
        let expected = "<div><p>HTML 测试</p></div>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_comment_only() {
        let input = "<?php // Line Comment";
        let expected = "<?php // Line Comment"; // Expect raw extraction
        assert_eq!(process(input).unwrap(), expected);

        let input = "<?php // Line Comment ?>";
        let expected = "<?php // Line Comment ?>"; // Expect raw extraction
        assert_eq!(process(input).unwrap(), expected);

        let input = "<?php /* Block Comment */ ?>";
        let expected = "<?php /* Block Comment */ ?>"; // Expect raw extraction
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_string_only() {
        let input = "<?php 'Single Quoted'; \"Double Quoted \\\" Escaped\"; ?>";
        let expected = "<?php 'Single Quoted'; \"Double Quoted \\\" Escaped\"; ?>"; // Expect raw extraction
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_mixed_content() {
        let input = "<h1>Title</h1><?php echo 'Hello'; // Say hello ?> <p>World</p>";
        // HTML processed, PHP extracted raw
        let expected = "<h1>Title</h1><?php echo 'Hello'; // Say hello ?> <p>World</p>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_inside_html_attribute() {
        // This case highlights limitations - the current grammar treats this as HTML
        // A more advanced parser might need context switching within attributes.
        let input = "<input type=\"text\" value=\"<?php echo $value; ?>\">";
        // Expect html::process to handle the outer tag, PHP part treated as text by html::process
        let expected = "<input type=\"text\" value=\"<?php echo $value; ?>\">";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_code_char() {
        let input = "<?php echo 'Hello'; ?>";
        let expected = "<?php echo 'Hello'; ?>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_html_head() {
        let input = r#"<!DOCTYPE html>
<html>
<head>
<title>PHP语法综合示例</title>
</head>
<body>
<?php
echo 1;
"#;
        let expected = r#"<!DOCTYPE html>
<html>
<head>
<title>PHP 语法综合示例</title>
</head>
<body>
<?php
echo 1;
"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_string() {
        let input = r#"<?php echo "你好$name"; ?>"#;
        let expected = r#"<?php echo "你好$name"; ?>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_heredoc_string() {
        let input = r#"<?php
$str = <<<EOT
你好$name，
hello世界
EOT;
"#;
        let expected = r#"<?php
$str = <<<EOT
你好$name，
hello 世界
EOT;
"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_nowdoc_string() {
        let input = r#"<?php
$str = <<<'EOT'
你好$name，
hello世界
EOT;
"#;
        let expected = r#"<?php
$str = <<<'EOT'
你好 $name，
hello 世界
EOT;
"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_dq_inner() {
        let input = r#"<?php $str = "$pos双引号String"; ?>"#;
        let expected = r#"<?php $str = "$pos双引号 String"; ?>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_php_sq_inner() {
        let input = r#"<?php $str = '单引号String'; ?>"#;
        let expected = r#"<?php $str = '单引号 String'; ?>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_echo() {
        let input = r#"<?php echo "NULL变量:"; ?>"#;
        let expected = r#"<?php echo "NULL 变量:"; ?>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_3() {
        let input = r#"<?php echo $boolVar ? "真<br>" : "假<br>"; ?>"#;
        let expected = r#"<?php echo $boolVar ? "真<br>" : "假<br>"; ?>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_mix() {
        let input = r#"<?php ?>
<p>这是HTML内容，下面是 PHP 输出的内容：</p>"#;
        let expected = r#"<?php ?>
<p>这是 HTML 内容，下面是 PHP 输出的内容：</p>"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_integration() {
        let input = std::fs::read_to_string("test-data/source.php").unwrap();
        let expected = std::fs::read_to_string("test-data/expect.php").unwrap();
        let result = process(&input).unwrap();
        std::fs::write("test-data/.result.php", &result).unwrap();
        assert_eq!(result, expected);
    }
}
