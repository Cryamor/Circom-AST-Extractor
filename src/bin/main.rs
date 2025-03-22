use std::io;
use lib::input::input::*;
use lib::lexer::lexer::Lexer;
use lib::parser::grammar::Grammar;

fn main() -> io::Result<()>{

    // let file_path = "testcase/array.circom";
    let file_path = "testcase/array.circom";

    let content = read_circom_file(file_path)?;

    println!("Processed content:\n{}\n", content);

    let mut lexer = Lexer::new(content.to_string());

    lexer.tokenize();

    for token in lexer.tokens {
        println!("{:?}", token);
    }

    let grammar_path = "grammar/grammar.txt";

    let grammar_str = read_circom_file(grammar_path)?;

    let grammar = Grammar::new(&*grammar_str).unwrap();

    println!("{:#?}", grammar);

    Ok(())
}
