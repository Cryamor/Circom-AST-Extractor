## Circom代码抽象语法树提取工具
### Features
- 对输入的Circom代码文件进行词法分析、语法分析，随后构建抽象语法树，以JSON格式输出。
- 对文法规则错误和代码语法错误报错，在日志中查看
- 日志记录于`/logs/1.log`，还会记录词法分析与语法分析的各步骤信息
- 语法分析器缓存于`/cache/parser_cache.json`，在不改变文法规则的情况下无需再构建一遍
- 默认文法规则为`grammar/grammar.txt`

### Usage

项目编译&运行：
```bash
cargo build
cargo run
```

可执行文件`target/debug/Circom_AST_Extractor`或`target/debug/Circom_AST_Extractor.exe`
将其移至根目录使用

命令行参数:
(输入circom文件路径) (输出json文件路径)

默认输入路径1.circom，默认输出路径out/文件名.json

```bash
# Linux/MacOS
Circom_AST_Extractor
Circom_AST_Extractor 1.circom 
Circom_AST_Extractor 1.circom 1.json

# Windows
Circom_AST_Extractor.exe
Circom_AST_Extractor.exe 1.circom 
Circom_AST_Extractor.exe 1.circom 1.json
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

`AstBuilder`/`AstBuilder.exe`是对官方Circom Compiler逆向工程得到的AST提取工具，使用方式：

```bash
# Linux/MacOS
./Astbuilder 1.circom 1.json  

# Windows
./Astbuilder.exe 1.circom 1.json  
```