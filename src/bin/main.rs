use std::io;
use lib::input::input::*;
use lib::lexer::lexer::Lexer;

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

    Ok(())
}
