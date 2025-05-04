// src/js.rs
use anyhow::Result;
use pest::Parser;
use pest::iterators::Pair;
// Import the spacing function from the crate root
use crate::spacing;

#[derive(pest_derive::Parser)]
#[grammar = "grammar/js.pest"] // Path relative to src
pub struct JsParser;

pub fn process(input: &str) -> Result<String> {
    let pairs = match JsParser::parse(Rule::program, input) {
        Ok(p) => p,
        Err(e) => {
            println!("Pest parsing error:\n{}", e);
            return Err(e.into());
        }
    };
    let mut result = String::with_capacity(input.len());

    fn parse_pair(result: &mut String, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::comment | Rule::string => {
                for inner_pair in pair.into_inner() {
                    parse_pair(result, inner_pair);
                }
            }
            Rule::line_comment => {
                let content_pairs: Vec<_> = pair.into_inner().collect();
                let content = content_pairs.iter().map(|p| p.as_str()).collect::<String>();
                let spaced_content = spacing(&content);
                result.push_str(&format!("//{}", spaced_content));
            }
            Rule::block_comment => {
                let content_pairs: Vec<_> = pair.into_inner().collect();
                let content = content_pairs.iter().map(|p| p.as_str()).collect::<String>();
                let spaced_content = spacing(&content);
                result.push_str(&format!("/*{}*/", spaced_content));
            }
            Rule::double_quoted_string | Rule::single_quoted_string | Rule::template_literal => {
                let quote_char = match pair.as_rule() {
                    Rule::double_quoted_string => '"',
                    Rule::single_quoted_string => '\'',
                    Rule::template_literal => '`',
                    _ => unreachable!(),
                };

                if pair.as_rule() == Rule::template_literal {
                    result.push('`'); // Start template literal

                    let mut current_literal_chunk = String::new();
                    // Find the content pair first. Clone pair as into_inner consumes it.
                    let content_pair_opt = pair.clone().into_inner().find(|p| p.as_rule() == Rule::template_literal_content);
                    
                    if let Some(content_pair) = content_pair_opt {
                         for inner_pair in content_pair.into_inner() {
                            match inner_pair.as_rule() {
                                Rule::template_expression => {
                                    // Process the preceding literal chunk
                                    if !current_literal_chunk.is_empty() {
                                        result.push_str(&spacing(&current_literal_chunk));
                                        current_literal_chunk.clear();
                                    }
                                    // Append the expression directly (no spacing)
                                    result.push_str(inner_pair.as_str());
                                }
                                _ => {
                                    // Append other chars (literal text, escapes) to the current chunk
                                    current_literal_chunk.push_str(inner_pair.as_str());
                                }
                            }
                        }
                    } // else: Handle case with no template_literal_content?
                   

                    // Process any remaining literal chunk
                    if !current_literal_chunk.is_empty() {
                        result.push_str(&spacing(&current_literal_chunk));
                    }

                    result.push('`'); // End template literal
                } else {
                    // Original handling for double/single quotes
                    // Assume content is the first (and only) inner pair based on grammar
                    let content = pair.into_inner().next().map(|p| p.as_str()).unwrap_or(""); // Get content str
                    let spaced_content = spacing(content);
                    result.push(quote_char);
                    result.push_str(&spaced_content);
                    result.push(quote_char);
                }
            }
            _ => {
                result.push_str(pair.as_str());
            }
        }
    }

    for pair in pairs {
        parse_pair(&mut result, pair);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    // Helper function to check if parsing is successful for a rule
    fn assert_parses(rule: Rule, input: &str) {
        match JsParser::parse(rule, input) {
            Ok(_) => (), // Good, it parsed
            Err(e) => panic!("Failed to parse \"{}\" as {:?}:\n{}", input, rule, e),
        }
    }

    // Helper function to check if parsing fails for a rule
    fn assert_does_not_parse(rule: Rule, input: &str) {
        match JsParser::parse(rule, input) {
            Ok(pairs) => panic!(
                "Expected parsing to fail for \"{}\" as {:?}, but it succeeded with: {:?}",
                input, rule, pairs
            ),
            Err(_) => (), // Good, it failed
        }
    }

    // Test cases from before...
    #[test]
    fn test_parses_line_comment() {
        assert_parses(Rule::line_comment, "// This is a line comment\n");
        assert_parses(Rule::program, "const x = 1; // Assign value\n"); // Embedded in code
        assert_parses(Rule::program, "// Just a comment"); // EOF after comment
    }

    #[test]
    fn test_parses_block_comment() {
        assert_parses(Rule::block_comment, "/* This is a block comment */");
        assert_parses(Rule::block_comment, "/* Multi\nline\ncomment */");
        assert_parses(Rule::program, "let y = /* value */ 2;"); // Embedded
        assert_parses(Rule::program, "/* Start */ const z = 3; /* End */");
        assert_parses(Rule::program, "/*** Fancy comment ***/");
    }

    #[test]
    fn test_parses_strings() {
        // Double quotes
        assert_parses(Rule::double_quoted_string, r#""Hello""#);
        assert_parses(Rule::double_quoted_string, r#""With \"escaped\" quotes""#);
        assert_parses(Rule::double_quoted_string, r#""New\nline""#);
        assert_parses(Rule::program, r#"let greeting = "world";"#);

        // Single quotes
        assert_parses(Rule::single_quoted_string, r#"'Hello'"#);
        assert_parses(Rule::single_quoted_string, r#"'With \'escaped\' quotes'"#);
        assert_parses(Rule::single_quoted_string, r#"'New\nline'"#);
        assert_parses(Rule::program, r#"const name = 'Alice';"#);

        // Template literals
        assert_parses(Rule::template_literal, r#"`Simple template`"#);
        assert_parses(Rule::template_literal, r#"`Template with ${expression}`"#);
        assert_parses(Rule::template_literal, r#"`Escaped \` backtick`"#);
        assert_parses(Rule::template_literal, r#"`Multi\nline\ntemplate`"#);
        assert_parses(Rule::program, "const message = `Count: ${count * 2}`;");
        assert_parses(Rule::template_literal, r"`$`"); // Standalone $
        assert_parses(Rule::template_literal, r"`No expression ${}`"); // Empty expression
    }

    #[test]
    fn test_parses_double_quoted_string_directly() {
        assert_parses(Rule::double_quoted_string, r#""Hello""#);
        assert_parses(Rule::double_quoted_string, r#""""#); // Empty string
        assert_parses(Rule::double_quoted_string, r#""\\""#); // Escaped backslash (corrected)
    }

    #[test]
    fn test_parses_mixed_content() {
        let code = r#"
            // This is a setup function
            function setup() {
                const message = "Hello, World!"; // Initialize message
                let count = 0; /* Starting count */
                console.log(message);
                return `Setup complete. Count: ${count}`;// Return status
            }

            /*
             * Another block comment
             * with multiple lines.
             */
            const result = setup(); // Call the function
            console.log('Final result:', result);
        "#;
        // We only care that the whole program parses, specific captured pairs can be tested separately if needed
        // This test doesn't check the *output* of process, just that the grammar works.
        assert_parses(Rule::program, code);
    }

    // --- Add tests for the process function itself ---
    #[test]
    fn test_process_function_spacing() {
        // Test line comment spacing
        assert_eq!(process("//你好world").unwrap(), "//你好 world");
        // Test block comment spacing
        assert_eq!(process("/*你好world*/").unwrap(), "/*你好 world*/");
        // Test string spacing
        assert_eq!(process(r#""你好world""#).unwrap(), r#""你好 world""#);
        assert_eq!(process("'你好world'").unwrap(), "'你好 world'");
        assert_eq!(process("`你好world`").unwrap(), "`你好 world`");
        // Test mixed content
        let input = r#"
            const msg = "你好world"; //这是comment
            let val = /*测试一下*/ 123;
            console.log(`结果是 ${val}`); // Template literal with expression - spacing might break this currently
        "#;
        let expected = r#"
            const msg = "你好 world"; //这是 comment
            let val = /*测试一下*/ 123;
            console.log(`结果是 ${val}`); // Template literal with expression - spacing might break this currently
        "#;
        assert_eq!(process(input).unwrap(), expected);

        // Test content without needing spacing
        assert_eq!(process("hello // world").unwrap(), "hello // world");
        assert_eq!(process("let x = 'abc';").unwrap(), "let x = 'abc';");

        // Test empty comments/strings
        assert_eq!(process("//").unwrap(), "//");
        assert_eq!(process("/**/").unwrap(), "/**/");
        assert_eq!(process(r#""""#).unwrap(), r#""""#);
        assert_eq!(process("''").unwrap(), "''");
        assert_eq!(process("``").unwrap(), "``");

        // Test code before/after/between items
        assert_eq!(
            process("let before = 1; //你好world\nlet after = 2;").unwrap(),
            "let before = 1; //你好 world\nlet after = 2;"
        );
        assert_eq!(process("(\"你好world\")").unwrap(), "(\"你好 world\")");
    }

    #[test]
    fn test_snip() {
        let input = r#"let multiLineStr = "这是一个长字符串，\
可以用反斜杠换行"; // 字符串换行"#;
        let expect = r#"let multiLineStr = "这是一个长字符串，\
可以用反斜杠换行"; // 字符串换行"#;
        assert_eq!(process(input).unwrap(), expect);
    }

    #[test]
    fn test_snip2() {
        let input = r#"// 条件语句
if (num > 40) {
  console.log("num大于40");
} else if (num === 40) {
  console.log("num等于40");
} else {
  console.log("num小于40");
}"#;
        let expect = r#"// 条件语句
if (num > 40) {
  console.log("num 大于 40");
} else if (num === 40) {
  console.log("num 等于 40");
} else {
  console.log("num 小于 40");
}"#;
        assert_eq!(process(input).unwrap(), expect);
    }

    #[test]
    fn test_snip3() {
        let input = r#"// 数组和对象字面量
let arr = [1, 'two', true, null];
let obj = {
  name: "张三",
  age: 28,
  "favorite color": "blue", // 属性名含空格，必须用引号
  greet() { // 对象方法简写
    return `Hello, my name is ${this.name}`;
  }
};"#;
        let expect = r#"// 数组和对象字面量
let arr = [1, 'two', true, null];
let obj = {
  name: "张三",
  age: 28,
  "favorite color": "blue", // 属性名含空格，必须用引号
  greet() { // 对象方法简写
    return `Hello, my name is ${this.name}`;
  }
};"#;
        assert_eq!(process(input).unwrap(), expect);
    }

    #[test]
    fn test_snip4() {
        let input = r#"let s = "这是一个Long String，\
可以用反斜杠换行"; // 字符串换行"#;
        let expect = r#"let s = "这是一个 Long String，\
可以用反斜杠换行"; // 字符串换行"#;
        assert_eq!(process(input).unwrap(), expect);
    }

    #[test]
    fn test_seg1() {
        let source = r#"// Single Line声明变量并赋值
var num = 42; // 整数
let pi = 3.14159; // 浮点数
const greeting = "Hello世界!"; // 常量字符串
"#;
        let expected = r#"// Single Line 声明变量并赋值
var num = 42; // 整数
let pi = 3.14159; // 浮点数
const greeting = "Hello 世界!"; // 常量字符串
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg2() {
        let source = r#"/*
多行注释：
下面定义一个Function，演示函数声明和模板字符串的用法
*/
function sayHello(name) {
  // 模板String，支持变量插入和多行
  return `Hi, ${name}!
Welcome to JavaScript syntax示例.`;
}

// 使用函数
console.log(sayHello('小明'));
"#;
        let expected = r#"/*
多行注释：
下面定义一个 Function，演示函数声明和模板字符串的用法
*/
function sayHello(name) {
  // 模板 String，支持变量插入和多行
  return `Hi, ${name}!
Welcome to JavaScript syntax 示例.`;
}

// 使用函数
console.log(sayHello('小明'));
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg3() {
        let source = r#"// 字符串示例：单引号、双引号、转义字符
let singleQuoteStr = '这是单引号String';
let doubleQuoteStr = "这是双引号字符串";
let escapedStr = "He said, \"JavaScript好有趣!\""; // 转义双引号
let multiLineStr = "这是一个长字符串，\
可以用反斜杠换行"; // 字符串换行
"#;
        let expected = r#"// 字符串示例：单引号、双引号、转义字符
let singleQuoteStr = '这是单引号 String';
let doubleQuoteStr = "这是双引号字符串";
let escapedStr = "He said, \"JavaScript 好有趣!\""; // 转义双引号
let multiLineStr = "这是一个长字符串，\
可以用反斜杠换行"; // 字符串换行
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg4() {
        let source = r#"// 布尔值和空值
let isActive = true;
let isComplete = false;
let nothing = null;
let notDefined; // undefined
"#;
        let expected = r#"// 布尔值和空值
let isActive = true;
let isComplete = false;
let nothing = null;
let notDefined; // undefined
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg5() {
        let source = r#"// 数组和Object字面量
let arr = [1, 'two', true, null];
let obj = {
  name: "张三",
  age: 28,
  "favorite color": "blue", // 属性名含空格，必须用引号
  greet() { // 对象方法简写
    return `Hello, my name is ${this.name}`;
  }
};

console.log(obj.greet());
"#;
        let expected = r#"// 数组和 Object 字面量
let arr = [1, 'two', true, null];
let obj = {
  name: "张三",
  age: 28,
  "favorite color": "blue", // 属性名含空格，必须用引号
  greet() { // 对象方法简写
    return `Hello, my name is ${this.name}`;
  }
};

console.log(obj.greet());
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg6() {
        let source = r#"// 条件语句
if (num > 40) {
  console.log("num大于40");
} else if (num === 40) {
  console.log("num等于40");
} else {
  console.log("num小于40");
}
"#;
        let expected = r#"// 条件语句
if (num > 40) {
  console.log("num 大于 40");
} else if (num === 40) {
  console.log("num 等于 40");
} else {
  console.log("num 小于 40");
}
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg7() {
        let source = r#"// 循环示例
for (let i = 0; i < arr.length; i++) {
  console.log(`arr[${i}] = ${arr[i]}`);
}

// while循环
let count = 3;
while (count > 0) {
  console.log(`倒计时：${count}`);
  count--;
}
"#;
        let expected = r#"// 循环示例
for (let i = 0; i < arr.length; i++) {
  console.log(`arr[${i}] = ${arr[i]}`);
}

// while 循环
let count = 3;
while (count > 0) {
  console.log(`倒计时：${count}`);
  count--;
}
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg8() {
        let source = r#"// try...catch异常处理
try {
  throw new Error("这是一个错误");
} catch (e) {
  console.error("捕获到错误:", e.message);
} finally {
  console.log("无论是否出错，都会执行");
}
"#;
        let expected = r#"// try...catch 异常处理
try {
  throw new Error("这是一个错误");
} catch (e) {
  console.error("捕获到错误:", e.message);
} finally {
  console.log("无论是否出错，都会执行");
}
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg9() {
        let source = r#"// 箭头函数和默认参数
const add = (a = 0, b = 0) => a + b;
console.log(`1 + 2 = ${add(1, 2)}`);

// 解构赋值
const [first, second] = arr;
const { name, age } = obj;
console.log(`名字：${name}, 年龄：${age}`);
"#;
        let expected = r#"// 箭头函数和默认参数
const add = (a = 0, b = 0) => a + b;
console.log(`1 + 2 = ${add(1, 2)}`);

// 解构赋值
const [first, second] = arr;
const { name, age } = obj;
console.log(`名字：${name}, 年龄：${age}`);
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_seg10() {
        let source = r#"// 使用模板字符串多行和表达式
const multiline = `
这是一个多行字符串示例
2 + 3 = ${2 + 3}
`;

console.log(multiline);

// 使用注释阻止代码执行
// console.log("这行代码被注释，不会执行");

/*
多行注释也可以阻止多行代码执行
console.log("这行Code不会执行");
console.log("这行代码也不会执行");
*/
"#;
        let expected = r#"// 使用模板字符串多行和表达式
const multiline = `
这是一个多行字符串示例
2 + 3 = ${2 + 3}
`;

console.log(multiline);

// 使用注释阻止代码执行
// console.log("这行代码被注释，不会执行");

/*
多行注释也可以阻止多行代码执行
console.log("这行 Code 不会执行");
console.log("这行代码也不会执行");
*/
"#;
        assert_eq!(process(source).unwrap(), expected);
    }

    #[test]
    fn test_integrative() {
        let source = std::fs::read_to_string("test-data/source.js").unwrap();
        let expect = std::fs::read_to_string("test-data/expect.js").unwrap();

        let result = process(&source).unwrap();
        std::fs::write("test-data/.result.js", &result).unwrap();

        assert_eq!(result, expect);
    }
}
