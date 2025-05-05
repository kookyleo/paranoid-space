use crate::markdown;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    LineOuter,  // ///
    LineInner,  // //!
    BlockOuter, // /** */
    BlockInner, // /*! */
}

#[derive(Debug)]
pub struct DocCommentBlock {
    style: CommentStyle,
    raw_lines: Vec<String>,
}

impl DocCommentBlock {
    pub fn new(style: CommentStyle, raw_lines: Vec<String>) -> Self {
        Self { style, raw_lines }
    }

    pub(in crate::rust::doc_comments) fn split_prefix_and_content(&self) -> (Vec<String>, Vec<String>) {
        let prefix_pattern = match self.style {
            CommentStyle::LineOuter => r"^\s*/// ?",
            CommentStyle::LineInner => r"^\s*//! ?",
            CommentStyle::BlockOuter | CommentStyle::BlockInner => r"^\s*\* ?",
        };
        let prefix_re = Regex::new(prefix_pattern).expect("Invalid prefix regex pattern");

        let mut prefixes: Vec<String> = Vec::new();
        let mut contents: Vec<String> = Vec::new();

        for line in &self.raw_lines {
            if let Some(mat) = prefix_re.find(line) {
                prefixes.push(mat.as_str().to_string());
                contents.push(line[mat.end()..].to_string());
            } else {
                prefixes.push("".to_string());
                contents.push(line.clone());
            }
        }

        (prefixes, contents)
    }

