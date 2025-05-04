#[macro_use]
extern crate pest_derive;

use std::collections::VecDeque;
use unicode_width::UnicodeWidthChar;

// 声明模块
pub mod html;
mod json;
mod json5;
mod js; // Add js module declaration
pub mod markdown;
mod rust;
pub mod css;
pub mod php; // Add php module declaration

// Re-export 主要函数
pub use html::process as process_html;
pub use markdown::process as process_markdown;
pub use css::process as process_css;
pub use rust::process as process_rust;
pub use json::process as process_json;
pub use json5::process as process_json5;
pub use php::process as process_php;

/// （在一定条件下）在全角和半角字符之间添加空格
///
/// # Examples
///
/// ```
/// use paranoid_space::spacing;
///
/// let text = "当你凝视着bug，bug也凝视着你";
/// assert_eq!(spacing(text), "当你凝视着 bug，bug 也凝视着你");
/// ```
pub fn spacing(text: &str) -> String {
    // 如果文本为空，直接返回空字符串
    if text.is_empty() {
        return String::new();
    }
    // println!("--- spacing called with: \"{}\" ---", text); // Start marker

    // 处理转义字符
    // let text = process_escape_sequences(text);

    let mut origin: VecDeque<char> = text.chars().collect();
    let mut result: Vec<char> = Vec::new();

    let mut prev: Option<char> = None;
    // let mut cur: Option<char> = None;
    while let Some(cur_ch) = origin.pop_front() {
        // println!("Loop start: prev={:?}, cur='{}'", prev, cur_ch.escape_debug()); // Debug print

        match (prev, cur_ch) {
            (Some(prev_ch), cur_ch) => {
                let prev_ch_width = CharWidth::from_char(prev_ch);
                let cur_ch_width = CharWidth::from_char(cur_ch);
                // println!("  Prev width: {:?}, Cur width: {:?}", prev_ch_width, cur_ch_width); // Debug print

                // case 0: prev is space
                if prev_ch == ' ' {
                    // println!("  Case 0: prev is space"); // Debug print
                    result.push(cur_ch);
                    prev = Some(cur_ch);
                    continue;
                }

                // case 1: prev is full, cur is half
                if prev_ch_width.is_full() && cur_ch_width.is_half() {
                    // println!("  Case 1: prev=Full, cur=Half"); // Debug print
                    // special case: 全角字符与半角标点之间不加空格, 全角标点与半角字符之间不加空格
                    let special_pre_full = vec![
                        '，', '。', '！', '？', '：', '；', '“', '”', '‘', '’', '《', '》', '【',
                        '】', '（', '）', '—', '…', '～', '·', '、',
                    ];
                    let special_cur_half =
                        vec![',', '.', '!', '?', ':', ';', '"', '\'', '\n', '\r', '\t', '\\']; // Added missing quote for vec definition

                    // special case：货币符号后跟数字不加空格
                    let is_currency_before_number =
                        (prev_ch == '¥' || prev_ch == '€') && cur_ch.is_numeric();

                    if cur_ch != ' '
                        && !special_pre_full.contains(&prev_ch)
                        && !special_cur_half.contains(&cur_ch)
                        && !is_currency_before_number
                    {
                        // println!("    Adding space between Full and Half"); // Debug print
                        result.push(' ');
                    } else {
                        // println!("    Skipping space between Full and Half (special case)"); // Debug print
                    }

                    result.push(cur_ch);
                    prev = Some(cur_ch);
                    continue;
                }

                // case 2: prev is half, cur is full
                if prev_ch_width.is_half() && cur_ch_width.is_full() {
                    // println!("  Case 2: prev=Half, cur=Full"); // Debug print
                    // special case: 半角符号与全角字符不加空格，半角字符与全角标点间不加空格
                    let special_pre_half = vec![
                        '"', '\'', '[', '{', '<', '@', '#', '%', '^', '&',
                        '_', '|', '\\',
                    ];
                    // 全角标点等特殊字符
                    let special_cur_full = vec![
                        '，', '。', '！', '？', '：', '；', '“', '”', '‘', '’', '《', '》', '【',
                        '】', '（', '）', '—', '…',
                    ];

                    // special case: 货币符号后跟数字不加空格
                    let is_currency_before_number =
                        (prev_ch == '$' || prev_ch == '¥' || prev_ch == '€')
                            && cur_ch_width.is_full()
                            && !special_cur_full.contains(&cur_ch);

                    // special case: 换行符不加空格
                    let is_line_break = prev_ch == '\n';

                    if !special_pre_half.contains(&prev_ch)
                        && !special_cur_full.contains(&cur_ch)
                        && !is_currency_before_number
                        && !is_line_break
                    {
                        // println!("    Adding space between Half and Full"); // Debug print
                        result.push(' ');
                    } else {
                        // println!("    Skipping space between Half and Full (special case)"); // Debug print
                    }
                    result.push(cur_ch);
                    prev = Some(cur_ch);
                    continue;
                }

                // other cases:
                // prev is full, cur is full,
                // prev is half, cur is half
                // println!("  Other case: pushing cur"); // Debug print
                result.push(cur_ch);
                prev = Some(cur_ch);
                continue;
            }
            // the first char
            (None, cur) => {
                // println!("  First char case"); // Debug print
                result.push(cur);
                prev = Some(cur);
                continue;
            }
        }
    }

    // 将结果 Vec 转换为字符串
    let final_result: String = result.into_iter().collect();
    // println!("--- spacing returning: \"{}\" ---", final_result); // End marker
    final_result
}

