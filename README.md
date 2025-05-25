## Circom代码抽象语法树提取工具
### Features
对输入的Circom代码文件进行词法分析、语法分析，随后构建抽象语法树，以JSON格式输出。

### Usage

项目编译：
```bash
cargo build
```

可执行文件`target/debug/main`或`target/debug/main.exe`
将其移至根目录使用

命令行参数:
(输入circom文件路径) (输出json文件路径)

默认输入路径1.circom，默认输出路径out/文件名.json

```bash
main
main 1.circom 
main 1.circom 1.json
```

测试`/testcase`中的全部代码：

Windows: 使用`test_all.bat`：
```bash
./test_all.bat
```

Linux/MacOS: 使用`test_all.sh`：
```bash
chmod +x test_all.sh
./test_all.sh
```