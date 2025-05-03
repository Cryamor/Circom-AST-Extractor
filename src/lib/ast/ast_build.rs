use std::ops::Range;
use serde::de::Unexpected::Option;
use serde::Serialize;
use crate::ast::ast::*;
use crate::ast::ast::Definition::{Function, Template};
use crate::ast::ast::Expression::{Call, InfixOp, Number, Variable};
use crate::ast::ast::Sign::NoSign;
use crate::ast::ast::Statement::{Block, IfThenElse, Return, Substitution, While};
use crate::lexer::token::Token;
use crate::parser::lr1::ReduceResult;

fn process_version(version: &str) -> Version {
    let parts: Vec<&str> = version.split('.').collect();
    let p1 = parts[0].parse::<usize>().unwrap();
    let p2 = parts[1].parse::<usize>().unwrap();
    let p3 = parts[2].parse::<usize>().unwrap();

    (p1, p2, p3)
}

fn process_signal_stmt(r: &ReduceResult,
                       id_stack: &mut Vec<Token>,
                       block_stack: &mut Vec<Statement>)
{
    let start = r.token.iter().find(|t| t.token_type == "SIGNAL").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap();
    let meta = Meta::new(start.clone(), end.clone());

    let mut xtype: VariableType;
    if r.body.contains(&"INPUT".to_string()) {
        xtype = VariableType::Signal(SignalType::Input, vec![]);
    }
    else if r.body.contains(&"OUTPUT".to_string()) {
        xtype = VariableType::Signal(SignalType::Output, vec![]);
    }
    else {
        xtype = VariableType::Signal(SignalType::Intermediate, vec![]);
    }

    let id = id_stack.pop().unwrap();

    // "Declaration"
    let dec = Statement::Declaration {
        meta: meta.clone(),
        xtype: xtype.clone(),
        name: id.value.clone(),
        dimensions: vec![],
        is_constant: true,
    };

    // "InitializationBlock"
    let i_b = Statement::InitializationBlock {
        meta: meta.clone(),
        xtype: xtype.clone(),
        initializations: vec![dec],
    };

    block_stack.push(i_b);
}