#[derive(Debug, PartialEq)]
pub enum CharWidth {
    Half, // 半角字符
    Full, // 全角字符
}

impl CharWidth {
    pub fn from_char(c: char) -> Self {
        match c.width() {
            Some(1) | None => CharWidth::Half,
            Some(2) => CharWidth::Full,
            Some(_) => CharWidth::Full, // 其他宽度视为全角
        }
    }

    pub fn is_half(&self) -> bool {
        matches!(self, CharWidth::Half)
    }

    pub fn is_full(&self) -> bool {
        matches!(self, CharWidth::Full)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_comments() {
        assert_eq!(spacing("/* block comment */"), "/* block comment */");
        assert_eq!(spacing("/* block 注释 */"), "/* block 注释 */");
        assert_eq!(spacing("// line comment"), "// line comment");
        assert_eq!(spacing("// line 注释"), "// line 注释");
    }

    #[test]
    fn test_end_space() {
        assert_eq!(spacing("a "), "a ");
        assert_eq!(spacing("a  "), "a  ");
        assert_eq!(spacing("a啊 "), "a 啊 ");
    }

    #[test]
    fn test_newline_or_tab() {
        assert_eq!(CharWidth::from_char('\n'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('\r'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('\t'), CharWidth::Half);

        assert_eq!(spacing("a\nb"), "a\nb");
        assert_eq!(spacing("a\rb"), "a\rb");
        assert_eq!(spacing("a\tb"), "a\tb");

        assert_eq!(spacing("中\nb"), "中\nb");
        assert_eq!(spacing("中\rb"), "中\rb");
        assert_eq!(spacing("中\tb"), "中\tb");
    }

    #[test]
    fn test_char_width() {
        assert_eq!(CharWidth::from_char(' '), CharWidth::Half);
        assert_eq!(CharWidth::from_char('a'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('A'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('1'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('，'), CharWidth::Full);
        assert_eq!(CharWidth::from_char('。'), CharWidth::Full);
        assert_eq!(CharWidth::from_char('！'), CharWidth::Full);
        assert_eq!(CharWidth::from_char('？'), CharWidth::Full);
        assert_eq!(CharWidth::from_char('￥'), CharWidth::Full);
        assert_eq!(CharWidth::from_char('$'), CharWidth::Half);
        assert_eq!(CharWidth::from_char('中'), CharWidth::Full);
    }

    #[test]
    fn test_spacing() {
        // 中文和英文之间
        assert_eq!(spacing("中文English"), "中文 English");
        assert_eq!(spacing("中文English中文"), "中文 English 中文");

        // 中文和数字之间
        assert_eq!(spacing("中文123"), "中文 123");
        assert_eq!(spacing("123中文"), "123 中文");

        // 中文和符号之间
        assert_eq!(spacing("中文!"), "中文!");
        assert_eq!(spacing("中文?"), "中文?");

        // 货币符号测试
        assert_eq!(spacing("价格是$50和¥300"), "价格是 $50 和 ¥300");
        assert_eq!(spacing("价格是¥300"), "价格是 ¥300");

        // 复杂情况
        assert_eq!(
            spacing("当你凝视着bug，bug也凝视着你"),
            "当你凝视着 bug，bug 也凝视着你"
        );
        assert_eq!(
            spacing("与PM战斗的人，应当小心自己不要成为PM"),
            "与 PM 战斗的人，应当小心自己不要成为 PM"
        );
        assert_eq!(
            spacing("与PM战斗的人，应当小心自己不要成为 PM"),
            "与 PM 战斗的人，应当小心自己不要成为 PM"
        );
    }

    #[test]
    fn test_all_width() {
        // 测试中英文混排
        assert_eq!(
            spacing("中文和拉丁字母English混排"),
            "中文和拉丁字母 English 混排"
        );

        // 测试全角和半角数字
        assert_eq!(
            spacing("中文数字１２３４５６７８９０和半角数字1234567890混排"),
            "中文数字１２３４５６７８９０和半角数字 1234567890 混排"
        );

        // 测试混合案例，包含函数名和引号
        assert_eq!(
            spacing("使用了Python的print()函数打印\"你好,世界\""),
            "使用了 Python 的 print() 函数打印\"你好, 世界\""
        );

        // 测试货币符号
        assert_eq!(
            spacing("价格人民币¥100美元$100欧元€100英镑£100"),
            "价格人民币 ¥100 美元 $100 欧元 €100 英镑 £100"
        );

        // 测试全角空格和半角空格
        assert_eq!(
            spacing("全角空格　和半角空格 混用"),
            "全角空格　和半角空格 混用"
        );

        // 测试全角和半角字母数字混排
        assert_eq!(
            spacing("AＡBＢCＣ和abc以及1１２３和123混排"),
            "A Ａ B Ｂ C Ｃ和 abc 以及 1 １２３和 123 混排"
        );

        // 测试路径表示
        assert_eq!(
            spacing("文件保存在~/Documents目录"),
            "文件保存在 ~/Documents 目录"
        );
    }

    #[test]
    fn test_symbols() {
        // 波浪号测试
        assert_eq!(
            spacing("用户目录是~，完整路径是~/Documents"),
            "用户目录是 ~，完整路径是 ~/Documents"
        );

        // 括号测试
        assert_eq!(spacing("函数add(a,b)返回a+b"), "函数 add(a,b) 返回 a+b");

        // 路径测试
        assert_eq!(
            spacing("文件保存在/usr/local/bin/目录"),
            "文件保存在 /usr/local/bin/ 目录"
        );

        // 点号测试
        assert_eq!(
            spacing("网址是example.com而不是example。com"),
            "网址是 example.com 而不是 example。com"
        );

        // 引号测试
        assert_eq!(
            spacing(r#"他说"这很好"然后离开了"#),
            r#"他说"这很好"然后离开了"#
        );

        // 复杂组合
        assert_eq!(
            spacing("安装命令是npm install --save-dev @types/react使用v16.8版本"),
            "安装命令是 npm install --save-dev @types/react 使用 v16.8 版本"
        );

        // 货币符号
        assert_eq!(spacing("价格是$50和¥300"), "价格是 $50 和 ¥300");

        // 分隔符测试
        assert_eq!(
            spacing("name|age|gender表示不同字段"),
            "name|age|gender 表示不同字段"
        );

        // 数学符号
        assert_eq!(
            spacing("5+3*2=11，需要满足x>0且y<100"),
            "5+3*2=11，需要满足 x>0 且 y<100"
        );

        // 反引号
        assert_eq!(
            spacing("命令是`ls -la`，注意不要用''"),
            "命令是 `ls -la`，注意不要用''"
        );
    }

    #[test]
    fn test_process_escape_sequences() {
        let input = r#"\t"#;
        let expected = r#"\t"#;
        assert_eq!(spacing(input), expected);
    }

    #[test]
    fn test_string_with_escapes() {
        let input: &str = r#"你好\n world\t!"#;
        let expected: &str = r#"你好\n world\t!"#;
        assert_eq!(spacing(input), expected);
    }

    #[test]
    fn test_specific_char_widths() {
        use unicode_width::UnicodeWidthChar;
        // Test specific characters involved in the failing case
        assert_eq!(Some(2), '好'.width(), "Width of '好'");
        assert_eq!(Some(1), 'w'.width(), "Width of 'w'");

        // Test some other common cases
        assert_eq!(Some(2), '，'.width(), "Width of full-width comma");
        assert_eq!(Some(1), ','.width(), "Width of half-width comma");
        assert_eq!(Some(1), ' '.width(), "Width of space");
        assert_eq!(None, '\n'.width(), "Width of newline"); // Control char
    }

    #[test]
    fn test_charwidth_enum_mapping() {
        // Test specific characters involved in the failing case
        assert_eq!(CharWidth::Full, CharWidth::from_char('好'), "Mapping for '好'");
        assert_eq!(CharWidth::Half, CharWidth::from_char('w'), "Mapping for 'w'");

        // Test some other common cases
        assert_eq!(CharWidth::Full, CharWidth::from_char('，'), "Mapping for full-width comma");
        assert_eq!(CharWidth::Half, CharWidth::from_char(','), "Mapping for half-width comma");
        assert_eq!(CharWidth::Half, CharWidth::from_char(' '), "Mapping for space");
        assert_eq!(CharWidth::Half, CharWidth::from_char('\n'), "Mapping for newline"); // Maps None to Half
    }

    #[test]
    fn test_spacing_nihaoworld() {
        let input = "你好world";
        let expected = "你好 world";
        // 调用 spacing，此时应该触发内部的 println!
        let actual = spacing(input);
        // 在断言前打印结果，增加看到输出的机会
        println!("[test_spacing_nihaoworld] Input: \"{}\", Expected: \"{}\", Actual: \"{}\"", input, expected, actual);
        assert_eq!(actual, expected);
    }
}
