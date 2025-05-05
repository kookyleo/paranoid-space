<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>PHP 语法综合示例</title>
</head>
<body>

<h1>PHP 语法综合示例</h1>

<?php
// 这是单行注释，使用双斜杠

# 这是另一种单行注释，使用井号

/*
这是多行注释
可以写多行说明
*/

// 变量示例
$intVar = 123;              // 整数
$floatVar = 3.14159;        // 浮点数
$stringVar1 = "双引号 String"; // 双引号字符串，支持变量解析
$stringVar2 = '单引号 String'; // 单引号字符串，内容原样输出
$boolVar = true;            // 布尔值
$nullVar = NULL;            // 空值

// 复杂字符串示例：heredoc 和 nowdoc
$heredocStr = <<<EOD
这是 heredoc 字符串，
支持变量解析：变量值是$stringVar1
EOD;

$nowdocStr = <<<'EOD'
这是 nowdoc 字符串，
不支持变量解析：变量值是 $stringVar1
EOD;

// 输出变量和字符串
echo "<h2>变量 Output 示例</h2>";
echo "整数变量: $intVar<br>";
echo "浮点数变量: $floatVar<br>";
echo "双引号字符串: $stringVar1<br>";
echo '单引号字符串:' . $stringVar2 . "<br>";
echo "布尔变量:";
echo $boolVar ? "真<br>" : "假<br>";
echo "NULL 变量:";
echo is_null($nullVar) ? "是 NULL<br>" : "不是 NULL<br>";
echo "<pre>heredoc 字符串内容: $heredocStr</pre>";
echo "<pre>nowdoc 字符串内容: $nowdocStr</pre>";

// 数组示例
$colors = array("红色", "绿色", "蓝色");
echo "<h2>数组示例</h2>";
echo "第一个颜色是：" . $colors[0] . "<br>";

// 关联数组
$person = [
    "姓名" => "张三",
    "年龄" => 28,
    "职业" => "程序员"
];
echo "姓名：" . $person["姓名"] . "<br>";

// 条件语句
echo "<h2>条件语句示例</h2>";
if ($intVar > 100) {
    echo "整数变量大于 100<br>";
} else {
    echo "整数变量小于等于 100<br>";
}

// 循环语句
echo "<h2>循环语句示例</h2>";
echo "颜色列表：<br>";
foreach ($colors as $color) {
    echo "- $color<br>";
}

// 函数定义和调用
echo "<h2>函数示例</h2>";
function greet($name) {
    return "你好，" . $name . "！";
}
echo greet("世界") . "<br>";

// 变量作用域示例
echo "<h2>变量作用域示例</h2>";
$globalVar = "全局变量";

function testScope() {
    global $globalVar; // 引用全局变量
    echo "函数内部访问：$globalVar<br>";
}
testScope();

// 输出 HTML 和 PHP 混合内容
?>

<p>这是 HTML 内容，下面是 PHP 输出的内容：</p>

<?php
echo "<strong>这是用 PHP 输出的加粗文字。</strong>";
echo "</body></html>";