fn process_assign_stmt(r: &ReduceResult,
                       id_stack: &mut Vec<Token>,
                       assign_stack: &mut Vec<Token>,
                       expr_stack: &mut Vec<Expression>,
                       block_stack: &mut Vec<Statement>)
{
    let i = id_stack.pop().unwrap();
    let var_name = i.value.clone();
    let start = i.start.clone();
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap().clone();
    let mut meta = Meta::new(start, end);
    let ex = expr_stack.pop().unwrap();
    let assign = assign_stack.pop().unwrap();
    let mut rhe: Expression = ex.clone();
    let mut infixop: ExpressionInfixOpcode = ExpressionInfixOpcode::Add;
    if assign.value == "=" {
        let substitution: Statement = Substitution {
            meta: meta.clone(),
            var: var_name.clone(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: rhe.clone(),
        };
        block_stack.push(substitution);
    }
    else {
        match assign.value.as_str() {
            "+=" => infixop = ExpressionInfixOpcode::Add,
            "-=" => infixop = ExpressionInfixOpcode::Sub,
            "*=" => infixop = ExpressionInfixOpcode::Mul,
            "/=" => infixop = ExpressionInfixOpcode::Div,
            "%=" => infixop = ExpressionInfixOpcode::Mod,
            "QUOTIENT_ASSIGN" => infixop = ExpressionInfixOpcode::IntDiv,
            "&=" => infixop = ExpressionInfixOpcode::BitAnd,
            "BITWISE_OR_ASSIGN" => infixop = ExpressionInfixOpcode::BitOr,
            "^=" => infixop = ExpressionInfixOpcode::BitXor,
            "<<=" => infixop = ExpressionInfixOpcode::ShiftL,
            ">>=" => infixop = ExpressionInfixOpcode::ShiftR,
            _ => {}
        }

        let lhe = Variable {
            meta: meta.clone(),
            name: var_name.clone(),
            access: vec![],
        };

        let rhe_1: Expression = InfixOp {
            meta: meta.clone(),
            lhe: Box::new(lhe.clone()),
            infix_op: infixop.clone(),
            rhe: Box::new(rhe.clone()),
        };

        let substitution: Statement = Substitution {
            meta: meta.clone(),
            var: var_name.clone(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: rhe_1.clone(),
        };

        block_stack.push(substitution);
    }
}

fn process_var_def(r: &ReduceResult,
                   id_stack: &mut Vec<Token>,
                   expr_stack: &mut Vec<Expression>,
                   var_stack: &mut Vec<Statement>)
{
    let i = id_stack.pop().unwrap();
    let var_name = i.value.clone();
    let start =  i.start.clone();
    let end = i.end.clone();
    let mut meta = Meta::new(start, end);
    let mut ex: Expression;

    // "Declaration"
    let dec = Statement::Declaration {
        meta: meta.clone(),
        xtype: VariableType::Var,
        name: var_name.clone(),
        dimensions: vec![],
        is_constant: true,
    };
    var_stack.push(dec);

    if r.body.len() == 1 {
        // var a;
        let num = Num::new(NoSign, vec![]);
        ex = Number(meta.clone(), num.clone());
    }
    else {
        // var a = 0;
        ex = expr_stack.pop().unwrap();
    }
    // "Substitution"
    let sub = Substitution {
        meta: meta.clone(),
        var: var_name.clone(),
        access: vec![],
        op: AssignOp::AssignVar,
        rhe: ex.clone(),
    };
    var_stack.push(sub);
}

fn process_var_stmt(r: &ReduceResult,
                    var_stack: &mut Vec<Statement>,
                    var_counter: &mut usize,
                    var_start: &mut usize,
                    block_stack: &mut Vec<Statement>)
{
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap();
    let meta = Meta::new(*var_start, end.clone());

    let mut inis: Vec<Statement> = vec![];
    for _ in 0..*var_counter {
        inis.push(var_stack.pop().unwrap());
    }
    *var_counter = 0;
    inis.reverse();

    // "InitializationBlock"
    let i_b = Statement::InitializationBlock {
        meta: meta.clone(),
        xtype: VariableType::Var,
        initializations: inis.clone(),
    };

    block_stack.push(i_b);
}

fn process_comp_def(r: &ReduceResult,
                    id_stack: &mut Vec<Token>,
                    expr_stack: &mut Vec<Expression>,
                    comp_stack: &mut Vec<Statement>,
                    comp_counter: &mut usize,)
{
    let i = id_stack.pop().unwrap();
    let var_name = i.value.clone();
    let start =  i.start.clone();
    let end = i.end.clone();
    let mut meta = Meta::new(start, end);
    let mut ex: Expression;

    // "Declaration"
    let dec = Statement::Declaration {
        meta: meta.clone(),
        xtype: VariableType::Component,
        name: var_name.clone(),
        dimensions: vec![],
        is_constant: true,
    };
    comp_stack.push(dec);
    *comp_counter += 1;

    if r.body.len() == 1 {
        // component a;
    }
    else {
        // component a = A();
        ex = expr_stack.pop().unwrap();
        // "Substitution"
        let sub = Substitution {
            meta: meta.clone(),
            var: var_name.clone(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: ex.clone(),
        };
        comp_stack.push(sub);
        *comp_counter += 1;
    }
}

fn process_comp_stmt(r: &ReduceResult,
                     comp_stack: &mut Vec<Statement>,
                     comp_counter: &mut usize,
                     comp_start: &mut usize,
                     block_stack: &mut Vec<Statement>)
{
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap();
    let meta = Meta::new(*comp_start, end.clone());

    let mut inis: Vec<Statement> = vec![];
    for _ in 0..*comp_counter {
        inis.push(comp_stack.pop().unwrap());
    }
    *comp_counter = 0;
    inis.reverse();

    // "InitializationBlock"
    let i_b = Statement::InitializationBlock {
        meta: meta.clone(),
        xtype: VariableType::Component,
        initializations: inis.clone(),
    };

    block_stack.push(i_b);
}

fn process_ret_stmt(r: &ReduceResult,
                    expr_stack: &mut Vec<Expression>,
                    block_stack: &mut Vec<Statement>)
{
    let start = r.token.iter().find(|t| t.token_type == "RETURN").map(|t| &t.start).unwrap().clone();
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap().clone();
    let meta = Meta::new(start, end.clone());
    let ex = expr_stack.pop().unwrap();
    let ret : Statement = Return { meta: meta.clone(), value: ex.clone() };
    block_stack.push(ret);
}

fn process_func_block(r: &ReduceResult,
                      block_stack: &mut Vec<Statement>,
                      stmt_counter: &mut usize,
                      param_stack: &mut Vec<Token>,
                      param_counter: &mut usize) -> Definition
{
    let start = r.token.iter().find(|t| t.token_type == "FUNCTION").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "RBRACE").map(|t| &t.end).unwrap();
    let arg = r.token.iter().find(|t| t.token_type == "LBRACE").map(|t| &t.end).unwrap();
    let meta = Meta::new(start.clone(), end.clone());
    let meta1 = Meta::new(arg.clone(), end.clone());
    let mut args: Vec<String> = vec![];
    let id = r.token.iter().find(|t| t.token_type == "ID").map(|t| &t.value).unwrap();

    if r.body.contains(&"PARAM".to_string()) {
        for _ in 0..*param_counter {
            let p = param_stack.pop().unwrap();
            args.push(p.value.clone());
        }
        *param_counter = 0;
    }

    let mut block_stmts = vec![];
    for _ in 0..*stmt_counter {
        block_stmts.push(block_stack.pop().unwrap());
    }
    *stmt_counter = 0;

    block_stmts.reverse();
    args.reverse();

    let mut block: Statement = Block {
        meta: meta1.clone(),
        stmts: block_stmts.clone(),
    };

    let mut temp: Definition = Function {
        meta: meta.clone(),
        name: id.to_string(),
        args: args.clone(),
        arg_location: *arg..*arg,
        body: block.clone(),
    };

    temp
}

fn process_template_block(r: &ReduceResult,
                          block_stack: &mut Vec<Statement>,
                          stmt_counter: &mut usize,
                          param_stack: &mut Vec<Token>,
                          param_counter: &mut usize) -> Definition
{
    let start = r.token.iter().find(|t| t.token_type == "TEMPLATE").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "RBRACE").map(|t| &t.end).unwrap();
    let arg = r.token.iter().find(|t| t.token_type == "LBRACE").map(|t| &t.end).unwrap();
    let meta = Meta::new(start.clone(), end.clone());
    let meta1 = Meta::new(arg.clone(), end.clone());
    let mut args: Vec<String> = vec![];

    let id = r.token.iter().find(|t| t.token_type == "ID").map(|t| &t.value).unwrap();

    if r.body.contains(&"PARAM".to_string()) {
        for _ in 0..*param_counter {
            let p = param_stack.pop().unwrap();
            args.push(p.value.clone());
        }
        *param_counter = 0;
    }

    let mut block_stmts = vec![];
    for _ in 0..*stmt_counter {
        block_stmts.push(block_stack.pop().unwrap());
    }
    *stmt_counter = 0;

    block_stmts.reverse();
    args.reverse();

    let mut block: Statement = Block {
        meta: meta1.clone(),
        stmts: block_stmts.clone(),
    };

    let mut temp: Definition = Template {
        meta: meta.clone(),
        name: id.to_string(),
        args: args.clone(),
        arg_location: *arg..*arg,
        body: block.clone(),
        parallel: false,
        is_custom_gate: false,
    };

    temp
}

fn process_component_block(r: &ReduceResult,
                           block_stack: &mut Vec<Statement>,
                           last: &mut usize,
                           param_stack: &mut Vec<Token>,
                           param_counter: &mut usize) -> MainComponent
{
    let start = r.token.iter().find(|t| t.token_type == "COMPONENT").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap();
    let meta = Meta::new(start.clone(), end.clone());
    *last = *end;
    let id = r.token.iter().find(|t| t.token_type == "ID").map(|t| &t.value).unwrap();
    let mut args: Vec<Expression> = vec![];

    if r.body.contains(&"PARAM".to_string()) {
        for _ in 0..*param_counter {
            let p = param_stack.pop().unwrap();
            match p.token_type.as_str() {
                "ID" => {
                    let ex: Expression = Variable {
                        meta: Meta::new(p.start.clone(), p.end.clone()),
                        name: p.value.clone(),
                        access: vec![],
                    };
                    args.push(ex);
                },
                "NUM" => {
                    let num = Num::new(Sign::NoSign, vec![p.value.clone().parse::<i128>().unwrap()]);
                    let ex: Expression = Number(Meta::new(p.start.clone(), p.end.clone()), num.clone());
                    args.push(ex);
                },
                _ => {}
            }
        }
        *param_counter = 0;
    }

    args.reverse();

    let mut ex: Expression = Call {
        meta: meta.clone(),
        id: id.clone(),
        args: args.clone(),
    };

    let mut mc:MainComponent = (vec![], ex.clone());

    mc
}

fn process_expr(r: &ReduceResult,
                id_stack: &mut Vec<Token>,
                expr_stack: &mut Vec<Expression>,
                op_stack: &mut Vec<Token>,
                param_stack: &mut Vec<Token>,
                param_counters: &mut Vec<usize>)
{
    if r.body.len() == 1 {
        // EXPR -> ID_OR_NUM
        let i = id_stack.pop().unwrap();
        match i.token_type.as_str() {
            "ID" => {
                let ex: Expression = Variable {
                    meta: Meta::new(i.start.clone(), i.end.clone()),
                    name: i.value.clone(),
                    access: vec![],
                };
                expr_stack.push(ex);
            },
            "NUM" => {
                let num = Num::new(Sign::NoSign, vec![i.value.clone().parse::<i128>().unwrap()]);
                let ex: Expression = Number(Meta::new(i.start.clone(), i.end.clone()), num.clone());
                expr_stack.push(ex);
            },
            _ => {}
        }
    }
    else if r.body.contains(&"OP".to_string()) {
        // EXPR -> EXPR OP ID_OR_NUM
        let i = id_stack.pop().unwrap();
        let mut lhe = expr_stack.pop().unwrap();
        let mut rhe: Expression;
        match i.token_type.as_str() {
            "ID" => {
                rhe = Variable {
                    meta: Meta::new(i.start.clone(), i.end.clone()),
                    name: i.value.clone(),
                    access: vec![],
                }
            }
            "NUM" => {
                let num = Num::new(Sign::NoSign, vec![i.value.clone().parse::<i128>().unwrap()]);
                rhe = Number(Meta::new(i.start.clone(), i.end.clone()), num.clone());
            }
            _ => {
                let num = Num::new(Sign::Minus, vec![1]);
                rhe = Number(Meta::new(i.start.clone(), i.end.clone()), num.clone());
            }
        }

        let op = op_stack.pop().unwrap();
        let mut meta = Meta::new(op.start.clone(), op.end.clone());
        let mut eio: ExpressionInfixOpcode;
        let is_infix: bool = true;
        match op.value.as_str() {
            "*" => eio = ExpressionInfixOpcode::Mul,
            "+" => eio = ExpressionInfixOpcode::Add,
            "-" => eio = ExpressionInfixOpcode::Sub,
            "/" => eio = ExpressionInfixOpcode::Div,
            "%" => eio = ExpressionInfixOpcode::Mod,
            "QUOTIENT" => eio = ExpressionInfixOpcode::IntDiv,
            "\\" => eio = ExpressionInfixOpcode::IntDiv,
            "&" => eio = ExpressionInfixOpcode::BoolAnd,
            "|" => eio = ExpressionInfixOpcode::BoolOr,
            "BITWISE_OR" => eio = ExpressionInfixOpcode::BoolOr,
            "^" => eio = ExpressionInfixOpcode::BitXor,
            "<<" => eio = ExpressionInfixOpcode::ShiftL,
            ">>" => eio = ExpressionInfixOpcode::ShiftR,
            _ => eio = ExpressionInfixOpcode::Add,
        }

        if is_infix {
            let mut infixop : Expression = InfixOp {
                meta: meta.clone(),
                lhe: Box::new(lhe.clone()),
                infix_op: eio.clone(),
                rhe: Box::new(rhe.clone()),
            };
            expr_stack.push(infixop);
        }
    }
    else if r.body.contains(&"PLUS".to_string()) {
        // EXPR -> PLUS NUM
    }
    else if r.body.contains(&"ID_OR_ARRAY".to_string()) {
        let i = id_stack.pop().unwrap();
        let mut args = vec![];
        let end = r.token.iter().find(|t| t.token_type == "RPAREN").map(|t| &t.end).unwrap();
        if r.body.contains(&"PARAM".to_string()) {
            // EXPR -> ID ( PARAM )
            let counter = param_counters.pop().unwrap();
            for _ in 0..counter {
                let p = param_stack.pop().unwrap();
                match p.token_type.as_str() {
                    "ID" => {
                        let ex: Expression = Variable {
                            meta: Meta::new(p.start.clone(), p.end.clone()),
                            name: p.value.clone(),
                            access: vec![],
                        };
                        args.push(ex);
                    },
                    "NUM" => {
                        let num = Num::new(Sign::NoSign, vec![p.value.clone().parse::<i128>().unwrap()]);
                        let ex: Expression = Number(Meta::new(p.start.clone(), p.end.clone()), num.clone());
                        args.push(ex);
                    },
                    _ => {}
                }
            }
        }
        args.reverse();
        let ca: Expression = Call {
            meta: Meta::new(i.start.clone(), end.clone()),
            id: i.value.clone(),
            args: args.clone(),
        };
        expr_stack.push(ca);
    }
    else {
        // EXPR -> ( EXPR )
    }

}

fn process_c_assign_stmt(r: &ReduceResult,
                         id_stack: &mut Vec<Token>,
                         expr_stack: &mut Vec<Expression>,
                         block_stack: &mut Vec<Statement>,
                         op_stack: &mut Vec<Token>)
{
    let mut end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap().clone();
    let i = id_stack.pop().unwrap();
    let mut start = i.start.clone();
    let mut meta = Meta::new(start, end);

    let ex = expr_stack.pop().unwrap();

    let op = op_stack.pop().unwrap();
    let mut aop: AssignOp;
    match op.value.as_str() {
        "===" => aop = AssignOp::AssignConstraintSignal,
        "<==" => aop = AssignOp::AssignConstraintSignal,
        "==>" => aop = AssignOp::AssignConstraintSignal,
        "<--" => aop = AssignOp::AssignSignal,
        "-->" => aop = AssignOp::AssignSignal,
        _ => aop = AssignOp::AssignSignal,
    }

    let substitution: Statement = Substitution {
        meta: meta.clone(),
        var: i.value.clone(),
        access: vec![],
        op: aop.clone(),
        rhe: ex.clone(),
    };

    block_stack.push(substitution);
}

fn process_cond(r: &ReduceResult,
                expr_stack: &mut Vec<Expression>,
                rel_stack: &mut Vec<Token>,
                cond_stack: &mut Vec<Expression>)
{
    let re = expr_stack.pop().unwrap();
    let le = expr_stack.pop().unwrap();
    let rel = rel_stack.pop().unwrap();

    let mut meta = Meta::new(rel.start.clone(), rel.end.clone());

    let mut eio: ExpressionInfixOpcode;
    match rel.value.as_str() {
        "==" => eio = ExpressionInfixOpcode::Eq,
        "!=" => eio = ExpressionInfixOpcode::NotEq,
        "<" => eio = ExpressionInfixOpcode::Lesser,
        "<=" => eio = ExpressionInfixOpcode::LesserEq,
        ">" => eio = ExpressionInfixOpcode::Greater,
        ">=" => eio = ExpressionInfixOpcode::GreaterEq,
        _ => eio =ExpressionInfixOpcode::Eq,
    }

    let mut cond : Expression = InfixOp {
        meta: meta.clone(),
        lhe: Box::new(le.clone()),
        infix_op: eio.clone(),
        rhe: Box::new(re.clone()),
    };

    cond_stack.push(cond);
}

fn process_if_stmt(r: &ReduceResult,
                   cond_stack: &mut Vec<Expression>,
                   stmt_counters: &mut Vec<usize>,
                   block_stack: &mut Vec<Statement>)
{
    let start = r.token.iter().find(|t| t.token_type == "IF").map(|t| &t.start).unwrap();
    let end = r.token.iter()
        .filter(|t| t.token_type == "RBRACE")
        .fold(None, |max, t| Some(max.map_or(t.end, |current_max| if t.end > current_max { t.end } else { current_max })))
        .unwrap_or(0);

    let meta: Meta = Meta::new(start.clone(), end.clone());
    let cond = cond_stack.pop().unwrap();

    if r.body.iter().any(|e| e == "ELSE") {
        let counter2 = stmt_counters.pop().unwrap();  // else
        let mut block_stmts2 = vec![];
        let counter1 = stmt_counters.pop().unwrap();  // if
        let mut block_stmts1 = vec![];

        for _ in 0..counter2 {
            block_stmts2.push(block_stack.pop().unwrap());
        }
        block_stmts2.reverse();

        for _ in 0..counter1 {
            block_stmts1.push(block_stack.pop().unwrap());
        }
        block_stmts1.reverse();

        let lbrace_tokens: Vec<_> = r.token.iter().filter(|t| t.token_type == "LBRACE").collect();
        let rbrace_tokens: Vec<_> = r.token.iter().filter(|t| t.token_type == "RBRACE").collect();

        let lbrace_start = lbrace_tokens.iter().map(|t| t.start).collect::<Vec<usize>>();
        let mut lbrace_start_sorted = lbrace_start.clone();
        lbrace_start_sorted.sort();
        let lbrace_start1 = lbrace_start_sorted[0];
        let lbrace_start2 = lbrace_start_sorted[1];

        let rbrace_end = rbrace_tokens.iter().map(|t| t.end).collect::<Vec<usize>>();
        let mut rbrace_end_sorted = rbrace_end.clone();
        rbrace_end_sorted.sort();
        let rbrace_end1 = rbrace_end_sorted[0];
        let rbrace_end2 = rbrace_end_sorted[1];

        let meta1: Meta = Meta::new(lbrace_start1.clone(), rbrace_end1.clone());
        let meta2: Meta = Meta::new(lbrace_start2.clone(), rbrace_end2.clone());

        let mut if_block: Statement = Block {
            meta: meta1.clone(),
            stmts: block_stmts1.clone(),
        };
        let mut else_block: Statement = Block {
            meta: meta2.clone(),
            stmts: block_stmts2.clone(),
        };
        let st: Statement = IfThenElse {
            meta: meta.clone(),
            cond: cond.clone(),
            if_case: Box::new(if_block.clone()),
            else_case: Some(Box::new(else_block.clone())),
        };
        block_stack.push(st.clone());
    }
    else {
        let counter = stmt_counters.pop().unwrap();
        let mut block_stmts = vec![];
        for _ in 0..counter {
            block_stmts.push(block_stack.pop().unwrap());
        }
        block_stmts.reverse();

        let if_start = r.token.iter().find(|t| t.token_type == "LBRACE").map(|t| &t.start).unwrap();
        let if_meta = Meta::new(if_start.clone(), end.clone());
        let mut if_block: Statement = Block {
            meta: if_meta.clone(),
            stmts: block_stmts.clone(),
        };
        let st: Statement = IfThenElse {
            meta: meta.clone(),
            cond: cond.clone(),
            if_case: Box::new(if_block.clone()),
            else_case: None,
        };
        block_stack.push(st.clone());
    }

}

fn process_while_stmt(r: &ReduceResult,
                      cond_stack: &mut Vec<Expression>,
                      stmt_counters: &mut Vec<usize>,
                      block_stack: &mut Vec<Statement>)
{
    let start = r.token.iter().find(|t| t.token_type == "WHILE").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "RBRACE").map(|t| &t.end).unwrap();
    let meta: Meta = Meta::new(start.clone(), end.clone());
    let cond = cond_stack.pop().unwrap();

    let mut counter = stmt_counters.pop().unwrap();
    let mut block_stmts = vec![];
    for _ in 0..counter {
        block_stmts.push(block_stack.pop().unwrap());
    }
    block_stmts.reverse();

    let while_block: Statement = Block{
        meta: meta.clone(),
        stmts: block_stmts.clone(),
    };

    let while_stmt: Statement = While {
        meta: meta.clone(),
        cond: cond.clone(),
        stmt: Box::new(while_block.clone()),
    };

    block_stack.push(while_stmt);
}

