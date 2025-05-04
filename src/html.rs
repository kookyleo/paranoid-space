// src/html.rs
use anyhow::Result;
use pest::Parser;
use pest::iterators::Pair;
// 导入spacing函数和其他处理函数
use crate::css;
use crate::js;
use crate::spacing;

#[derive(Parser)]
#[grammar = "grammar/html.pest"] // 相对于src的路径
pub struct HtmlParser;

/// process string or text
fn process_text(pair: Pair<Rule>) -> String {
    let mut result = String::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::single_quoted_string | Rule::double_quoted_string => {
                result.push_str(&spacing(inner_pair.as_str()));
            }
            _ => {
                result.push_str(inner_pair.as_str());
            }
        }
    }

    result
}

/// process attribute part, eg. class="...", id="..."
fn process_attribute(pair: Pair<Rule>) -> String {
    let mut result = String::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::attribute_value => {
                let mut inner = inner_pair.into_inner();
                let value = inner.next().expect("Attribute must have a value");
                result.push_str(&process_text(value));
            }
            _ => {
                result.push_str(inner_pair.as_str());
            }
        }
    }

    result
}

/// process tag part, eg. <img src="" />, <div class="..." />
fn process_tag(pair: Pair<Rule>) -> String {
    let mut result = String::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tag_name => {
                result.push_str(inner_pair.as_str());
            }
            Rule::attribute => {
                result.push_str(&process_attribute(inner_pair));
            }
            _ => {
                result.push_str(inner_pair.as_str());
            }
        }
    }

    result
}

/// process void element part, eg. <img src="" />
fn process_void_element(pair: Pair<Rule>) -> String {
    let mut result = String::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::attribute => {
                result.push_str(&process_attribute(inner_pair));
            }
            _ => {
                result.push_str(inner_pair.as_str());
            }
        }
    }

    result
}

