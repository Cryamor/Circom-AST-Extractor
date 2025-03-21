use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}
pub const RULES: &[(&str, &str)] = &[
    // 关键字
    ("PRAGMA", r"\bpragma\b"),
    ("CIRCOM", r"\bcircom\b"),
    ("TEMPLATE", r"\btemplate\b"),
    ("COMPONENT", r"\bcomponent\b"),
    ("INPUT", r"\binput\b"),
    ("OUTPUT", r"\boutput\b"),
    ("SIGNAL", r"\bsignal\b"),
    ("PUBLIC", r"\bpublic\b"),
    ("VAR", r"\bvar\b"),
    ("FUNCTION", r"\bfunction\b"),
    ("RETURN", r"\breturn\b"),
    ("IF", r"\bif\b"),
    ("ELSE", r"\belse\b"),
    ("FOR", r"\bfor\b"),
    ("WHILE", r"\bwhile\b"),
    ("DO", r"\bdo\b"),
    ("LOG", r"\blog\b"),
    ("ASSERT", r"\bassert\b"),
    ("INCLUDE", r"\binclude\b"),
    ("PARALLEL", r"\bparallel\b"),
    ("BUS", r"\bbus\b"),
    ("CUSTOM_TEMPLATES", r"\bcustom_templates\b"),
    ("MAIN", r"\bmain\b"),

    // 运算符
    ("INCREMENT", r"\+\+"),        // 自增 ++
    ("DECREMENT", r"--"),          // 自减 --
    ("PLUS_ASSIGN", r"\+="),       // 加等于 +=
    ("MINUS_ASSIGN", r"-="),       // 减等于 -=
    ("MULTIPLY_ASSIGN", r"\*="),   // 乘等于 *=
    ("DIVIDE_ASSIGN", r"/="),      // 除等于 /=
    ("MODULUS_ASSIGN", r"%="),     // 取模等于 %=
    ("QUOTIENT_ASSIGN", r"\\="),   // 取商等于 \=
    ("BITWISE_AND_ASSIGN", r"&="),         // 位与等于 &=
    ("BITWISE_OR_ASSIGN", r"\|="),         // 位或等于 |=
    ("BITWISE_XOR_ASSIGN", r"\^="),        // 位异或等于 ^=
    ("BITWISE_NOT_ASSIGN", r"~="),         // 位非等于 ~=
    ("LEFT_SHIFT_ASSIGN", r"<<="),         // 左移等于 <<=
    ("RIGHT_SHIFT_ASSIGN", r">>="),        // 右移等于 >>=

    ("PLUS", r"\+"),               // 加号 +
    ("MINUS", r"-"),               // 减号 -
    ("MULTIPLY", r"\*"),           // 乘号 *
    ("DIVIDE", r"/"),              // 除号 /
    ("MODULUS", r"%"),             // 取模 %
    ("QUOTIENT", r"\\"),           // 取商 \
    ("LOGICAL_AND", r"&&"),        // 逻辑与 &&
    ("LOGICAL_OR", r"\|\|"),       // 逻辑或 ||
    ("BITWISE_AND", r"&"),         // 位与 &
    ("BITWISE_OR", r"\|"),         // 位或 |
    ("BITWISE_XOR", r"\^"),        // 位异或 ^
    ("BITWISE_NOT", r"~"),         // 位非 ~
    ("LEFT_SHIFT", r"<<"),         // 左移 <<
    ("RIGHT_SHIFT", r">>"),        // 右移 >>

    // 约束运算符
    ("CIRCOM_L_ASSIGN", r"<--"),
    ("CIRCOM_R_ASSIGN", r"-->"),
    ("CIRCOM_L_CONSTRAINT_ASSIGN", r"<=="),
    ("CIRCOM_R_CONSTRAINT_ASSIGN", r"==>"),
    ("CIRCOM_CONSTRAINT", r"==="),

    // 比较运算符
    ("EQUAL", r"=="),              // 等于 ==
    ("NOT_EQUAL", r"!="),          // 不等于 !=
    ("LESS_THAN_OR_EQUAL", r"<="), // 小于等于 <=
    ("GREATER_THAN_OR_EQUAL", r">="), // 大于等于 >=
    ("LESS_THAN", r"<"),           // 小于 <
    ("GREATER_THAN", r">"),        // 大于 >

    ("ASSIGN", r"="),              // 赋值 =

    ("LOGICAL_NOT", r"!"),         // 逻辑非 !

    // 括号和其他符号
    ("LPAREN", r"\("),             // 左括号 (
    ("RPAREN", r"\)"),             // 右括号 )
    ("LBRACE", r"\{"),             // 左大括号 {
    ("RBRACE", r"\}"),             // 右大括号 }
    ("LBRACKET", r"\["),           // 左中括号 [
    ("RBRACKET", r"\]"),           // 右中括号 ]
    ("SEMICOLON", r";"),           // 分号 ;
    ("COMMA", r","),               // 逗号 ,
    ("COLON", r":"),               // 冒号 :
    ("DOT", r"\."),                // 点 .
    ("QUESTION_MARK", r"\?"),      // 问号 ?

    // 标识符
    ("VERSION", r"\b\d+(\.\d+)+\b"),
    ("ID", r"\b_[a-zA-Z][a-zA-Z0-9_$]*\b|\b[a-zA-Z][a-zA-Z0-9_$]*\b"),
    ("NUM", r"\b\d+\b"),

    // 其他
    ("NEWLINE", r"\n"),
    ("SKIP", r"[ \t]+"),

];