fn process_for_stmt(r: &ReduceResult,
                    for_cond_stack: &mut Vec<ForCond>,
                    stmt_counters: &mut Vec<usize>,
                    block_stack: &mut Vec<Statement>)
{
    let start = r.token.iter().find(|t| t.token_type == "FOR").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "RBRACE").map(|t| &t.end).unwrap();
    let meta: Meta = Meta::new(start.clone(), end.clone());

    let fc = for_cond_stack.pop().unwrap();
    let cond = fc.p2.clone();
    let p3 = fc.p3.clone();
    let p1 = fc.p1.clone();

    let mut counter = stmt_counters.pop().unwrap();
    let mut block_stmts = vec![];
    for _ in 0..counter {
        block_stmts.push(block_stack.pop().unwrap());
    }
    block_stmts.reverse();
    block_stmts.push(p3.clone());

    // p1转化为initialization block, counter+=1
    block_stack.push(p1.clone());

    let while_block: Statement = Block{
        meta: meta.clone(),
        stmts: block_stmts.clone(),
    };

    let while_stmt: Statement = While {
        meta: meta.clone(),
        cond: cond.clone(),
        stmt: Box::new(while_block.clone()),
    };

    block_stack.push(while_stmt);
}

#[derive(Clone, Debug, Serialize)]
pub struct ForCond {
    pub p1: Statement, // var i = 0;   InitializationBlock
    pub p2: Expression, // i < 10;
    pub p3: Statement, // i += 1;   Substitution
}
impl ForCond {
    pub fn new(p1: Statement, p2: Expression, p3: Statement) -> ForCond {
        ForCond { p1, p2, p3 }
    }
}

