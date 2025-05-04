use pest::Parser;
use pest_derive::Parser;
use crate::spacing;
use anyhow::Result;

#[derive(Parser)]
#[grammar = "grammar/css.pest"]
pub struct CssParser;

pub fn process(input: &str) -> Result<String> {
    let pairs = CssParser::parse(Rule::css, input)?;
    let mut result: Vec<String> = Vec::new();

    if let Some(css_pair) = pairs.peek() {
        if css_pair.as_rule() == Rule::css {
            let inner_pairs = css_pair.clone().into_inner();

            for pair in inner_pairs {
                match pair.as_rule() {
                    Rule::COMMENT => {
                        let s = pair.as_str();
                        if s.len() >= 4 {
                            let content = &s[2..s.len() - 2];
                            let spaced_content = spacing(content);
                            result.push(format!("/*{}*/", spaced_content));
                        } else {
                            result.push(s.to_owned());
                        }
                    }
                    Rule::STRING => {
                        let s = pair.as_str();
                        if s.len() >= 2 {
                            let quote = &s[0..1];
                            let content = &s[1..s.len() - 1];
                            let spaced_content = spacing(content);
                            result.push(format!("{}{}{}", quote, spaced_content, quote));
                        } else {
                            result.push(s.to_owned());
                        }
                    }
                    Rule::WHITESPACE | Rule::ELSE_CONTENT => {
                        result.push(pair.as_str().to_owned());
                    }
                    Rule::EOI => {
                        // Do nothing, EOI is handled implicitly by loop end
                    }
                    _ => { /* Rule::css should not appear here */ }
                }
            }
        } else {
            return Err(anyhow::anyhow!("Expected top-level rule to be 'css', found {:?}", css_pair.as_rule()));
        }
    }

    Ok(result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_css_file() {
        let css_content = std::fs::read_to_string("test-data/source.css")
            .expect("Should have been able to read the source file");
        let expected_css = std::fs::read_to_string("test-data/expect.css")
            .expect("Should have been able to read the expect file");

        // Test the new processing function
        let processed_css = process(&css_content).expect("Processing failed");

        // write back to file
        // std::fs::write("test-data/.result.css", processed_css.clone())
        //     .expect("Should have been able to write the processed file");

        assert_eq!(processed_css, expected_css);
    }
    
    #[test]
    fn test_line_continuation() {
        // 测试 CSS 中的反斜杠换行特性
        let css_with_line_continuation = r#"body {
    content: "这是一个长字符串，\
可以用反斜杠换行";
}
"#;
        let expected_css = r#"body {
    content: "这是一个长字符串，\
可以用反斜杠换行";
}
"#;
        let processed_css = process(css_with_line_continuation).expect("处理带有反斜杠换行的 CSS 失败");
        assert_eq!(processed_css, expected_css);
    }
}