use std::{io, mem};
use lib::input::input::*;
use lib::lexer::lexer::Lexer;
use lib::parser::grammar::Grammar;
use lib::parser::lr1::{LR1Parser};
use std::fs::File;
use log::info;

fn main() -> io::Result<()>{

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // let file_path = "testcase/array.circom";
    let file_path = "testcase/array.circom";

    let content = read_circom_file(file_path)?;

    info!("{}",format!("Processed content:\n{}\n", content).as_str());

    let mut lexer = Lexer::new(content.to_string());

    lexer.tokenize();

    let tokens = lexer.tokens;

    for token in tokens {
        info!("{}",format!("{:?}", token).as_str());
    }

    let grammar_path = "grammar/grammar.txt";

    let grammar_str = read_circom_file(grammar_path)?;

    let grammar = Grammar::new(&*grammar_str).unwrap();

    println!("Grammar:\n{}", grammar);

    let parser = LR1Parser::new(grammar);

    // match parser.parse1(&tokens) {
    //     Ok(steps) => {
    //         println!("Parse succeeded! Steps:");
    //         for step in steps {
    //             println!("{:?}", step);
    //         }
    //     }
    // }

    Ok(())
}