fn process_for_cond(r: &ReduceResult,
                    cond_stack: &mut Vec<Expression>,
                    for_cond_stack: &mut Vec<ForCond>,
                    block_stack: &mut Vec<Statement>,
                    id_stack: &mut Vec<Token>,
                    assign_stack: &mut Vec<Token>,
                    expr_stack: &mut Vec<Expression>)
{
    let p2 = cond_stack.pop().unwrap();
    let p1 = block_stack.pop().unwrap();
    let mut p3: Statement;

    // ID_OR_ARRAY ASSIGN EXPR
    let i = id_stack.pop().unwrap();
    let var_name = i.value.clone();
    let ex = expr_stack.pop().unwrap();
    let assign = assign_stack.pop().unwrap();
    let start = i.start.clone();
    let mut end = start.clone();
    if let Expression::InfixOp { meta, .. } = ex.clone() {
        end = meta.end;
    }
    let mut meta = Meta::new(start.clone(), end.clone());  // incorrect
    let mut rhe: Expression = ex.clone();
    let mut infixop: ExpressionInfixOpcode = ExpressionInfixOpcode::Add;
    if assign.value == "=" {
        let substitution: Statement = Substitution {
            meta: meta.clone(),
            var: var_name.clone(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: rhe.clone(),
        };
        p3 = substitution.clone();
    }
    else {
        match assign.value.as_str() {
            "+=" => infixop = ExpressionInfixOpcode::Add,
            "-=" => infixop = ExpressionInfixOpcode::Sub,
            "*=" => infixop = ExpressionInfixOpcode::Mul,
            "/=" => infixop = ExpressionInfixOpcode::Div,
            "%=" => infixop = ExpressionInfixOpcode::Mod,
            "QUOTIENT_ASSIGN" => infixop = ExpressionInfixOpcode::IntDiv,
            "&=" => infixop = ExpressionInfixOpcode::BitAnd,
            "BITWISE_OR_ASSIGN" => infixop = ExpressionInfixOpcode::BitOr,
            "^=" => infixop = ExpressionInfixOpcode::BitXor,
            "<<=" => infixop = ExpressionInfixOpcode::ShiftL,
            ">>=" => infixop = ExpressionInfixOpcode::ShiftR,
            _ => {}
        }

        let lhe = Variable {
            meta: meta.clone(),
            name: var_name.clone(),
            access: vec![],
        };

        let rhe_1: Expression = InfixOp {
            meta: meta.clone(),
            lhe: Box::new(lhe.clone()),
            infix_op: infixop.clone(),
            rhe: Box::new(rhe.clone()),
        };

        let substitution: Statement = Substitution {
            meta: meta.clone(),
            var: var_name.clone(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: rhe_1.clone(),
        };

        p3 = substitution.clone();
    }

    let fc = ForCond::new(p1, p2, p3);
    for_cond_stack.push(fc);
}

pub fn build_ast(results: Vec<ReduceResult>) -> AST {

    let mut compiler_version: Version = (0, 0, 0);
    let mut definitions: Vec<Definition> = vec![];
    let mut main_component: std::option::Option<MainComponent> = None;

    let mut last: usize = 256;

    let mut id_stack: Vec<Token> = vec![];  // Identifier and Num
    let mut op_stack: Vec<Token> = vec![];  // Operator
    let mut expr_stack: Vec<Expression> = vec![];  // Expression
    let mut block_stack: Vec<Statement> = vec![];  // Block
    let mut param_stack: Vec<Token> = vec![];  // Params for template and component
    let mut rel_stack: Vec<Token> = vec![];  // relation operator eg. == >=
    let mut cond_stack: Vec<Expression> = vec![]; // condition eg. 1==1
    let mut assign_stack: Vec<Token> = vec![];  // assign operator eg. = +=

    let mut var_stack: Vec<Statement> = vec![];  // var a=1;
    let mut var_counter: usize = 0;
    let mut var_start: usize = 0;

    let mut comp_stack: Vec<Statement> = vec![];  // component a=A(1);
    let mut comp_counter: usize = 0;
    let mut comp_start: usize = 0;

    let mut for_cond_stack: Vec<ForCond> = vec![];  // for (p1;p2;p3)

    let mut stmt_counters : Vec<usize> = vec![];
    let mut param_counters: Vec<usize> = vec![];
    let mut counter_flag: bool = false;  // 判断counter是否需要额外+1

    for r in results {
        match r.head.clone().as_str() {
            "HEADER" => {
                if r.body.iter().any(|e| e == "VERSION") {
                    let v_str = r.token.iter().find(|t| t.token_type == "VERSION")
                        .map(|t| &t.value).unwrap();
                    compiler_version = process_version(v_str);
                }
            },
            "ID_OR_ARRAY" => {
                let mut t = r.token.iter().find(|t| t.token_type == "ID").unwrap();
                id_stack.push(t.clone());
            },
            "SIGNAL_STMT" => {
                process_signal_stmt(&r, &mut id_stack, &mut block_stack);
            },
            "C_ASSIGN" => {
                let mut t = r.token[0].clone();
                op_stack.push(t.clone());
            },
            "OP" => {
                if !r.body.iter().any(|e| e == "PLUS") {
                    let mut t = r.token[0].clone();
                    op_stack.push(t.clone());
                }
            },
            "PLUS" => {
                let mut t = r.token[0].clone();
                op_stack.push(t.clone());
            },
            "ASSIGN" => {
                let mut t = r.token[0].clone();
                assign_stack.push(t.clone());
            },
            "ASSIGN_STMT" => {
                process_assign_stmt(&r, &mut id_stack, &mut assign_stack, &mut expr_stack, &mut block_stack);
            },
            "EXPR" => {
                process_expr(&r, &mut id_stack, &mut expr_stack, &mut op_stack, &mut param_stack, &mut param_counters);
            },
            "C_ASSIGN_STMT" => {
                process_c_assign_stmt(&r, &mut id_stack, &mut expr_stack, &mut block_stack, &mut op_stack);
            },
            "VAR_STMT" => {
                process_var_stmt(&r, &mut var_stack, &mut var_counter, &mut var_start, &mut block_stack);
            },
            "VAR_" => {
                if r.body.len() == 2 {
                    // VAR_ -> VAR VAR_DEF
                    var_start = r.token.iter().find(|t| t.token_type == "VAR").unwrap().start;
                }
                else {
                    // VAR_ -> VAR_ , VAR_DEF
                }
            },
            "VAR_DEF" => {
                process_var_def(&r, &mut id_stack, &mut expr_stack, &mut var_stack);
                var_counter += 2;
            },
            "COMPONENT_STMT" => {
                process_comp_stmt(&r, &mut comp_stack, &mut comp_counter, &mut comp_start, &mut block_stack);
            },
            "COMP_" => {
                if r.body.len() == 2 {
                    // COMP_ -> COMPONENT COMP_DEF
                    comp_start = r.token.iter().find(|t| t.token_type == "COMPONENT").unwrap().start;
                }
                else {}
            },
            "COMP_DEF" => {
                process_comp_def(&r, &mut id_stack, &mut expr_stack, &mut comp_stack, &mut comp_counter);
            },
            "STMTS" => {
                if r.body.len() == 1 {
                    // STMTS -> STMT 表示一个新的块
                    stmt_counters.push(1);
                }
                else {
                    // STMTS -> STMT STMTS 表示新增
                    let index = stmt_counters.len() - 1;
                    stmt_counters[index] += 1;
                }
                if counter_flag {
                    let index = stmt_counters.len() - 1;
                    stmt_counters[index] += 1;
                    counter_flag = false;
                }
            },
            "TEMPLATE_CONTENT" => {},
            "TEMPLATE_STMT" => {
                let index = stmt_counters.len() - 1;
                let mut param_counter = 0;
                if r.body.iter().any(|e| e == "PARAM") {
                    let index1 = param_counters.len() - 1;
                    param_counter = param_counters[index1];
                }
                definitions.push(process_template_block(&r, &mut block_stack, &mut stmt_counters[index], &mut param_stack, &mut param_counter));
                stmt_counters.pop();
                if r.body.iter().any(|e| e == "PARAM") {
                    param_counters.pop();
                }
            },
            "FUNC_STMT" => {
                let index = stmt_counters.len() - 1;
                let mut param_counter = 0;
                if r.body.iter().any(|e| e == "PARAM") {
                    let index1 = param_counters.len() - 1;
                    param_counter = param_counters[index1];
                }
                definitions.push(process_func_block(&r, &mut block_stack, &mut stmt_counters[index], &mut param_stack, &mut param_counter));
                stmt_counters.pop();
                if r.body.iter().any(|e| e == "PARAM") {
                    param_counters.pop();
                }

            },
            "RET_STMT" => {
                process_ret_stmt(&r, &mut expr_stack, &mut block_stack);
            },
            "COMPONENT_BLOCK" => {
                let mut param_counter = 0;
                if r.body.iter().any(|e| e == "PARAM") {
                    let index = param_counters.len() - 1;
                    param_counter = param_counters[index];
                }
                main_component = Some(process_component_block(&r, &mut block_stack, &mut last, &mut param_stack, &mut param_counter));
                param_counters.pop();
            },
            "PROGRAM" => break,
            "ID_OR_NUM" => {
                // 这里只处理 -> NUM 的情况，其他情况在 ID_OR_ARRAY 处理
              if r.body.iter().any(|e| e == "NUM") {
                  let mut t = r.token.iter().find(|t| t.token_type == "NUM").unwrap();
                  id_stack.push(t.clone());
              }
            },
            "PARAM" => {
                let mut t = id_stack.pop().unwrap();
                param_stack.push(t.clone());
                if r.body.len() == 1 {
                    param_counters.push(1);
                }
                else {
                    let index = param_counters.len() - 1;
                    param_counters[index] += 1;
                }
            },
            "IF_STMT" => {
                process_if_stmt(&r, &mut cond_stack, &mut stmt_counters, &mut block_stack);
            },
            "CONDITION" => {
                process_cond(&r, &mut expr_stack, &mut rel_stack, &mut cond_stack);
            },
            "REL" => {
                let mut t = r.token[0].clone();
                rel_stack.push(t.clone());
            },
            "WHILE_STMT" => {
                process_while_stmt(&r, &mut cond_stack, &mut stmt_counters, &mut block_stack);
            },
            "FOR_STMT" => {
                process_for_stmt(&r, &mut for_cond_stack, &mut stmt_counters, &mut block_stack);
                counter_flag = true;
            },
            "FOR_COND" => {
                process_for_cond(&r, &mut cond_stack, &mut for_cond_stack, &mut block_stack, &mut id_stack, &mut assign_stack, &mut expr_stack);
            },
            _ => {},
        }
    }

    let mut meta = Meta::new(0, last);
    let mut pragmas: Vec<Pragma> = vec![];
    pragmas.push(Pragma::Version(meta.clone(), 0, compiler_version));

    let mut ast: AST = AST::new(
        meta.clone(),
        pragmas.clone(),
        vec![],
        definitions.clone(),
        main_component.clone(),
    );

    ast
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_version() {
        assert_eq!(process_version("2.0.0"), (2, 0, 0));
    }

}
