use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use paranoid_space::spacing;

#[derive(Parser)]
#[command(name = "paranoid-space")]
#[command(about = "auto add space between full-width and half-width characters")]
#[command(version)]
struct Cli {
    /// 需要处理的文件路径，如不指定则从标准输入读取
    file: Option<PathBuf>,

    /// 是否直接修改源文件（如不指定则输出到标准输出）
    #[arg(short = 'i')]
    in_place: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.file {
        Some(file_path) => {
            // 处理文件
            let content = fs::read_to_string(&file_path)?;
            let result = spacing(&content);

            if cli.in_place {
                // 直接修改源文件
                fs::write(file_path, result)?;
            } else {
                // 输出到标准输出
                io::stdout().write_all(result.as_bytes())?;
            }
        },
        None => {
            // 从标准输入读取
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            
            let result = spacing(&buffer);
            io::stdout().write_all(result.as_bytes())?;
        }
    }

    Ok(())
}