/// HTML处理函数，使用pest解析HTML并应用spacing
pub fn process(input: &str) -> Result<String> {
    let pairs = match HtmlParser::parse(Rule::html, input) {
        Ok(p) => p,
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut result = Vec::new();

    fn parse_pair(result: &mut Vec<String>, pair: Pair<Rule>, input: &str) {
        match pair.as_rule() {
            // 1. Recursive descent for structure rules:
            Rule::html | Rule::content | Rule::element => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair, input);
                }
            }
            // 2. Apply spacing to specific content rules:
            Rule::COMMENT => {
                result.push("<!--".to_string());
                let inner = pair
                    .into_inner()
                    .into_iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>();
                let new_html = process(&inner.join(""));
                result.push(new_html.unwrap());
                result.push("-->".to_string());
            }
            Rule::text => {
                let spaced_text = spacing(pair.as_str());
                result.push(spaced_text);
            }
            Rule::html_entity => result.push(pair.as_str().to_string()),
            // 3. Reconstruct tags/attributes without internal spacing (but spacing attribute values):
            Rule::left_tag => result.push(process_tag(pair)),
            Rule::right_tag => result.push(pair.as_str().to_string()),
            // Handle void elements (like <input>)
            Rule::void_element => result.push(process_void_element(pair)),
            Rule::script_tag => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::script_content => {
                            result.push(js::process(inner_pair.as_str()).unwrap())
                        }
                        _ => result.push(inner_pair.as_str().to_string()),
                    }
                }
            }
            Rule::style_tag => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::style_content => {
                            result.push(css::process(inner_pair.as_str()).unwrap())
                        }
                        _ => result.push(inner_pair.as_str().to_string()),
                    }
                }
            }
            _ => {
                result.push(pair.as_str().to_string());
            }
        }
    }

    for pair in pairs {
        parse_pair(&mut result, pair, input);
    }

    Ok(result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrative() {
        let input = std::fs::read_to_string("test-data/source.html").unwrap();
        let expect = std::fs::read_to_string("test-data/expect.html").unwrap();
        let result = process(&input).unwrap();
        std::fs::write("test-data/.result.html", result.clone()).unwrap();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_html_in_comment() {
        let input = "<!-- <p>段落Paragraph</p> -->";
        let expected = "<!-- <p>段落 Paragraph</p> -->";
        let r = process(input);
        assert_eq!(r.unwrap(), expected);
    }

    #[test]
    fn test_html_comments() {
        // 测试单行注释
        let input = "<!-- 这是注释Comment -->";
        let expected = "<!-- 这是注释 Comment -->";
        assert_eq!(process(input).unwrap(), expected);

        // 测试多行注释
        let input = "<!-- \n  多行注释Multi\n  Line Comment\n-->";
        let expected = "<!-- \n  多行注释 Multi\n  Line Comment\n-->";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_html_strings() {
        // 测试双引号字符串
        let input = "<a title=\"这是标题Title\">链接Link</a>";
        let expected = "<a title=\"这是标题 Title\">链接 Link</a>";
        assert_eq!(process(input).unwrap(), expected);

        // 测试单引号字符串
        let input = "<input value='这是值Value'>";
        let expected = "<input value='这是值 Value'>";
        assert_eq!(process(input).unwrap(), expected);

        // 测试包含特殊字符的字符串
        let input = "<div title=\"包含'单引号'和\\反斜杠\">内容Content</div>";
        let expected = "<div title=\"包含'单引号'和\\反斜杠\">内容 Content</div>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_script_content() {
        // 测试JavaScript内容处理
        let input = "<script>// 这是注释Comment\nlet x = '这是字符串String';</script>";
        let expected = "<script>// 这是注释 Comment\nlet x = '这是字符串 String';</script>";
        assert_eq!(process(input).unwrap(), expected);

        // 测试带有属性的script标签
        let input = "<script type=\"text/javascript\">var name = \"用户User\";</script>";
        let expected = "<script type=\"text/javascript\">var name = \"用户 User\";</script>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_style_content() {
        // 测试CSS内容处理
        let input = "<style>/* 这是注释Comment */\n.class { font-family: '微软雅黑Font'; }</style>";
        let expected =
            "<style>/* 这是注释 Comment */\n.class { font-family: '微软雅黑 Font'; }</style>";
        assert_eq!(process(input).unwrap(), expected);

        // 测试带有属性的style标签
        let input = "<style type=\"text/css\">.title { color: red; /* 红色Red */ }</style>";
        let expected = "<style type=\"text/css\">.title { color: red; /* 红色 Red */ }</style>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_html_entities() {
        // 测试HTML实体保持不变
        let input = "版权&copy;所有";
        let expected = "版权&copy;所有";
        assert_eq!(process(input).unwrap(), expected);

        let input = "&lt;div&gt;这是内容Content&lt;/div&gt;";
        let expected = "&lt;div&gt;这是内容 Content&lt;/div&gt;";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_simple_tag() {
        let input = "<div class='paragraph' id='p1'>段落Paragraph</div>";
        let expected = "<div class='paragraph' id='p1'>段落 Paragraph</div>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_nested_tags() {
        // 测试嵌套标签
        let input = "<div><p>段落Paragraph <strong>强调Emphasis</strong></p></div>";
        let expected = "<div><p>段落 Paragraph <strong>强调 Emphasis</strong></p></div>";
        assert_eq!(process(input).unwrap(), expected);

        // 测试复杂嵌套结构
        let input = "<div class=\"container\"><header><h1>标题Title</h1></header><main>内容Content</main></div>";
        let expected = "<div class=\"container\"><header><h1>标题 Title</h1></header><main>内容 Content</main></div>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_mixed_content() {
        // 测试混合内容（注释、标签、文本）
        let input = "<!-- 注释Comment --><div>文本Text</div><!-- 另一个注释Another -->";
        let expected = "<!-- 注释 Comment --><div>文本 Text</div><!-- 另一个注释 Another -->";
        assert_eq!(process(input).unwrap(), expected);

        // 测试包含script和style的混合内容
        let input = "<div>前文Text</div><style>.cls{/*注释Comment*/}</style><div>中间Middle</div><script>var x='变量Var';</script><div>后文Text</div>";
        let expected = "<div>前文 Text</div><style>.cls{/*注释 Comment*/}</style><div>中间 Middle</div><script>var x='变量 Var';</script><div>后文 Text</div>";
        assert_eq!(process(input).unwrap(), expected);
    }

    #[test]
    fn test_special_cases() {
        // Test incomplete tags - should result in parsing errors
        let input1 = "<div";
        assert!(
            process(input1).is_err(),
            "Incomplete opening tag should fail"
        );

        let input2 = "</div";
        assert!(
            process(input2).is_err(),
            "Incomplete closing tag should fail"
        );

        // Test unclosed tag
        let input3 = "<div>test";
        assert!(process(input3).is_err(), "Unclosed tag should fail");

        // Test self-closing void element
        let input4 = "<br />";
        let expected4 = "<br />"; // Self-closing tags are preserved
        assert_eq!(process(input4).unwrap(), expected4);

        // Test void element without self-closing slash (HTML5 style)
        let input5 = "<br>";
        let expected5 = "<br>"; // Should be handled correctly
        assert_eq!(process(input5).unwrap(), expected5);

        // Test attribute without value
        let input6 = "<input disabled>";
        let expected6 = "<input disabled>"; // Boolean attributes
        assert_eq!(process(input6).unwrap(), expected6);

        // Test attribute with empty value
        let input7 = "<input value=\"\">";
        let expected7 = "<input value=\"\">"; // Empty attribute value
        assert_eq!(process(input7).unwrap(), expected7);
    }

    #[test]
    fn test_simple() {
        let input = "<!DOCTYPE html>\n<html>\n<head>\n<title>PHP语法综合示例</title>\n</head>\n<body></body></html>";
        let expected = "<!DOCTYPE html>\n<html>\n<head>\n<title>PHP 语法综合示例</title>\n</head>\n<body></body></html>";
        assert_eq!(process(input).unwrap(), expected);

        let input = "<!DOCTYPE html>\n<html>\n<head>\n<title>PHP语法综合示例</title>\n</head>\n<body>";
        let expected = "<!DOCTYPE html>\n<html>\n<head>\n<title>PHP 语法综合示例</title>\n</head>\n<body>";
        assert_eq!(process(input).unwrap(), expected);
    }
}
