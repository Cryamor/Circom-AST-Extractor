use std::collections::{HashMap, HashSet};
use std::fmt;
use log::{info, log};
use std::option::Option;
use crate::lexer::token::Token;
use crate::parser::grammar::{Grammar, GrammarError};

// LR1项目
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LR1Item {
    head: String,
    body: Vec<String>,
    dot_pos: usize,
    lookahead: String,
}

// 分析表动作
#[derive(Debug, Clone)]
enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
    Goto(usize),
    Conflict(Vec<Action>),
    Err,
}

// LR1分析表结构
#[derive(Debug)]
pub struct LR1Parser {
    grammar: Grammar,
    augmented_grammar: Grammar,
    first: HashMap<String, HashSet<String>>,
    follow: HashMap<String, HashSet<String>>,
    items: Vec<HashSet<LR1Item>>,
    action_table: HashMap<usize, HashMap<String, Action>>,
    goto_table: HashMap<usize, HashMap<String, usize>>,
    productions: Vec<(String, Vec<String>)>,  // 索引化产生式
}

#[derive(Debug, Clone)]
pub struct ReduceResult {
    pub head: String,
    pub body: Vec<String>,
    pub token: Vec<Token>,
}

#[derive(Debug)]
pub struct ReduceSymbol {
    symbol: String,
    token: Option<Token>,
}

