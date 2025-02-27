use regex::Regex;
use crate::lexer::token::{Token, RULES};

pub struct Lexer {
    code: String,
    pub tokens: Vec<Token>,
    current_line: usize,
    current_column: usize,
    token_specification: Vec<(String, String)>,
}

impl Lexer {
    pub fn new(code: String) -> Self {
        let mut lexer = Lexer {
            code,
            tokens: Vec::new(),
            current_line: 1,
            current_column: 1,
            token_specification: Vec::new(),
        };

        // 循环插入规则到 HashMap
        for (token_type, pattern) in RULES {
            lexer.token_specification.push((token_type.to_string(), pattern.to_string()));
        }

        // println!("{:?}", lexer.token_specification);

        lexer
    }

    fn generate_token(&mut self, token_type: &str, value: &str) {
        if token_type == "NEWLINE" {
            self.current_line += 1;
            self.current_column = 0;
        } else if token_type != "SKIP" {
            let token = Token {
                token_type: token_type.to_string(),
                value: value.to_string(),
                line: self.current_line,
                column: self.current_column,
            };
            self.tokens.push(token);
        }
        self.current_column += value.len();
    }

    pub fn tokenize(&mut self) {
        let mut regex_parts = Vec::new();
        // 将每个规则格式化为带命名捕获组的正则表达式，用｜连接为一个完整的正则表达式，并编译为一个 Regex 对象
        for (key, val) in &self.token_specification {
            regex_parts.push(format!(r"(?P<{}>{})", key, val));
        }
        let regex = Regex::new(&regex_parts.join("|")).unwrap();

        let codestr = String::from(self.code.as_str());
        let mut pos = 0;
        while pos < self.code.len() {
            if let Some(captures) = regex.captures(&codestr[pos..]) {
                for (index, cap) in captures.iter().enumerate() {
                    if index == 0 {
                        continue;  // index=0是捕获集，但不包括名称，所以跳过找到真正的index，才能找到名称
                    }
                    if let Some(m) = cap {
                        let token_type = regex.capture_names()
                            .nth(index) // 根据index找到名称
                            .and_then(|name| name) // 如果没有名称，则返回 None
                            .unwrap_or_else(|| "UNKNOWN"); // 如果没有名称，使用默认值 "UNKNOWN"

                        let value = m.as_str();
                        self.generate_token(token_type, value);
                        pos += value.len();
                        break;
                    }
                }
            } else {
                panic!(
                    "Unexpected character {} at line {} column {}",
                    &self.code[pos..].chars().next().unwrap(),
                    self.current_line,
                    self.current_column
                );
            }
        }
    }

}
