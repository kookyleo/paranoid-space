extern crate pest;

use pest::Parser;
use anyhow::{Result};
use crate::spacing; // Import the spacing function

#[derive(Parser)]
#[grammar = "grammar/markdown.pest"]
struct MarkdownParser;

// Main processing function
pub fn process(text: &str) -> Result<String> {
    match MarkdownParser::parse(Rule::document, text) {
        Ok(pairs) => {
            // Process the parsed document, pair by pair
            Ok(pairs.map(process_pair).collect::<String>())
        }
        Err(e) => {
            // On parsing error, log it and return the original text
            eprintln!("Markdown Parse failed: {:?}", e);
            Err(e.into())
        }
    }
}

// Recursive function to process parsed pairs, reconstructing Markdown
fn process_pair(pair: pest::iterators::Pair<Rule>) -> String {
    match pair.as_rule() {
        // Structural Rules: Recursively process inner content
        Rule::document | Rule::paragraph | Rule::link_text | Rule::image_alt => {
            pair.into_inner().map(process_pair).collect()
        }

        // Basic Content Rules:
        Rule::text => spacing(pair.as_str()), // Apply spacing to raw text segments
        Rule::WHITESPACE => pair.as_str().to_string(), // Preserve whitespace
        Rule::link_url | Rule::image_url | Rule::language => pair.as_str().to_string(),
        Rule::NEWLINE => "\n".to_string(),
        Rule::EOI => "".to_string(),

        // --- Block Elements Reconstruction ---
        Rule::heading => {
            let mut inner = pair.into_inner();
            let marker = inner.next().unwrap().as_str(); // # marker
            let content = inner.map(process_pair).collect::<String>();
            format!("{}{}\n", marker, content) // Ensure newline after heading
        }
        Rule::code_block => {
            let mut inner = pair.into_inner();
            let lang_node = inner.next().unwrap();
            let lang = if lang_node.as_rule() == Rule::language { 
                lang_node.as_str() 
            } else {
                "" // No language specified
            };
            // The rest should be code_content or NEWLINEs, get the raw string
            let code = inner.map(|p| p.as_str()).collect::<String>(); 
            if lang.is_empty() {
                 format!("```\n{}\n```\n", code.trim_end())
            } else {
                 format!("```{}\n{}\n```\n", lang, code.trim_end()) // Corrected escape and newline
            }
        }
        Rule::blockquote => {
             let mut inner = pair.into_inner();
             let marker = inner.next().unwrap().as_str(); // > marker
             let content = inner.map(process_pair).collect::<String>();
             format!("{}{}\n", marker, content) // Ensure newline
        }
        Rule::list_item => {
             let mut inner = pair.into_inner();
             let marker = inner.next().unwrap().as_str(); // list marker (*, -, 1., etc.)
             let content = inner.map(process_pair).collect::<String>();
             format!("{}{}\n", marker, content)
        }
         Rule::task_item => {
            let mut inner = pair.into_inner();
            let marker = inner.next().map(|p| p.as_str()).unwrap_or("[ ]"); // Safer unwrap
            let content = inner.map(process_pair).collect::<String>();
            format!("- {} {}\n", marker, content)
        }
        Rule::horizontal_rule => format!("{}\n", pair.as_str()),

        // --- Inline Elements Reconstruction ---
        Rule::inline_code => {
            let code_content = pair.as_str()
                                .trim_start_matches('`')
                                .trim_end_matches('`');
            format!("`{}`", code_content)
        },
        Rule::strong => format!("**{}**", pair.into_inner().map(process_pair).collect::<String>()),
        Rule::emphasis => format!("*{}*", pair.into_inner().map(process_pair).collect::<String>()), // Using '*' for simplicity
        Rule::link => {
            let pair_str = pair.as_str(); // Get string representation first
            let mut inner = pair.into_inner();
            if let (Some(text_pair), Some(url_pair)) = (inner.next(), inner.next()) {
                let text = spacing(text_pair.as_str()); // Apply spacing to link text
                let url = url_pair.as_str();
                format!("[{}]({})", text, url)
            } else {
                // Grammar matched link, but inner structure is wrong? Return raw string.
                eprintln!("Warning: Malformed link inner structure: {}", pair_str);
                pair_str.to_string()
            }
        }
        Rule::image => {
            let pair_str = pair.as_str(); // Get string representation first
            let mut inner = pair.into_inner();
            if let (Some(alt_text_pair), Some(url_pair)) = (inner.next(), inner.next()) {
                let alt_text = spacing(alt_text_pair.as_str()); // Apply spacing to alt text
                let url = url_pair.as_str();
                format!("![{}]({})", alt_text, url)
            } else {
                // Grammar matched image, but inner structure is wrong? Return raw string.
                eprintln!("Warning: Malformed image inner structure: {}", pair_str);
                pair_str.to_string()
            }
        }

        // Fallback for unhandled rules (should ideally be exhaustive)
        // Print a warning and the raw text for debugging
        _ => {
             eprintln!("Warning: Unhandled rule {:?} in process_pair: {}", pair.as_rule(), pair.as_str());
             pair.as_str().to_string() // Return raw string as fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests will be restored here later
    #[test]
    fn test_new_line(){
        let result = process("\nThis is a block doc comment\nThis is another line\n");
        assert_eq!(result.unwrap(), "\nThis is a block doc comment\nThis is another line\n");
    }

    #[test]
    fn test_spacing_markdown() {
        let input = r#"
## Example Title
This is `inline code` example.

`another code`

Link: [Example](http://example.com)

- List item 1
- List item 2

*   Task item 1
*   Task item 2 [x]

```rust
fn main() {
    println!("Hello, world!");
}
```

![Alt text](/path/to/image.png)
"#;
        let expected = r#"
## Example Title
This is `inline code` example.

`another code`

Link: [Example](http://example.com)

- List item 1
- List item 2

*   Task item 1
*   Task item 2 [x]

```rust
fn main() {
    println!("Hello, world!");
}
```

![Alt text](/path/to/image.png)
"#;
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test() {
        let input = r#"
/// Summary
///
/// # Example示例
///
/// ```rust
/// let x = 1;
/// ```

///
/// ## Subsection
///
/// Link: [link](https://example.com)
/// Image: ![alt替代内容](/img.png)
/// Code: `code`
/// List:
/// - item1项
/// - item2
"#;
        let expected = r#"
/// Summary
///
/// # Example 示例
///
/// ```rust
/// let x = 1;
/// ```

///
/// ## Subsection
///
/// Link: [link](https://example.com)
/// Image: ![alt 替代内容](/img.png)
/// Code: `code`
/// List:
/// - item1 项
/// - item2
"#;

        let actual = process(input).unwrap();
        assert_eq!(actual, expected); // Use the stored 'actual'
    }
}