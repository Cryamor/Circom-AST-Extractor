use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Grammar {
    pub productions: HashMap<String, Vec<Vec<String>>>,
    pub terminals: HashSet<String>,
    pub nonterminals: HashSet<String>,
    pub start_symbol: String,
    pub symbols: HashSet<String>,
    pub grammar_str : String,
}

#[derive(Debug, Clone)]
pub enum GrammarError {
    InvalidProduction(String),
    MissingArrow(String),
    EmptyProductionBody(String),
    UndefinedStartSymbol,
}

impl fmt::Display for GrammarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidProduction(msg) => write!(f, "Invalid production: {}", msg),
            Self::MissingArrow(line) => write!(f, "Missing -> in line: {}", line),
            Self::EmptyProductionBody(line) => write!(f, "Empty production body in line: {}", line),
            Self::UndefinedStartSymbol => write!(f, "Start symbol not defined"),
        }
    }
}

impl Error for GrammarError {}

impl Grammar {
    pub fn new(grammar_str: &str) -> Result<Self, GrammarError> {
        let mut productions = HashMap::new();
        let mut terminals = HashSet::new();
        let mut nonterminals = HashSet::new();
        let mut start_symbol = None;

        let mut token_dict = build_token_dict();

        for line in grammar_str.lines().filter(|l| !l.trim().is_empty()) {
            let (head, body) = line.split_once(" -> ")
                .ok_or_else(|| GrammarError::MissingArrow(line.to_string()))?;

            let head = head.trim().to_string();
            if start_symbol.is_none() {
                start_symbol = Some(head.clone());
            }

            // 头部符号强制视为非终结符
            nonterminals.insert(head.clone());

            let production_body: Vec<Vec<String>> = body.split('|')
                .map(|b| b.split_whitespace()
                    .map(|s| s.to_string())
                    .collect())
                .collect();

            // 校验空产生式
            if production_body.is_empty() {
                return Err(GrammarError::EmptyProductionBody(line.to_string()));
            }

            // 校验每个产生式体
            for symbols in &production_body {
                if symbols.contains(&"NULL".to_string()) && symbols.len() > 1 {
                    return Err(GrammarError::InvalidProduction(
                        format!("{} -> {}", head, symbols.join(" "))
                    ));
                }

                // 处理符号
                for symbol in symbols {
                    if symbol == "NULL" { continue; }

                    match token_dict.get(symbol.as_str()) {
                        Some(true) => { terminals.insert(symbol.clone()); }
                        Some(false) | None => { nonterminals.insert(symbol.clone()); }
                    }
                }
            }

            productions.entry(head.clone())
                .or_insert_with(Vec::new)
                .extend(production_body);
        }

        let start_symbol = start_symbol.ok_or(GrammarError::UndefinedStartSymbol)?;
        let symbols = terminals.union(&nonterminals).cloned().collect();

        Ok(Self {
            productions,
            terminals,
            nonterminals,
            start_symbol,
            symbols,
            grammar_str: grammar_str.to_string(),
        })
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nProductions: \n{:?}", self.productions)?;
        writeln!(f, "Terminals: \n{:?}", self.terminals)?;
        writeln!(f, "Non-terminals: \n{:?}", self.nonterminals)?;
        writeln!(f, "Start Symbol: {}", self.start_symbol);
        writeln!(f, "Count: {} {} {} {}",
            self.productions.len(),
            self.terminals.len(),
            self.nonterminals.len(),
            self.symbols.len())
    }
}

fn build_token_dict() -> HashMap<&'static str, bool> {
    let mut dict = HashMap::new();

    // 非终结符 (出现在产生式左侧)
    let non_terminals = vec![
        "PROGRAM", "HEADER", "PROG", "TEMPLATE_BLOCK", "TEMPLATE_STMT",
        "PARAM", "PARAM_", "TEMPLATE_CONTENT", "STMTS", "STMT", "SIGNAL_STMT",
        "ID_OR_ARRAY", "ID_OR_NUM", "COMPONENT_ID", "CI_", "VAR_STMT", "VAR_",
        "VAR_DEF", "VAR_ASSIGN", "ASSIGN_STMT", "EXPR", "C_ASSIGN_STMT", "IF_STMT",
        "M_IF", "N_IF", "M_ELSE", "WHILE_STMT", "M_BEFORE_WHILE", "M_AFTER_WHILE",
        "FOR_STMT", "FOR_COND", "M_FOR", "CONDITION", "COMPONENT_BLOCK", "ASSIGN",
        "C_ASSIGN", "REL", "PLUS", "OP", "COMPONENT_STMT", "COMP_", "COMP_DEF"
    ];
    for nt in non_terminals {
        dict.insert(nt, false);
    }

    // 终结符 (关键字/运算符/字面量)
    let terminals = vec![
        // 保留字
        "PRAGMA", "CIRCOM", "VERSION", "CUSTOM_TEMPLATES", "TEMPLATE", "COMPONENT",
        "MAIN", "SIGNAL", "INPUT", "OUTPUT", "VAR", "IF", "ELSE", "WHILE", "FOR",

        // 运算符
        "=", "+=", "-=", "*=", "/=", "QUOTIENT_ASSIGN", "%=", "&=", "BITWISE_OR_ASSIGN", "^=", "~=", "<<=", ">>=",
        "===", "<--", "-->", "<==", "==>", "==", "<", "<=", ">", ">=", "!=",
        "+", "-", "*", "QUOTIENT", "/", "%", "&", "BITWISE_OR", "~", "^", "<<", ">>",

        // 符号
        ";", ",", "(", ")", "{", "}", "[", "]", "DOT",

        // 字面量
        "ID", "NUM", "NULL"
    ];
    for t in terminals {
        dict.insert(t, true);
    }

    // 特殊处理复合符号
    dict.insert("BITWISE_OR", true);      // 对应 | 运算符
    dict.insert("BITWISE_OR_ASSIGN", true); // 对应 |= 运算符

    dict
}
// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_parsing() {
        let grammar_str = r"
            PROG -> PROGRAM HEADER SUBPROG .
            HEADER -> CONST_STATEMENT VARIABLE_STATEMENT
            SUBPROG -> BEGIN STATEMENT END
        ";

        let grammar = Grammar::new(grammar_str).unwrap();

        assert!(grammar.nonterminals.contains("PROG"));
        assert!(grammar.terminals.contains("BEGIN"));
        assert!(grammar.terminals.contains("PROGRAM"));
    }
}