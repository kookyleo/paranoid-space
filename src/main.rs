use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use paranoid_space::{
    process_css, process_html, process_js, process_json, process_json5, process_markdown,
    process_php, process_rust, spacing,
};

// 添加 diff 相关的依赖
use console::Style;

#[derive(Parser)]
#[command(name = "paranoid-space")]
#[command(about = "auto add space between full-width and half-width characters")]
#[command(version)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// 需要处理的文件路径，如不指定则从标准输入读取
    file: Option<PathBuf>,

    /// 是否直接修改源文件（如不指定则输出到标准输出）
    #[arg(short = 'i')]
    in_place: bool,

    /// 是否显示差异对比
    #[arg(short = 'd', long = "diff")]
    diff: bool,
}

/// 根据文件扩展名选择合适的处理函数
fn process_content(content: &str, file_path: Option<&PathBuf>) -> String {
    match file_path {
        Some(path) => {
            if let Some(extension) = path.extension() {
                match extension.to_str() {
                    Some("html") | Some("htm") => process_html(content).unwrap(),
                    Some("md") | Some("markdown") => process_markdown(content).unwrap(),
                    Some("js") => process_js(content).unwrap(),
                    Some("json") => process_json(content).unwrap(),
                    Some("json5") => process_json5(content).unwrap(),
                    Some("php") => process_php(content).unwrap(),
                    Some("rust") => process_rust(content).unwrap(),
                    Some("css") => process_css(content).unwrap(),
                    _ => spacing(content),
                }
            } else {
                // 没有扩展名的文件使用普通文本处理
                spacing(content)
            }
        }
        None => {
            // 从标准输入读取的内容使用普通文本处理
            spacing(content)
        }
    }
}

/// 显示原始内容和处理后内容的彩色差异，行对比方式
fn show_diff(original: &str, processed: &str) -> io::Result<()> {
    // 定义样式
    let added = Style::new().green();
    let removed = Style::new().red();
    let unchanged = Style::new().dim(); // 用于显示无变化的行
    let line_num_style = Style::new().cyan().dim(); // 行号样式

    // 存储原始行和处理后行，用于配对显示
    let original_lines: Vec<&str> = original.lines().collect();
    let processed_lines: Vec<&str> = processed.lines().collect();

    // 计算行号显示宽度（根据总行数确定）
    let total_lines = original_lines.len().max(processed_lines.len());
    let line_width = total_lines.to_string().len();

    let mut line_num = 0;
    let mut in_unchanged_block = false; // 跟踪是否在连续无变化的区块中

    // 打印空首行
    writeln!(io::stdout(), "")?;

    // 逐行配对显示差异
    while line_num < original_lines.len() || line_num < processed_lines.len() {
        // 只有在两者都存在并且内容相同时才显示为"无变化"
        if line_num < original_lines.len() && line_num < processed_lines.len() {
            if original_lines[line_num] == processed_lines[line_num] {
                // 记录无变化区块的开始
                if !in_unchanged_block {
                    in_unchanged_block = true;
                    writeln!(io::stdout(), "{}", unchanged.apply_to("..."))?;
                }

                line_num += 1;
                continue;
            }
        }

        // 当遇到有变化的行时，重置无变化区块标志
        in_unchanged_block = false;

        // 显示原始行（如果存在）
        if line_num < original_lines.len() {
            write!(
                io::stdout(),
                "{} ",
                line_num_style.apply_to(format!("{:0width$}", line_num + 1, width = line_width))
            )?;
            writeln!(
                io::stdout(),
                "{} {}",
                removed.apply_to("-"),
                removed.apply_to(original_lines[line_num])
            )?;
        }

        // 显示处理后行（如果存在）
        if line_num < processed_lines.len() {
            write!(
                io::stdout(),
                "{} ",
                line_num_style.apply_to(format!("{:0width$}", line_num + 1, width = line_width))
            )?;
            writeln!(
                io::stdout(),
                "{} {}",
                added.apply_to("+"),
                added.apply_to(processed_lines[line_num])
            )?;
        }

        line_num += 1;
    }

    // 打印空尾行
    writeln!(io::stdout(), "")?;

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.file {
        Some(ref file_path) => {
            // 处理文件
            let content = fs::read_to_string(file_path)?;
            let result = process_content(&content, Some(file_path));

            if cli.diff {
                // 显示差异
                show_diff(&content, &result)?;
            } else if cli.in_place {
                // 直接修改源文件
                fs::write(file_path, result)?;
            } else {
                // 输出到标准输出
                io::stdout().write_all(result.as_bytes())?;
            }
        }
        None => {
            // 从标准输入读取
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;

            let result = process_content(&buffer, None);

            if cli.diff {
                // 显示差异
                show_diff(&buffer, &result)?;
            } else {
                // 输出到标准输出
                io::stdout().write_all(result.as_bytes())?;
            }
        }
    }

    Ok(())
}
