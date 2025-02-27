
use std::fs;
use std::io;

/// 读取指定路径的 .circom 文件内容
/// # 参数
/// - `file_path`: 输入的 .circom 文件路径
/// # 返回
/// - `Ok(String)`: 成功时返回处理后的字符串
/// - `Err(io::Error)`: 文件读取失败时返回错误信息
pub fn read_circom_file(file_path: &str) -> io::Result<String> {
    let content = fs::read_to_string(file_path)?;

    // 移除缩进符
    let cleaned_content = content
        .replace("\t", "");

    let cleaned_content = remove_line_comments(cleaned_content);
    let cleaned_content = remove_block_comments(cleaned_content);

    Ok(cleaned_content)
}

/// 读取指定路径的 .circom 文件内容，并移除换行符
/// # 参数
/// - `file_path`: 输入的 .circom 文件路径
/// # 返回
/// - `Ok(String)`: 成功时返回处理后的字符串
/// - `Err(io::Error)`: 文件读取失败时返回错误信息
pub fn read_circom_file_rm_newline(file_path: &str) -> io::Result<String> {
    let content = fs::read_to_string(file_path)?;

    // 移除所有换行符（包括 Windows 的 `\r\n` 和 Unix 的 `\n`）
    // 移除缩进符
    let cleaned_content = content
        .replace("\r\n", "")  // 处理 Windows 换行符
        .replace('\n', "")   // 处理 Unix 换行符
        .replace("\t", "");

    Ok(cleaned_content)
}

/// 移除 // 后的本行内容（保留换行符）
fn remove_line_comments(content: String) -> String {
    content
        .lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[0..pos]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// 移除 /* */ 包裹的内容，但保留换行符
fn remove_block_comments(content: String) -> String {
    let mut result = String::new();
    let mut in_comment = false;

    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            // 进入块注释
            in_comment = true;
            chars.next(); // 跳过 '*' 字符
        } else if c == '*' && chars.peek() == Some(&'/') {
            // 退出块注释
            in_comment = false;
            chars.next(); // 跳过 '/' 字符
        } else if !in_comment {
            // 不在注释中，将字符添加到结果中
            result.push(c);
        } else if c == '\n' {
            // 在注释中，但保留换行符
            result.push(c);
        }
    }

    result
}

// pub fn read_multi_circom_file(file_path: &str) -> io::Result<Vec<String>> {
//
//
// }