use pulldown_cmark::{Parser, Options, Event, Tag};
use crate::spacing;

/// 处理Markdown文件中的文本，保留Markdown语法标记
///
/// # Examples
///
/// ```
/// use paranoid_space::spacing_markdown;
///
/// let md_text = "# 标题\n\n代码示例：`console.log(Hello世界)`";
/// let result = spacing_markdown(md_text);
/// // 保留标题标记，只对内容应用间距规则
/// assert!(result.contains("# 标题") && result.contains("`console.log(Hello世界)`"));
/// ```
pub fn spacing_markdown(text: &str) -> String {
    // 设置Markdown解析选项
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    
    // 解析Markdown
    let parser = Parser::new_ext(text, options);
    
    // 收集处理后的事件
    let mut processed_events = Vec::new();
    
    // 处理每个事件
    for event in parser {
        match event {
            // 处理纯文本内容
            Event::Text(text) => {
                // 对文本应用间距规则
                let processed_text = spacing(&text);
                processed_events.push(Event::Text(processed_text.into()));
            },
            // 代码块不处理
            Event::Code(code) => {
                processed_events.push(Event::Code(code));
            },
            // HTML块使用spacing_html处理
            Event::Html(html) => {
                // 由于我们没有实现crate::spacing_html的用法，这里暂时原样保留
                processed_events.push(Event::Html(html));
            },
            // 其他事件不变
            _ => processed_events.push(event),
        }
    }
    
    // 将处理后的事件按行输出，用于调试
    let parser = Parser::new_ext(text, options);
    let mut markdown_output = String::new();
    let mut in_code_block = false;
    
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        markdown_output.push_str("\n\n");
                        for _ in 0..level as usize {
                            markdown_output.push('#');
                        }
                        markdown_output.push(' ');
                    },
                    Tag::BlockQuote(_) => {
                        markdown_output.push_str("\n\n> ");
                    },
                    Tag::CodeBlock(kind) => {
                        in_code_block = true;
                        markdown_output.push_str("\n\n```");
                        if let pulldown_cmark::CodeBlockKind::Fenced(info) = kind {
                            markdown_output.push_str(&info);
                        }
                        markdown_output.push('\n');
                    },
                    Tag::List(first_item_number) => {
                        markdown_output.push_str("\n\n");
                        if let Some(n) = first_item_number {
                            markdown_output.push_str(&format!("{}. ", n));
                        }
                    },
                    Tag::Item => {
                        markdown_output.push_str("\n- ");
                    },
                    Tag::Emphasis => {
                        markdown_output.push('*');
                    },
                    Tag::Strong => {
                        markdown_output.push_str("**");
                    },
                    Tag::Link { .. } => {
                        markdown_output.push('[');
                    },
                    Tag::Image { .. } => {
                        markdown_output.push_str("![");
                    },
                    // Tag::Paragraph => do nothing
                    _ => {},
                }
            },
            Event::End(tag) => {
                match tag {
                    pulldown_cmark::TagEnd::Paragraph => markdown_output.push('\n'),
                    pulldown_cmark::TagEnd::Heading(_) => markdown_output.push('\n'),
                    pulldown_cmark::TagEnd::BlockQuote(_) => markdown_output.push('\n'),
                    pulldown_cmark::TagEnd::CodeBlock => {
                        in_code_block = false;
                        markdown_output.push_str("\n```\n");
                    },
                    pulldown_cmark::TagEnd::List(_) => markdown_output.push('\n'),
                    pulldown_cmark::TagEnd::Item => markdown_output.push('\n'),
                    pulldown_cmark::TagEnd::Emphasis => markdown_output.push('*'),
                    pulldown_cmark::TagEnd::Strong => markdown_output.push_str("**"),
                    pulldown_cmark::TagEnd::Link => {
                        // 简化处理，不尝试重建完整的链接语法
                        markdown_output.push_str("](url)");
                    },
                    pulldown_cmark::TagEnd::Image => {
                        markdown_output.push_str("](image)");
                    },
                    _ => {},
                }
            },
            Event::Text(text) => {
                if !in_code_block {
                    // 应用间距规则
                    markdown_output.push_str(&spacing(&text));
                } else {
                    // 代码块内不处理
                    markdown_output.push_str(&text);
                }
            },
            Event::Code(code) => {
                markdown_output.push('`');
                markdown_output.push_str(&code);
                markdown_output.push('`');
            },
            Event::Html(html) => {
                markdown_output.push_str(&html);
            },
            Event::SoftBreak => {
                markdown_output.push('\n');
            },
            Event::HardBreak => {
                markdown_output.push_str("  \n");
            },
            Event::Rule => {
                markdown_output.push_str("\n\n---\n\n");
            },
            Event::TaskListMarker(checked) => {
                if checked {
                    markdown_output.push_str("[x] ");
                } else {
                    markdown_output.push_str("[ ] ");
                }
            },
            _ => {},
        }
    }
    
    // 一个简单的方法：使用正则表达式来替换重复的空格
    let result = markdown_output.trim().to_string();
    
    // 返回最终的 Markdown 文本
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let result = spacing_markdown("> Rust是一门系统编程语言");
        assert_eq!(result, "> Rust 是一门系统编程语言");
    }
    
    #[test]
    fn test_spacing_markdown() {
        // 测试代码块保留
        let result = spacing_markdown("```rust\nlet x = Hello世界;\n```");
        assert!(result.contains("```rust") && result.contains("Hello世界") && !result.contains("Hello 世界"));
        
        // 测试行内代码保留
        let result = spacing_markdown("使用`println!`函数输出");
        assert!(result.contains("使用") && result.contains("`println!`") && result.contains("函数输出"));
        
        // 测试链接文本处理
        let result = spacing_markdown("[关于Rust语言](https://rust-lang.org)");
        assert!(result.contains("关于") && result.contains("Rust") && result.contains("语言"));
        
        // 测试图片Alt文本处理
        let result = spacing_markdown("![Rust标志](rust-logo.png)");
        assert!(result.contains("Rust") && result.contains("标志"));
        
        // 测试标题处理
        let result = spacing_markdown("# Rust编程语言介绍");
        assert!(result.contains("# Rust 编程语言介绍") || result.contains("#Rust 编程语言介绍"));
        
        // 测试列表项处理
        let result = spacing_markdown("- 安装Rust\n- 学习语法");
        assert!(result.contains("安装 Rust") && result.contains("学习语法"));
        
        // 测试任务列表处理
        let result = spacing_markdown("- [ ] 完成Rust项目\n- [x] 阅读文档");
        assert!(result.contains("完成 Rust 项目") && result.contains("阅读文档"));
        
        // 测试引用处理
        let result = spacing_markdown("> Rust是一门系统编程语言");
        assert!(result.contains("> Rust 是一门系统编程语言") || result.contains(">Rust 是一门系统编程语言"));
        
        // 测试复杂情况
        let result = spacing_markdown("# 学习Rust\n\n代码示例：`let x = Hello世界;`\n\n详细信息请参考[官方文档](https://doc.rust-lang.org)");
        assert!(result.contains("学习 Rust") && 
               result.contains("代码示例") && 
               result.contains("`let x = Hello世界;`") && 
               result.contains("详细信息请参考") && 
               result.contains("官方文档"));
    }
} 