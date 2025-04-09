use std::{io, mem};
use lib::input::input::*;
use lib::lexer::lexer::Lexer;
use lib::parser::grammar::Grammar;
use lib::parser::lr1::{LR1Parser, ParseError, ReduceResult};
use std::fs::File;
use std::io::Write;
use log::info;
use lib::ast::ast_build::build_ast;

fn main() -> io::Result<()>{

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // let file_path = "testcase/array.circom";
    let file_path = "testcase/1.circom";
    let grammar_path = "grammar/grammar.txt";
    let out_path = "out/1.json";

    let content = read_circom_file(file_path)?;

    info!("{}",format!("Processed content:\n{}\n", content).as_str());

    let mut lexer = Lexer::new(content.to_string());

    lexer.tokenize();

    let tokens = lexer.tokens;

    for token in tokens.clone() {
        info!("{}",format!("{:?}", token).as_str());
    }


    let grammar_str = read_circom_file(grammar_path)?;
    let grammar = Grammar::new(&*grammar_str).unwrap();

    info!("Grammar:\n{}", grammar);

    let parser = LR1Parser::new(grammar);

    let reduce_result = parser.unwrap().run_parse(&tokens);

    match reduce_result {
        Ok(steps) => {
            let s = steps.clone();
            info!("\nParse succeeded! Steps:");
            for step in steps {
                info!("{:?}", step);
            }

            let ast = build_ast(s);
            let json_data = serde_json::to_string_pretty(&ast)?;
            let mut file = File::create(out_path)?;
            file.write_all(json_data.as_bytes())?;

            info!("\nOutput json to: {}\n", out_path);

        }
        _ => {}
    }

    Ok(())
}
