use std::{
    env, fs,
    io::{BufRead, Write},
};

use ast_parser::{ExpressionAST, ParsingError, ProgramAST};
use interpreter::InterpreterContext;
use tokenizer::Tokenizer;

use crate::{builtin::register_builtins, tokenizer::Token};

mod ast_parser;
mod builtin;
mod interpreter;
mod str_ext;
mod tokenizer;

fn run_file(mut interpreter: InterpreterContext) -> Result<(), ParsingError> {
    let file = fs::read_to_string(env::args().skip(1).next().unwrap()).unwrap();
    let mut tokenizer = Tokenizer::new(&file);
    tokenizer.verify_syntax();
    let ast = ExpressionAST::parse(tokenizer).unwrap();
    let mut ast = ProgramAST::parse(ast)?;
    ast.finalize();
    match interpreter.run_anonym_func(ast, vec![]) {
        Ok(val) => println!("Program returned: {:?}", val),
        Err(err) => println!("{:?}", err),
    }
    Ok(())
}

fn main() -> Result<(), ParsingError> {
    // let str = "{ test (test `test) test { (test test {test; test;}); test; test; } test; blabla }";
    // let str = "{{print 5; + 1 arg1;} 2;}";
    // let str = "{(+ 1) `test;}";
    // let str = "{{arg1 arg2 => + arg1 arg2;} 1 2;}";
    // let str = "{1 2 3 4;}";

    // tokenizer.for_each(|e| println!("{:?}", e));
    let mut interpreter = InterpreterContext::new();
    register_builtins(&mut interpreter);
    if env::args().len() > 1 {
        return run_file(interpreter);
    }

    let stdin = std::io::stdin();
    print!("> ");
    std::io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut tokenizer = Tokenizer::new(&line);
        tokenizer.verify_syntax();
        let mut should_define_func = None;
        if let Some(Token::Token(token)) = tokenizer.next() {
            if token == ":" {
                should_define_func = if let Token::Token(name) = tokenizer.next().unwrap() {
                    Some(name)
                } else {
                    panic!()
                };
            }
        } else {
            tokenizer.back();
        }
        let ast = match ExpressionAST::parse(tokenizer) {
            Ok(o) => o,
            Err(e) => {
                println!("{:?}", e);
                print!("> ");
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
        let mut ast = match ProgramAST::parse(ast) {
            Ok(o) => o,
            Err(e) => {
                println!("{:?}", e);
                print!("> ");
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
        ast.finalize();

        if let Some(name) = should_define_func {
            match ast {
                ProgramAST::FunctionDef(func) => interpreter.register_func(name, func),
                _ => {
                    println!("Can only define functions")
                }
            }
        } else {
            println!(
                "Return Value: {:?}",
                interpreter.run_anonym_func(ast, vec![])
            );
        }
        print!("> ");
        std::io::stdout().flush().unwrap();
    }
    Ok(())
}
