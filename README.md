# 中文空格处理工具 (Paranoid Space)

这个项目提供了一个自动在全角（如中文、日文、韩文）和半角（如英文字母、数字、符号）字符之间添加空格的工具，以提高文本的可读性。

## 为什么需要这个工具？

当全角和半角文字混合排列时，如果没有空格分隔，会影响阅读体验。例如：

- 未处理：`数字123与中文之间需要空格`
- 处理后：`数字 123 与中文之间需要空格`

## Warning && Notice

这是一个基于规则的项目，只是针对若干常见情况做了添加，或不添加空格的处理，不可能适用于所有情况。
另外，它也还在开发中，受限于本人的精力，目前只实现了部分语言的部分情况。
后续会随着工作需要，不断完善。
欢迎提议、提交 PR，提交 Issue，一起完善它。

## 安装

### Cargo install

```
cargo install paranoid-space
```

### 从源码编译

```bash
git clone https://github.com/kookyleo/paranoid-space
cd paranoid-space
cargo build --release
```

编译后的可执行文件将在 `target/release` 目录中。

## 使用方法

这个工具提供了几种使用方式：

### 1. 处理文件并输出到标准输出

```bash
paranoid-space your_file.txt
```

输出示例:
```
中文 English 混合文本处理示例：

以下是一些需要处理的例子：
1. 中文与英文 Rust 之间需要有空格
...
```

### 2. 直接修改源文件（使用 -i 参数）

```bash
paranoid-space -i your_file.txt
```

这个命令会直接将修改写回到原文件中，不会有任何输出。

### 3. 显示差异对比（使用 -d 或 --diff 参数）

```bash
paranoid-space -d your_file.txt
```

这个命令会以行对比方式显示原始内容和处理后内容的彩色差异:
- 变化的行显示行号、原始内容（红色 `-`）和处理后内容（绿色 `+`）
- 连续无变化的行仅用灰色 `...` 表示，不显示行号

```diff
01 - 这是第1行，包含中文English混合内容。
01 + 这是第 1 行，包含中文 English 混合内容。
02 - 这是第2行，数字123和汉字之间需要空格。
02 + 这是第 2 行，数字 123 和汉字之间需要空格。
...
06 - 这是第6行，中英混排English需要处理。
06 + 这是第 6 行，中英混排 English 需要处理。
```

这种显示方式让你可以清晰地看到哪些行发生了变化，同时忽略没有变化的部分以减少输出量。

### 4. 从标准输入读取

如果不指定文件参数，程序会从标准输入读取内容：

```bash
echo "这是一个Example" | paranoid-space
```

输出:
```
这是一个 Example
```

也可以结合 `-d/--diff` 参数显示差异：

```bash
echo "这是一个Example" | paranoid-space -d
```

## 特殊文件格式支持

命令行工具会根据文件扩展名自动选择合适的处理方式：

- **HTML 文件** (`.html`, `.htm`) - 调用 `process_html`，保留 HTML 标签，只对标签内的文本内容添加空格。
- **Markdown 文件** (`.md`, `.markdown`) - 调用 `process_markdown`，保留 Markdown 语法（如代码块、行内代码、链接等），只对普通文本内容添加空格。
- **其他文件** - 对于所有其他文件扩展名或没有扩展名的文件，会调用通用的 `spacing` 函数，对整个内容进行处理。这意味着对于 CSS, JS, PHP, Rust, JSON 等格式，其代码结构可能不会被正确保留，建议在代码中使用对应的特定处理函数。

## 在代码中使用

可以在 Rust 项目中将本库作为依赖引入，并使用针对特定格式的函数：

```rust
use paranoid_space::{
    spacing,        // 通用处理函数
    process_html,   // 处理 HTML
    process_markdown, // 处理 Markdown
    process_css,    // 处理 CSS
    process_rust,   // 处理 Rust
    process_json,   // 处理 JSON
    process_json5,  // 处理 JSON5
    process_php,    // 处理 PHP
    process_js,     // 处理 JS
};

fn main() {
    // 处理普通文本
    let text = "数字123与中文之间需要空格";
    let result = spacing(text);
    println!("{}", result);  // output: 数字 123 与中文之间需要空格

    // 处理 HTML 文本
    let html = "<div>这是中文English混合</div>";
    let html_result = process_html(html).unwrap_or_else(|e| {
        eprintln!("HTML processing error: {}", e);
        html.to_string() // 错误时返回原始内容
    });
    println!("{}", html_result); // output: <div>这是中文 English 混合</div>

    // 处理 Markdown 文本
    let md = "# 标题Title\n```code块不处理```";
    let md_result = process_markdown(md);
    println!("{}", md_result); // output: # 标题 Title\n```code块不处理```

    // 处理 CSS
    let css = "body { font-family: \"微软雅黑\", Arial; } /* 注释comment */";
    let css_result = process_css(css).unwrap_or_else(|e| {
        eprintln!("CSS processing error: {}", e);
        css.to_string()
    });
    println!("{}", css_result); // output: body { font-family: \"微软雅黑\", Arial; } /* 注释 comment */

    // 处理 Rust 代码 (示例，具体效果取决于实现)
    let rust_code = "fn main() { println!(\"你好world\"); }";
    let rust_result = process_rust(rust_code).unwrap_or_else(|e| {
        eprintln!("Rust processing error: {}", e);
        rust_code.to_string()
    });
    println!("{}", rust_result); // output: fn main() { println!(\"你好 world\"); }

    // 处理 PHP 代码 (示例)
    let php_code = "<?php echo '你好'.'world'; ?>";
    let php_result = process_php(php_code).unwrap_or_else(|e| {
        eprintln!("PHP processing error: {}", e);
        php_code.to_string()
    });
    println!("{}", php_result); // output: <?php echo '你好' . 'world'; ?> (假设它保留了语法结构)
}

## 许可证

MIT