// 错误处理
#[derive(Debug)]
pub enum ParseError {
    Conflict {
        symbol: String,
        state: usize,
        line: usize,
        column: usize,
    },
    InvalidAction {
        symbol: String,
        state: usize,
        line: usize,
        column: usize,
    },
    UnexpectedEnd,
    GrammarError(GrammarError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Conflict { symbol, state, line, column } =>
                write!(f, "Conflict at line {}-{}: state {} on {}", line, column, state, symbol),
            Self::InvalidAction { symbol, state, line, column } =>
                write!(f, "Invalid action at line {}-{}: state {} on {}", line, column, state, symbol),
            Self::UnexpectedEnd => write!(f, "Unexpected end of input"),
            Self::GrammarError(e) => write!(f, "Grammar error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<GrammarError> for ParseError {
    fn from(err: GrammarError) -> Self {
        Self::GrammarError(err)
    }
}

impl LR1Parser {
    pub fn new(grammar: Grammar) -> Result<Self, ParseError> {
        // 扩展文法
        let augmented_grammar = Self::build_augmented_grammar(&grammar)?;

        // 索引化产生式
        let productions = Self::index_productions(&augmented_grammar);

        // 计算FIRST/FOLLOW
        let (first, follow) = Self::compute_first_and_follow(&augmented_grammar);
        info!("{}", Self::print_first_and_follow(&first, &follow));

        // 构建项目集规范族
        let items = Self::build_items(&augmented_grammar, &first);

        // 构建分析表
        let (action_table, goto_table) = Self::build_parse_table(&augmented_grammar, &items, &first, &productions);

        info!("{}",Self::print_parse_table(&action_table, &goto_table));

        Ok(Self {
            grammar,
            augmented_grammar,
            first,
            follow,
            items,
            action_table,
            goto_table,
            productions,
        })
    }

    // 扩展文法
    fn build_augmented_grammar(grammar: &Grammar) -> Result<Grammar, GrammarError> {
        let new_start = format!("{}'", grammar.start_symbol);
        let mut new_grammar = format!("{} -> {}\n", new_start, grammar.start_symbol);
        new_grammar.push_str(&grammar.grammar_str);
        Grammar::new(&new_grammar)
    }

    // 序列化产生式
    fn index_productions(grammar: &Grammar) -> Vec<(String, Vec<String>)> {
        let mut indexed = Vec::new();
        for (head, bodies) in &grammar.productions {
            for body in bodies {
                indexed.push((head.clone(), body.clone()));
            }
        }

        println!("Indexed productions:");
        let mut i = 0;
        while i < indexed.len() {
            println!("{}: {:?} -> {:?}", i, indexed[i].0, &indexed[i].1);
            i += 1;
        }

        indexed
    }

    // 计算FIRST和FOLLOW集
    fn compute_first_and_follow(grammar: &Grammar) -> (HashMap<String, HashSet<String>>, HashMap<String, HashSet<String>>) {
        fn union(s1: &mut HashSet<String>, s2: &HashSet<String>) -> bool {
            let l1 = s1.len();
            s1.extend(s2.iter().cloned());

            l1 != s1.len()
        }

        let mut first: HashMap<String, HashSet<String>> = HashMap::new();
        for symbol in &grammar.symbols {
            first.insert(symbol.clone(), HashSet::new());
        }
        for terminal in &grammar.terminals {
            first.get_mut(terminal).unwrap().insert(terminal.clone());
        }

        let mut follow: HashMap<String, HashSet<String>> = HashMap::new();
        for nt in &grammar.nonterminals {
            follow.insert(nt.clone(), HashSet::new());
        }
        follow.get_mut(&grammar.start_symbol).unwrap().insert("#".to_string());

        let mut updated = false;
        loop {
            updated = false;
            for (head, bodies) in &grammar.productions {
                for body in bodies {
                    for symbol in body {
                        if symbol != "NULL" {
                            let symbol_first = first.get(symbol).unwrap().clone();
                            let mut temp_set: HashSet<String> = symbol_first
                                .iter()
                                .filter(|s| *s != "NULL")
                                .cloned()
                                .collect();
                            updated |= union(first.get_mut(head).unwrap(), &temp_set);
                            if !symbol_first.contains("NULL") {
                                break;
                            }
                        }
                        else {
                            updated |= union(first.get_mut(head).unwrap(), &HashSet::from_iter(std::iter::once("NULL".to_string())));
                        }
                    }
                    // 如果整个产生式体都是空符号，将空符号加入FIRST
                    if body.iter().all(|s| s == "NULL") {
                        updated |= union(first.get_mut(head).unwrap(), &HashSet::from_iter(std::iter::once("NULL".to_string())));
                    }

                    // 处理FOLLOW集合
                    let mut aux = follow.get(head).unwrap().clone();
                    for symbol in body.iter().rev() {
                        if symbol == "NULL" { continue; }
                        if follow.contains_key(symbol) {
                            let symbol_follow = follow.get_mut(symbol).unwrap();
                            let temp_set: HashSet<String> = aux
                                .iter()
                                .filter(|s| *s != "NULL")
                                .cloned()
                                .collect();
                            updated |= union(symbol_follow, &temp_set);
                        }
                        let symbol_first = first.get(symbol).unwrap().clone();
                        if symbol_first.contains("NULL") {
                            aux = symbol_first.union(&aux).cloned().collect();
                        } else {
                            aux = symbol_first;
                        }
                    }
                }
            }
            if !updated { break; }
        }

        (first, follow)
    }

    fn print_first_and_follow(first: &HashMap<String, HashSet<String>>, follow: &HashMap<String, HashSet<String>>) -> String {
        let mut s = String::new();
        s += "\nFirst:\n";
        for f in first {
            s += format!("{}: {:?}\n", f.0, f.1).as_str();
        }
        s += "Follow:\n";
        for f in follow {
            s += format!("{}: {:?}\n", f.0, f.1).as_str();
        }
        s
    }

    // 构建项目集规范族
    fn build_items(grammar: &Grammar, first: &HashMap<String, HashSet<String>>, ) -> Vec<HashSet<LR1Item>> {
        let mut items = vec![];
        let initial = Self::initial_item(grammar);
        items.push(Self::closure(&initial, grammar, first));

        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..items.len() {
                for symbol in &grammar.symbols {
                    let goto = Self::goto(&items[i], symbol, grammar, first);
                    if !goto.is_empty() && !items.contains(&goto) {
                        items.push(goto);
                        changed = true;
                    }
                }
            }
        }
        items
    }

    fn initial_item(grammar: &Grammar) -> HashSet<LR1Item> {
        HashSet::from_iter(vec![LR1Item {
            head: grammar.start_symbol.clone(),
            body: vec![grammar.start_symbol.clone().replace("'","")],
            dot_pos: 0,
            lookahead: "#".to_string(),
        }])
    }

    // 闭包计算
    fn closure(items: &HashSet<LR1Item>, grammar: &Grammar, first: &HashMap<String, HashSet<String>>, ) -> HashSet<LR1Item> {
        let mut closure = items.clone();
        let mut changed = true;

        while changed {
            changed = false;

            for item in closure.clone() {
                if let Some(symbol) = item.body.get(item.dot_pos) {  // .在产生式中并且不是最后一个元素
                    if grammar.nonterminals.contains(symbol) {  // .后是非终结符
                        if item.body.len() == item.dot_pos + 1 {  // 处理A -> ... .B的情况
                            for production in grammar.productions.get(symbol).unwrap() {
                                let new_item = LR1Item {
                                    head: symbol.clone(),
                                    body: production.clone(),
                                    dot_pos: 0,
                                    lookahead: item.lookahead.clone(),
                                };
                                if !closure.contains(&new_item) {
                                    closure.insert(new_item);
                                    changed = true;
                                }
                            }
                        }
                        else {  // 处理A -> ... .BC的情况
                            let beta = &item.body[item.dot_pos+1..];
                            let lookaheads = Self::compute_lookaheads(beta, &item.lookahead, first);
                            for production in grammar.productions.get(symbol).unwrap() {
                                for la in &lookaheads {
                                    let new_item = LR1Item {
                                        head: symbol.clone(),
                                        body: production.clone(),
                                        dot_pos: 0,
                                        lookahead: la.clone(),
                                    };
                                    if !closure.contains(&new_item) {
                                        closure.insert(new_item);
                                        changed = true;
                                    }
                                }
                            }
                        }

                    }
                }
            }
        }
        closure
    }

    fn compute_lookaheads(beta: &[String], lookahead: &str, first: &HashMap<String, HashSet<String>>, ) -> HashSet<String> {
        let mut lookaheads = HashSet::new();
        let mut has_null = true;

        for symbol in beta {
            lookaheads.extend(first[symbol].iter().filter(|s| *s != "NULL").cloned());
            if !first[symbol].contains("NULL") {
                has_null = false;
                break;
            }
        }
        lookaheads.remove("NULL");

        if has_null {
            lookaheads.insert(lookahead.to_string());
        }
        lookaheads
    }

    // GOTO计算
    fn goto(
        items: &HashSet<LR1Item>,
        symbol: &str,
        grammar: &Grammar,
        first: &HashMap<String, HashSet<String>>,
    ) -> HashSet<LR1Item> {
        let mut goto_items = HashSet::new();

        // 遍历项目集中的每个项目
        for item in items {
            // 仅处理可以转移的项目
            if item.dot_pos < item.body.len() && item.body[item.dot_pos] == symbol {
                let new_item = LR1Item {
                    dot_pos: item.dot_pos + 1,
                    ..item.clone()
                };
                goto_items.insert(new_item);
            }
        }

        // 计算闭包
        Self::closure(&goto_items, grammar, first)
    }

    // 构建分析表
    fn build_parse_table(
        grammar: &Grammar,
        items: &[HashSet<LR1Item>],  // items 是状态集合的列表
        first: &HashMap<String, HashSet<String>>,
        productions: &Vec<(String, Vec<String>)>,
    ) -> (HashMap<usize, HashMap<String, Action>>, HashMap<usize, HashMap<String, usize>>) {
        let mut action_table = HashMap::new();
        let mut goto_table = HashMap::new();

        // 遍历所有状态（每个状态是一个项目集）
        for (state, state_items) in items.iter().enumerate() {
            let mut actions = HashMap::new();
            let mut gotos = HashMap::new();

            // 处理当前状态中的每个项目
            for item in state_items {
                // 处理移进动作
                // 如果点 '.' 不在产生式的末尾
                if item.dot_pos < item.body.len() {
                    // 获取点后的符号
                    let symbol = &item.body[item.dot_pos];
                    // 如果点后的符号是终结符
                    if grammar.terminals.contains(symbol) {
                        // 查找转移后的状态
                        let goto_result = Self::goto(state_items, symbol, grammar, first);

                        // 在状态列表中找到对应的状态索引
                        if let Some(next_state) = items.iter().position(|s| *s == goto_result) {
                            actions.insert(symbol.clone(), Action::Shift(next_state));
                        }
                    }

                }
                else {
                    // 处理规约
                    if item.head == grammar.start_symbol {
                        actions.insert("#".to_string(), Action::Accept);
                    }
                    else {
                        let production_index = productions
                            .iter()
                            .position(|p| p.0 == item.head && p.1 == item.body)
                            .unwrap();

                        match actions.entry(item.lookahead.clone()) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(Action::Reduce(production_index));
                                info!("state{} reduce {:?}  {}:{:?}", state, item, production_index, productions[production_index]);
                            },
                            std::collections::hash_map::Entry::Occupied(mut e) => {
                                info!("Occupied! state{} {:?}", state, item);
                                info!("{:?}", actions);
                                panic!("Occupied!");
                            }
                        }
                    }
                }
            }

            // 处理GOTO（非终结符转移）
            for nt in &grammar.nonterminals {
                let goto_result = Self::goto(state_items, nt, grammar, first);

                if let Some(next_state) = items.iter().position(|s| *s == goto_result) {
                    gotos.insert(nt.clone(), next_state);
                }
            }

            action_table.insert(state, actions);
            goto_table.insert(state, gotos);
        }

