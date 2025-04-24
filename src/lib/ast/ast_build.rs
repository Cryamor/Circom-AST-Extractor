use std::ops::Range;
use crate::ast::ast::*;
use crate::ast::ast::Definition::Template;
use crate::ast::ast::Expression::{Call, InfixOp, Number, Variable};
use crate::ast::ast::Statement::{Block, Substitution};
use crate::lexer::token::Token;
use crate::parser::lr1::ReduceResult;

fn process_version(version: &str) -> Version {
    let parts: Vec<&str> = version.split('.').collect();
    let p1 = parts[0].parse::<usize>().unwrap();
    let p2 = parts[1].parse::<usize>().unwrap();
    let p3 = parts[2].parse::<usize>().unwrap();

    (p1, p2, p3)
}

fn process_signal_stmt(r: &ReduceResult, id_stack: &mut Vec<Token>, block_stack: &mut Vec<Statement>) {
    let start = r.token.iter().find(|t| t.token_type == "SIGNAL").map(|t| &t.start).unwrap();
    let end = r.token.iter().find(|t| t.token_type == "SEMICOLON").map(|t| &t.end).unwrap();
    let meta = Meta::new(start.clone(), end.clone());

    let mut xtype: VariableType;
    if r.body.contains(&"INPUT".to_string()) {
        xtype = VariableType::Signal(SignalType::Input, vec![]);
    }
    else {
        xtype = VariableType::Signal(SignalType::Output, vec![]);
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

fn process_template_block(r: &ReduceResult, block_stack: &mut Vec<Statement>, stmt_counter: &mut usize, param_stack: &mut Vec<Token>, param_counter: &mut usize) -> Definition {
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

fn process_component_block(r: &ReduceResult, block_stack: &mut Vec<Statement>, last: &mut usize, param_stack: &mut Vec<Token>, param_counter: &mut usize) -> MainComponent {
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
                    let ex: Expression = Number(Meta::new(p.start.clone(), p.end.clone()), p.value.clone().parse::<i128>().unwrap());
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

fn process_expr(r: &ReduceResult, id_stack: &mut Vec<Token>, expr_stack: &mut Vec<Expression>, op_stack: &mut Vec<Token>) {
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
                let ex: Expression = Number(Meta::new(i.start.clone(), i.end.clone()), i.value.clone().parse::<i128>().unwrap());
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
                rhe = Number(Meta::new(i.start.clone(), i.end.clone()), i.value.clone().parse::<i128>().unwrap());
            }
            _ => {
                rhe = Number(Meta::new(i.start.clone(), i.end.clone()), -1);
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
    else {
        // EXPR -> ( EXPR )
    }

}

fn process_c_assign_stmt(r: &ReduceResult, id_stack: &mut Vec<Token>, expr_stack: &mut Vec<Expression>, block_stack: &mut Vec<Statement>, op_stack: &mut Vec<Token>) {
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

fn process_cond(r: &ReduceResult, expr_stack: &mut Vec<Expression>, rel_stack: &mut Vec<Token>) {

}

fn process_if_stmt(r: &ReduceResult, cond_stack: &mut Vec<Expression>) {

}



pub fn build_ast(results: Vec<ReduceResult>) -> AST {

    let mut compiler_version: Version = (0, 0, 0);
    let mut definitions: Vec<Definition> = vec![];
    let mut main_component: Option<MainComponent> = Option::None;

    let mut last: usize = 256;

    let mut id_stack: Vec<Token> = vec![];  // Identifier and Num
    let mut op_stack: Vec<Token> = vec![];  // Operator
    let mut expr_stack: Vec<Expression> = vec![];  // Expression
    let mut block_stack: Vec<Statement> = vec![];  // Block
    let mut param_stack: Vec<Token> = vec![];  // Params for template and component
    let mut rel_stack: Vec<Token> = vec![];  // relation operator eg. == >=
    let mut cond_stack: Vec<Expression> = vec![]; // condition eg. 1==1
    let mut assign_stack: Vec<Token> = vec![];  // assign operator eg. = +=

    let mut stmt_counter: usize = 0;
    let mut param_counter: usize = 0;

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

            },
            "EXPR" => {
                process_expr(&r, &mut id_stack, &mut expr_stack, &mut op_stack);
            },
            "C_ASSIGN_STMT" => {
                process_c_assign_stmt(&r, &mut id_stack, &mut expr_stack, &mut block_stack, &mut op_stack);
            },
            "STMTS" => {
                if r.body.len() == 1 {
                    // STMTS -> STMT 表示一个新的块


                }
                else {

                }
              stmt_counter = stmt_counter + 1;
            },
            "TEMPLATE_CONTENT" => {},
            "TEMPLATE_STMT" => {
                definitions.push(process_template_block(&r, &mut block_stack, &mut stmt_counter, &mut param_stack, &mut param_counter));
            },
            "COMPONENT_BLOCK" => {
                main_component = Some(process_component_block(&r, &mut block_stack, &mut last, &mut param_stack, &mut param_counter));
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
                param_counter = param_counter + 1;
            },
            "IF_STMT" => {

            },
            "CONDITION" => {

            },
            "REL" => {
                let mut t = r.token[0].clone();
                rel_stack.push(t.clone());
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
