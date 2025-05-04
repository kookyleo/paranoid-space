//! Module level inner doc comment 模块级别的内部文档注释
#![allow(dead_code)] // Allow unused code for example purposes

use std::fmt::{self, Display}; // Use statement 使用语句

// Simple line comment 行注释
/* Block comment
   跨越多行的 Block 注释
*/

/// Outer line doc comment 函数外部文档注释
/** Outer block doc comment
 * 函数的块文档注释
 */
fn example_function<T: Display>(item: T) { // Generic function 泛型函数
    //! Function inner line doc comment 函数内部文档注释
    /*! Function inner block doc comment */

    let message = "Hello, 世界！ Contains escapes: \n\t\""; // Regular string 普通字符串
    let raw_message = r#"Raw string with "quotes" and \escapes 原始字符串"#; // Raw string 原始字符串
    
    let bytes = b"ASCII only byte 字符串"; // Byte string 字节串
    let raw_bytes = br#"Raw ASCII only bytes"#; // Raw byte string 原始字节串

    const PI: f64 = 3.14159; // Constant 常量
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0); // Static 静态变量

    let calculation = PI * 2.0; // Some calculation

    // Another line comment 另一个行注释
    println!("Item: {}", item); // Using generic item 使用泛型参数
    println!("Raw: {}", raw_message);
    println!("Bytes: {:?}", bytes);
    println!("Raw Bytes: {:?}", raw_bytes);
    println!("Calculation: {}", calculation);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst); // Modify static

    /* Multiline block comment
       containing code like let y = 5; */

    let closure = |x: i32| x + 1; // Closure 闭包
    let _result = closure(5);
}

/// Struct with generic parameter and derive attribute 带有泛型和 derive 属性的结构体
#[derive(Debug, Clone, Copy)]
struct Point<T> {
    x: T, // Generic field 泛型字段
    y: T, // Field with comment 带有注释的字段
}

impl<T: Display> Point<T> {
    /// Associated function for Point Point 的关联函数
    fn new(x: T, y: T) -> Self {
        Point { x, y } // Create new instance 创建新实例
    }

    /// Method for Point Point 的方法
    fn display(&self) { // Self reference 自引用
        println!("Point: ({}, {})", self.x, self.y); // Print coordinates 打印坐标
    }
}

/// Doc comment for enum 枚举的文档注释
#[allow(unused)]
enum Status {
    Ok, // Variant 1
    Error(String), /* Variant 2 comment */
}

/// Trait 定义 Trait Definition
trait Summable {
    /// Method required by trait Trait 要求的方法
    fn sum(&self) -> i32; // Method signature 方法签名
}

/// Trait implementation for Point<i32> 为 Point<i32> 实现 Trait
impl Summable for Point<i32> {
    fn sum(&self) -> i32 {
        self.x + self.y // Calculation 计算
    }
}

/// Macro definition 宏定义 (simple example)
macro_rules! say_hello {
    () => {
        println!("Hello from macro! 来自宏的问候！");
    };
}

/// Module definition 模块定义
mod inner_module {
    /// Function inside module 模块内部函数
    pub(crate) fn inner_function() {
        println!("Inside inner module! 在内部模块中！"); // Print message 打印消息
    }
}

// Final line comment 结束行注释