        (action_table, goto_table)
    }

    fn print_parse_table(action_table: &HashMap<usize, HashMap<String, Action>>, goto_table: &HashMap<usize, HashMap<String, usize>>) -> String {
        let mut s = String::new();
        s += "\nAction table:\n";
        let mut keys: Vec<_> = action_table.keys().cloned().collect();
        keys.sort();  // 将外层 HashMap 的键提取出来并排序
        for key in keys {
            if let Some(sub_map) = action_table.get(&key) {
                // 将内层 HashMap 的键提取出来并排序
                let mut sub_keys: Vec<_> = sub_map.keys().cloned().collect();
                sub_keys.sort();
                s += format!("{}: (", key).as_str();
                for (i, sub_key) in sub_keys.iter().enumerate() {
                    if i > 0 {
                        s += ", ";
                    }
                    s += format!("\"{}\": {:?}", sub_key, sub_map[sub_key]).as_str();
                }
                s += ")\n";
            }
        }
        s += "\nGoto table:\n";
        let mut keys: Vec<_> = goto_table.keys().cloned().collect();
        keys.sort();  // 将外层 HashMap 的键提取出来并排序
        for key in keys {
            if let Some(sub_map) = goto_table.get(&key) {
                // 将内层 HashMap 的键提取出来并排序
                let mut sub_keys: Vec<_> = sub_map.keys().cloned().collect();
                sub_keys.sort();
                s += format!("{}: (", key).as_str();
                for (i, sub_key) in sub_keys.iter().enumerate() {
                    if i > 0 {
                        s += (", ");
                    }
                    s += format!("\"{}\": {:?}", sub_key, sub_map[sub_key]).as_str();
                }
                s += (")\n");
            }
        }

        s
    }

    // 执行语法分析
    pub fn run_parse(&self, tokens: &[Token]) -> Result<Vec<ReduceResult>, ParseError> {
        let mut input = tokens.to_vec();
        input.push(Token { // 添加结束标记
            token_type: "#".to_string(),
            value: "#".to_string(),
            line: 0,
            column: 0,
            start: 0,
            end: 0,
        });

        let mut state_stack = vec![0];
        let mut symbol_stack: Vec<ReduceSymbol> = vec![];
        let mut index = 0;
        let mut results: Vec<ReduceResult> = vec![];


        loop {
            let state = *state_stack.last().ok_or(ParseError::UnexpectedEnd)?;
            let token = input.get(index).ok_or(ParseError::UnexpectedEnd)?;

            let current_symbol = match token.token_type.as_str() {
                "SEMICOLON" => ";", "INCREMENT" => "++", "DECREMENT" => "--", "PLUS_ASSIGN" => "+=", "MINUS_ASSIGN" => "-=",
                "MULTIPLY_ASSIGN" => "*=", "DIVIDE_ASSIGN" => "/=", "MODULUS_ASSIGN" => "%=", "BITWISE_AND_ASSIGN" => "&=",
                "BITWISE_XOR_ASSIGN" => "^=", "BITWISE_NOT_ASSIGN" => "~=", "LEFT_SHIFT_ASSIGN" => "<<=", "RIGHT_SHIFT_ASSIGN" => ">>=",
                "PLUS" => "+", "MINUS" => "-", "MULTIPLY" => "*", "DIVIDE" => "/", "MODULUS" => "%", "LOGICAL_AND" => "&&",
                "BITWISE_AND" => "&", "BITWISE_XOR" => "^", "BITWISE_NOT" => "~", "LEFT_SHIFT" => "<<", "RIGHT_SHIFT" => ">>",
                "CIRCOM_L_ASSIGN" => "<--", "CIRCOM_R_ASSIGN" => "-->", "CIRCOM_L_CONSTRAINT_ASSIGN" => "<==", "CIRCOM_R_CONSTRAINT_ASSIGN" => "==>",
                "CIRCOM_CONSTRAINT" => "===", "EQUAL" => "==", "NOT_EQUAL" => "!=", "LESS_THAN_OR_EQUAL" => "<=", "GREATER_THAN_OR_EQUAL" => ">=",
                "LESS_THAN" => "<", "GREATER_THAN" => ">", "ASSIGN" => "=", "LOGICAL_NOT" => "!",
                "LPAREN" => "(", "RPAREN" => ")", "LBRACKET" => "[", "RBRACKET" => "]", "COMMA" => ",",
                "LBRACE" => "{", "RBRACE" => "}", "COLON" => ":", "QUESTION_MARK" => "?",

                t => t,
            };

            info!("{}", format!("States:{:?} Symbols:{:?} Current:{} Action: {:?}",
                state_stack, symbol_stack, current_symbol, self.action_table.get(&state).unwrap().get(current_symbol).unwrap()).as_str());

            match self.action_table.get(&state)
                .and_then(|actions| actions.get(current_symbol))
            {
                Some(Action::Shift(next_state)) => {
                    info!("{}", format!("Shift {}", next_state).as_str());
                    state_stack.push(*next_state);
                    // symbol_stack.push(token.token_type.clone());
                    symbol_stack.push(ReduceSymbol{
                        symbol: token.clone().token_type,
                        token: Option::from(token.clone()),
                    });
                    index += 1;
                }

                Some(Action::Reduce(prod_index)) => {
                    let (head, body) = &self.productions[*prod_index];

                    info!("{}", format!("Reduce {} -> {}", head, body.join(" ")).as_str());

                    let mut rr = ReduceResult{
                        head: head.clone(),
                        body: body.clone(),
                        token: vec![],
                    };

                    // 弹出栈顶len(body)个元素
                    if !body.contains(&"NULL".to_string()) {
                        for _ in 0..body.len() {
                            state_stack.pop();
                            let s = symbol_stack.pop();
                            let t = s.unwrap().token;
                            if t.is_some() {
                                rr.token.push(t.unwrap().clone());
                            }
                        }
                    }
                    results.push(rr);

                    // 获取新状态
                    let new_state = *state_stack.last().unwrap();
                    let goto_state = *self.goto_table[&new_state].get(head)
                        .ok_or(ParseError::InvalidAction {
                            symbol: head.clone(),
                            state: new_state,
                            line: token.line,
                            column: token.column,
                        })?;

                    state_stack.push(goto_state);
                    symbol_stack.push(ReduceSymbol{
                        symbol: head.clone(),
                        token: None,
                    });

                }

                Some(Action::Accept) => break,

                Some(Action::Conflict(_)) => return Err(ParseError::Conflict {
                    symbol: current_symbol.to_string(),
                    state,
                    line: token.line,
                    column: token.column,
                }),

                None => return Err(ParseError::InvalidAction {
                    symbol: current_symbol.to_string(),
                    state,
                    line: token.line,
                    column: token.column,
                }),
                Some(Action::Goto(_)) => {}
                _ => {}
            }
        }

        Ok(results)
    }
}

