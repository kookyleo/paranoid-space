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

/// Processes a PHP string, extracting comments and strings,
/// and delegating HTML sections to html::process.
pub fn process(input: &str) -> Result<String> {
    let pairs = PhpParser::parse(Rule::program, input)?;

    let mut result = Vec::new();

    // Define a recursive helper function to process pairs
    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>) -> Result<()> {
        match pair.as_rule() {
            // Top-level structure: descend into inner chunks
            Rule::program => {
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() != Rule::EOI { // Skip EOI for cleaner output
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
                        eprintln!("Warning: HTML processing failed for chunk: {:?}. Error: {}", html_str, e);
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
            Rule::php_comment | Rule::php_string => {
                // You might want different processing here, e.g., just extract content
                result.push(pair.as_str().to_string());
            }
            Rule::php_code_char => { 
                result.push(pair.as_str().to_string());
            }
            // Handle WHITESPACE now that it's not silent
            Rule::WHITESPACE => {
                result.push(pair.as_str().to_string());
            }
            // Ignore EOI (handled in program rule)
            Rule::EOI => {}
            // Catch unhandled rules during development
             _ => {
                 // This case should ideally not be reached if the grammar is complete
                 // for the parsed rules. Pushing the string is a safe fallback.
                 println!("Warning: Unhandled PHP rule: {:?}, pushing raw string.", pair.as_rule());
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
    fn test_html_only() -> Result<()> {
        let input = "<div><p>Just HTML</p></div>";
        // Expect html::process to handle this (assuming it preserves structure)
        let expected = "<div><p>Just HTML</p></div>";
        assert_eq!(process(input)?, expected);
        Ok(())
    }

    #[test]
    fn test_php_comment_only() -> Result<()> {
        let input = "<?php // Line Comment ?>";
        let expected = "<?php // Line Comment ?>"; // Expect raw extraction
        assert_eq!(process(input)?, expected);

        let input2 = "<?php /* Block Comment */ ?>";
        let expected2 = "<?php /* Block Comment */ ?>"; // Expect raw extraction
         assert_eq!(process(input2)?, expected2);
        Ok(())
    }

     #[test]
    fn test_php_string_only() -> Result<()> {
        let input = "<?php 'Single Quoted'; \"Double Quoted \\\" Escaped\"; ?>";
        let expected = "<?php 'Single Quoted'; \"Double Quoted \\\" Escaped\"; ?>"; // Expect raw extraction
        assert_eq!(process(input)?, expected);
        Ok(())
    }

    #[test]
    fn test_mixed_content() -> Result<()> {
        let input = "<h1>Title</h1><?php echo 'Hello'; // Say hello ?> <p>World</p>";
        // HTML processed, PHP extracted raw
        let expected = "<h1>Title</h1><?php echo 'Hello'; // Say hello ?> <p>World</p>";
        assert_eq!(process(input)?, expected);
        Ok(())
    }

    #[test]
    fn test_php_inside_html_attribute() -> Result<()> {
        // This case highlights limitations - the current grammar treats this as HTML
        // A more advanced parser might need context switching within attributes.
        let input = "<input type=\"text\" value=\"<?php echo $value; ?>\">";
         // Expect html::process to handle the outer tag, PHP part treated as text by html::process
        let expected = "<input type=\"text\" value=\"<?php echo $value; ?>\">";
        assert_eq!(process(input)?, expected);
        Ok(())
    }
}