    pub fn spacing(&self) -> String {
        if self.raw_lines.is_empty() {
            return "".to_string();
        }

        // Call the helper method to get prefixes and contents
        let (prefixes, contents) = self.split_prefix_and_content();

        // Join contents, process with markdown, and split back into lines
        let content_block = contents.join("");
        let spaced_content_block = markdown::process(&content_block).unwrap_or(content_block);
        let spaced_content_lines: Vec<String> = spaced_content_block
            .lines()
            .map(|s| s.to_string())
            .collect();

        // Reconstruct the final lines using original prefixes and spaced content
        let mut result_lines = Vec::new();
        let num_prefixes = prefixes.len(); // Use the number of original lines/prefixes

        for i in 0..num_prefixes {
             // Safely get prefix (should always exist)
             let prefix = prefixes.get(i).cloned().unwrap_or_else(|| "".to_string());
             // Safely get corresponding spaced line (might not exist if markdown changed line count)
            let spaced_line = spaced_content_lines.get(i).cloned().unwrap_or_else(|| "".to_string());

            result_lines.push(format!("{}{}", prefix, spaced_line));
        }

        let mut result = result_lines.join("\n");
        // Check if the original input had a trailing newline on the last line
        if let Some(last_line) = self.raw_lines.last() {
            if !last_line.is_empty() && last_line.ends_with('\n') {
                 // Append the newline only if the joined result doesn't already end with one
                 // (markdown::process might potentially add one in some cases)
                 if !result.ends_with('\n') {
                     result.push('\n');
                 }
            }
         }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::{Rule, RustParser};
    
    use pest::Parser;

    #[test]
    fn test_line_outer_parse() {
        let source = r#"/// this is a comment line
/// this is another comment line
/// this is yet another comment line"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 2);
        let comment = inner.into_iter().next().unwrap(); // comment.as_rule() = comment
        let inner = comment.into_inner();
        let block = inner.into_iter().next().unwrap(); // block.as_rule() = line_doc_comment_block
        let inner = block.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert!(lines.len() == 3);
        assert!(lines[0].as_rule() == Rule::line_outer_doc_comment);
        assert!(lines[0].as_str() == "/// this is a comment line\n");
        assert!(lines[1].as_rule() == Rule::line_outer_doc_comment);
        assert!(lines[1].as_str() == "/// this is another comment line\n");
        assert!(lines[2].as_rule() == Rule::line_outer_doc_comment);
        assert!(lines[2].as_str() == "/// this is yet another comment line");
    }

    #[test]
    fn test_line_outer_parse2() {
        let source = r#"/// this is a comment line
/// this is another comment line

/// this is yet another comment line"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 4);
        let inner_rules = inner.clone().map(|p| p.as_rule()).collect::<Vec<_>>();
        assert_eq!(
            inner_rules,
            vec![Rule::comment, Rule::WHITESPACE, Rule::comment, Rule::EOI]
        );
        let inner = inner.into_iter().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::comment);
        let inner = inner.into_inner().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::line_outer_doc_comment_block);
        let inner = inner.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_rule(), Rule::line_outer_doc_comment);
        assert_eq!(lines[0].as_str(), "/// this is a comment line\n");
        assert_eq!(lines[1].as_rule(), Rule::line_outer_doc_comment);
        assert_eq!(lines[1].as_str(), "/// this is another comment line\n");
    }

    #[test]
    fn test_line_outer_parse3() {
        let source = r#"/// this is a comment line
        /// this is another comment line"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        let inner_rules = inner.clone().map(|p| p.as_rule()).collect::<Vec<_>>();
        assert_eq!(
            inner_rules,
            vec![
                Rule::comment,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::WHITESPACE,
                Rule::comment,
                Rule::EOI
            ]
        );
        let inner = inner.into_iter().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::comment);
        let inner = inner.into_inner().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::line_outer_doc_comment_block);
        let inner = inner.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_rule(), Rule::line_outer_doc_comment);
        assert_eq!(lines[0].as_str(), "/// this is a comment line\n");
    }

    #[test]
    fn test_line_inner_parse() {
        let source = r#"//! this is a comment line
//! this is another comment line
//! this is yet another comment line"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 2);
        let comment = inner.into_iter().next().unwrap(); // comment.as_rule() = comment
        let inner = comment.into_inner();
        let block = inner.into_iter().next().unwrap(); // block.as_rule() = line_doc_comment_block
        let inner = block.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert!(lines.len() == 3);
        assert!(lines[0].as_rule() == Rule::line_inner_doc_comment);
        assert!(lines[0].as_str() == "//! this is a comment line\n");
        assert!(lines[1].as_rule() == Rule::line_inner_doc_comment);
        assert!(lines[1].as_str() == "//! this is another comment line\n");
        assert!(lines[2].as_rule() == Rule::line_inner_doc_comment);
        assert!(lines[2].as_str() == "//! this is yet another comment line");
    }

    #[test]
    fn test_line_inner_parse2() {
        let source = r#"//! this is a comment line
//! this is another comment line

//! this is yet another comment line"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 4);
        let inner_rules = inner.clone().map(|p| p.as_rule()).collect::<Vec<_>>();
        assert_eq!(
            inner_rules,
            vec![Rule::comment, Rule::WHITESPACE, Rule::comment, Rule::EOI]
        );
        let inner = inner.into_iter().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::comment);
        let inner = inner.into_inner().next().unwrap();
        assert_eq!(inner.as_rule(), Rule::line_inner_doc_comment_block);
        let inner = inner.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_rule(), Rule::line_inner_doc_comment);
        assert!(lines[0].as_str() == "//! this is a comment line\n");
        assert_eq!(lines[1].as_rule(), Rule::line_inner_doc_comment);
        assert!(lines[1].as_str() == "//! this is another comment line\n");
    }

    #[test]
    fn test_block_outer_parse() {
        let source = r#"/**
 * this is a comment line
 * this is another comment line
 */"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 2);
        let comment = inner.into_iter().next().unwrap(); // comment.as_rule() = comment
        let inner = comment.into_inner();
        let block = inner.into_iter().next().unwrap(); // block.as_rule() = block_outer_doc_comment
        let inner = block.into_inner(); // Vec[block_doc_comment_inner_line]
        let lines: Vec<_> = inner.into_iter().collect();
        assert!(lines.len() == 4);
        assert!(lines[0].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[0].as_str() == "\n");
        assert!(lines[1].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[1].as_str() == " * this is a comment line\n");
        assert!(lines[2].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[2].as_str() == " * this is another comment line\n");
        assert!(lines[3].as_rule() == Rule::block_doc_comment_last_line);
        assert!(lines[3].as_str() == " ");
    }

    #[test]
    fn test_block_outer_parse2() {
        let source = r#"/** this is not a good example of a comment
 * this is a comment line
this is another comment line
*/"#;

        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 2);
        let comment = inner.into_iter().next().unwrap(); // comment.as_rule() = comment
        let inner = comment.into_inner();
        let block = inner.into_iter().next().unwrap(); // block.as_rule() = block_doc_comment
        let inner = block.into_inner(); // Vec[block_doc_comment_inner_line]
        let lines: Vec<_> = inner.into_iter().collect();
        assert!(lines.len() == 4);
        assert!(lines[0].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[0].as_str() == " this is not a good example of a comment\n");
        assert!(lines[1].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[1].as_str() == " * this is a comment line\n");
        assert!(lines[2].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[2].as_str() == "this is another comment line\n");
        assert!(lines[3].as_rule() == Rule::block_doc_comment_last_line);
        assert!(lines[3].as_str() == "");
    }

    #[test]
    fn test_block_inner_parse() {
        let source = r#"/*!
 * this is a comment line
 * this is another comment line
 */"#;
        let r = RustParser::parse(Rule::program, source).unwrap();
        let pair = r.into_iter().next().unwrap(); // pair.as_rule() = program
        let inner = pair.into_inner();
        assert_eq!(inner.len(), 2);
        let comment = inner.into_iter().next().unwrap(); // comment.as_rule() = comment
        let inner = comment.into_inner();
        let block = inner.into_iter().next().unwrap(); // block.as_rule() = block_doc_comment
        let inner = block.into_inner();
        let lines: Vec<_> = inner.into_iter().collect();
        assert!(lines.len() == 4);
        assert!(lines[0].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[0].as_str() == "\n");
        assert!(lines[1].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[1].as_str() == " * this is a comment line\n");
        assert!(lines[2].as_rule() == Rule::block_doc_comment_inner_line);
        assert!(lines[2].as_str() == " * this is another comment line\n");
        assert!(lines[3].as_rule() == Rule::block_doc_comment_last_line);
        assert!(lines[3].as_str() == " ");
    }

    #[test]
    fn test_split_prefix_and_content() {
        // line outter
        let block = DocCommentBlock::new(
            CommentStyle::LineOuter,
            vec!["/// This is a comment\n".to_string(), "/// This is another comment".to_string()],
        );
        let (prefixes, contents) = block.split_prefix_and_content();
        assert_eq!(prefixes, vec!["/// ".to_string(), "/// ".to_string()]);
        assert_eq!(contents, vec!["This is a comment\n".to_string(), "This is another comment".to_string()]);

        // line inner
        let block = DocCommentBlock::new(
            CommentStyle::LineInner,
            vec!["//! This is a comment\n".to_string(), "//! This is another comment".to_string()],
        );
        let (prefixes, contents) = block.split_prefix_and_content();
        assert_eq!(prefixes, vec!["//! ".to_string(), "//! ".to_string()]);
        assert_eq!(contents, vec!["This is a comment\n".to_string(), "This is another comment".to_string()]);

        // block outer
        let block = DocCommentBlock::new(
            CommentStyle::BlockOuter,
            vec![" * This is a comment\n".to_string(), " * This is another comment\n".to_string()],
        );
        let (prefixes, contents) = block.split_prefix_and_content();
        assert_eq!(prefixes, vec![" * ".to_string(), " * ".to_string()]);
        assert_eq!(contents, vec!["This is a comment\n".to_string(), "This is another comment\n".to_string()]);


        // block outer2
        // /** this is not a good example of a comment
        //  * this is a comment line
        //  this is another comment line
        //  */
        let block = DocCommentBlock::new(
            CommentStyle::BlockOuter,
            vec![" this is not a good example of a comment\n".to_string(), " * this is a comment line\n".to_string(), "this is another comment line\n".to_string()],
        );
        let (prefixes, contents) = block.split_prefix_and_content();
        assert_eq!(prefixes, vec!["".to_string(), " * ".to_string(), "".to_string()]);
        assert_eq!(contents, vec![" this is not a good example of a comment\n".to_string(), "this is a comment line\n".to_string(), "this is another comment line\n".to_string()]);


        // block inner
        let block = DocCommentBlock::new(
            CommentStyle::BlockInner,
            vec![" * This is a comment\n".to_string(), " * This is another comment\n".to_string()],
        );
        let (prefixes, contents) = block.split_prefix_and_content();
        assert_eq!(prefixes, vec![" * ".to_string(), " * ".to_string()]);
        assert_eq!(contents, vec!["This is a comment\n".to_string(), "This is another comment\n".to_string()]);
    }

    #[test]
    fn test_spacing() {
        let block = DocCommentBlock::new(
            CommentStyle::LineOuter,
            vec!["/// This is一条注释\n".to_string(), "/// This is another comment\n".to_string()],
        );
        let result = block.spacing();
        assert_eq!(result, "/// This is 一条注释\n/// This is another comment\n");
    }
}
