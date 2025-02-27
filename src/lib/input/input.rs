
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

// pub fn read_multi_circom_file(file_path: &str) -> io::Result<Vec<String>> {
//
//
// }