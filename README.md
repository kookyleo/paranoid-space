# 中文空格处理工具 (Paranoid Space)

这个项目提供了一个自动在全角（如中文、日文、韩文）和半角（如英文字母、数字、符号）字符之间添加空格的工具，以提高文本的可读性。

## 为什么需要这个工具？

当全角和半角文字混合排列时，如果没有空格分隔，会影响阅读体验。例如：

- 未处理：`数字123与中文之间需要空格`
- 处理后：`数字 123 与中文之间需要空格`

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

这个工具提供了两种使用方式：

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

### 3. 从标准输入读取

如果不指定文件参数，程序会从标准输入读取内容：

```bash
echo "这是一个Example" | paranoid-space
```

输出:
```
这是一个 Example
```

## 特殊文件格式支持

工具会根据文件扩展名自动选择合适的处理方式：

- **Markdown文件** (`.md`, `.markdown`) - 保留Markdown语法，只对文本内容添加空格
- **HTML文件** (`.html`, `.htm`) - 保留HTML标签，只对文本内容添加空格
- **普通文本文件** - 对整个内容进行处理

例如，处理Markdown文件时，代码块和行内代码不会被修改，而只有普通文本段落会被添加空格。

## 在代码中使用

可以在 Rust 项目中将本库作为依赖引入：

```rust
// 处理普通文本
use paranoid_space::spacing;
// 处理HTML文本
use paranoid_space::spacing_html;
// 处理Markdown文本
use paranoid_space::spacing_markdown;

fn main() {
    // 处理普通文本
    let text = "数字123与中文之间需要空格";
    let result = spacing(text);
    println!("{}", result);  // output: 数字 123 与中文之间需要空格
    
    // 处理HTML文本
    let html = "<div>这是中文English混合</div>";
    let html_result = spacing_html(html);
    println!("{}", html_result);  // output: <div>这是中文 English 混合</div>
    
    // 处理Markdown文本
    let md = "# 标题Title\n```code块不处理```";
    let md_result = spacing_markdown(md);
    println!("{}", md_result);  // output: # 标题 Title\n```code块不处理```
}
```

## 许可证

MIT