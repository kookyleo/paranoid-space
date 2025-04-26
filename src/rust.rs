use pest::Parser;
use pest_derive::Parser;

use crate::spacing;
use anyhow::Result;
use pest::iterators::Pair;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar/rust.pest"]
struct RustParser;

pub fn process(text: &str) -> Result<String> {
    let r = RustParser::parse(Rule::rust_token, text)?;
    let mut result: Vec<String> = Vec::new();
    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::string => result.push(spacing(pair.as_str())),
            Rule::comment => result.push(spacing(pair.as_str())),
            Rule::block_comment => result.push(spacing(pair.as_str())),
            Rule::line_comment => result.push(spacing(pair.as_str())),
            Rule::line_doc_comment => result.push(spacing(pair.as_str())),
            Rule::block_doc_comment => result.push(spacing(pair.as_str())),
            _ => result.push(pair.as_str().to_owned()),
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
    fn test_rust_spacing() {
        let text = "let x = 1; // 这是一个注释";
        let result = process(text).unwrap();
        assert_eq!(result, "let x = 1; // 这是一个注释");
    }
}
