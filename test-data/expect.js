// Single Line 声明变量并赋值
var num = 42; // 整数
let pi = 3.14159; // 浮点数
const greeting = "Hello 世界!"; // 常量字符串

/*
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

// 字符串示例：单引号、双引号、转义字符
let singleQuoteStr = '这是单引号 String';
let doubleQuoteStr = "这是双引号字符串";
let escapedStr = "He said, \"JavaScript 好有趣!\""; // 转义双引号
let multiLineStr = "这是一个长字符串，\
可以用反斜杠换行"; // 字符串换行

// 布尔值和空值
let isActive = true;
let isComplete = false;
let nothing = null;
let notDefined; // undefined

// 数组和对象字面量
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

// 条件语句
if (num > 40) {
  console.log("num 大于 40");
} else if (num === 40) {
  console.log("num 等于 40");
} else {
  console.log("num 小于 40");
}

// 循环示例
for (let i = 0; i < arr.length; i++) {
  console.log(`arr[${i}] = ${arr[i]}`);
}

// while 循环
let count = 3;
while (count > 0) {
  console.log(`倒计时：${count}`);
  count--;
}

// try...catch 异常处理
try {
  throw new Error("这是一个错误");
} catch (e) {
  console.error("捕获到错误:", e.message);
} finally {
  console.log("无论是否出错，都会执行");
}

// 箭头函数和默认参数
const add = (a = 0, b = 0) => a + b;
console.log(`1 + 2 = ${add(1, 2)}`);

// 解构赋值
const [first, second] = arr;
const { name, age } = obj;
console.log(`名字：${name}, 年龄：${age}`);

// 使用模板字符串多行和表达式
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
