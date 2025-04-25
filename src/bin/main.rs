use std::{env, io, mem};
use lib::input::input::*;
use lib::lexer::lexer::Lexer;
use lib::parser::grammar::Grammar;
use lib::parser::lr1::{LR1Parser, ParseError, ReduceResult};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use log::info;
use lib::ast::ast_build::build_ast;

fn main() -> io::Result<()>{

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // let file_path = "testcase/array.circom";
    // let file_path = "testcase/1.circom";
    let grammar_path = "grammar/grammar.txt";
    // let out_path = "out/1.json";

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let file_path: &str = if args.len() > 1 {
        &*args[1].clone()
    } else {
        // "testcase/1.circom"
        // "testcase/2.circom"
        // "testcase/3.circom"
        // "testcase/4.circom"
        "testcase/5.circom"
    };

    let out_path: &str = if args.len() > 2 {
        &*args[2].clone()
    } else {
        let file_name = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
        &*format!("out/{}.json", file_name)
    };


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
