use std::{env, io, mem};
use lib::input::input::*;
use lib::lexer::lexer::Lexer;
use lib::parser::grammar::Grammar;
use lib::parser::lr1::{LR1Parser, ParseError, ReduceResult};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use log::info;
use lib::ast::ast_build::build_ast;

fn main() -> io::Result<()>{

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let start_time = Instant::now();

    // let file_path = "testcase/array.circom";
    // let file_path = "testcase/1.circom";
    let grammar_path = "grammar/grammar.txt";
    // let out_path = "out/1.json";
    let parser_cache_path = "cache/parser_cache.json";

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let file_path: &str = if args.len() > 1 {
        &*args[1].clone()
    }
    else {
        "testcase/1.circom"
        // "testcase/2.circom"
        // "testcase/3.circom"
        // "testcase/4.circom"
        // "testcase/5.circom"
        // "testcase/6.circom"
        // "testcase/11.circom"
        // "testcase/1-1.circom"
        // "testcase/1-error-lexer.circom"
        // "testcase/1-error-parser.circom"
    };

    let out_path: &str = if args.len() > 2 {
        &*args[2].clone()
    } else {
        let file_name = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
        &*format!("out/{}.json", file_name)
    };

    let content = read_circom_file(file_path)?;

    info!("{}",format!("Processed Content:\n{}\n", content).as_str());
    println!("Process Content...");

    let mut lexer = Lexer::new(content.to_string());

    lexer.tokenize();

    let tokens = lexer.tokens;

    println!("Receive Tokens...");
    info!("Receive Tokens:\n");
    for token in tokens.clone() {
        info!("{}",format!("{:?}", token).as_str());
    }

    let token_duration = start_time.elapsed();
    println!("Time Cost: {:?}", token_duration);

    // 可以将编译好的parser存储起来，之后直接读取使用
    // 尝试从文件加载 LR1Parser
    let parser: LR1Parser = if Path::new(parser_cache_path).exists() {
        println!("Loaded parser cache...");
        info!("Loaded parser cache...");
        LR1Parser::load_from_file(parser_cache_path)?
    } else {
        // 构建 LR1Parser
        let grammar_str = read_circom_file(grammar_path)?;
        let grammar = Grammar::new(&*grammar_str).unwrap();

        info!("\nGrammar:\n{}", grammar);
        println!("Construct LR1 Parser...");
        info!("Construct LR1 Parser...\n");

        let parser = LR1Parser::new(grammar);

        // 保存 LR1Parser 到文件
        parser.clone().unwrap().save_to_file(parser_cache_path)?;
        println!("Saved parser cache... to {:?}", parser_cache_path);
        info!("Saved parser cache... to {:?}", parser_cache_path);

        parser.unwrap()
    };

    let construct_duration = start_time.elapsed();
    println!("Time Cost: {:?}", construct_duration);

    println!("Start Parsing...");
    info!("Start Parsing...\n");
    let reduce_result = parser.run_parse(&tokens);

    match reduce_result {
        Ok(steps) => {
            let s = steps.clone();
            info!("\nParse succeeded! Steps:");
            println!("Parse succeeded!");
            for step in steps {
                info!("{:?}", step);
            }

            let ast = build_ast(s);
            let json_data = serde_json::to_string_pretty(&ast)?;
            let mut file = File::create(out_path)?;
            file.write_all(json_data.as_bytes())?;

            info!("\nOutput json to: {}\n", out_path);
            println!("Output json to: {}", out_path);
        }
        _ => {}
    }

    let duration = start_time.elapsed();
    info!("Time Cost: {:?}", duration);
    println!("Time Cost: {:?}", duration);
    println!("Parsing Time Cost: {:?}", duration - construct_duration);

    Ok(())
}
