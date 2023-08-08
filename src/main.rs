use std::{
    env, fs,
    io::{BufRead, Write},
    time::Instant,
};

use ast_parser::{ExpressionAST, ParsingError, ProgramAST};
use clap::Parser;
use interpreter::InterpreterContext;
use tokenizer::Tokenizer;

use crate::{builtin::create_builtin_interpreter, tokenizer::Token};

mod ast_parser;
mod builtin;
mod interpreter;
mod str_ext;
mod tokenizer;

fn run_file(interpreter: &mut InterpreterContext, file: String) -> Result<(), ParsingError> {
    let file = fs::read_to_string(file).unwrap();
    let mut tokenizer = Tokenizer::new(&file);
    tokenizer.verify_syntax();
    let ast = ExpressionAST::parse(tokenizer).unwrap();
    let mut ast = ProgramAST::parse(ast)?;
    ast.finalize();
    let before = Instant::now();
    match interpreter.run_anonym_func(ast, vec![], false) {
        Ok(val) => println!("Program returned: {:?} in {:?}", val, before.elapsed()),
        Err(err) => println!("{:?}", err),
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Arguments {
    #[arg(short, long)]
    run: Option<String>,
    #[arg(short, long)]
    preload: Option<String>,
}

fn main() -> Result<(), ParsingError> {
    let mut interpreter = create_builtin_interpreter();
    let args = Arguments::parse();

    if args.run.is_some() {
        return run_file(&mut interpreter, args.run.unwrap());
    }

    if args.preload.is_some() {
        println!("{:?}", run_file(&mut interpreter, args.preload.unwrap()));
    }

    let stdin = std::io::stdin();
    print!("> ");
    std::io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        let mut line = line.unwrap().trim().to_owned();
        if !line.starts_with("{") {
            line = "{".to_owned() + &line + ";}";
        }
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

        let before = Instant::now();
        if let Some(name) = should_define_func {
            match ast {
                ProgramAST::FunctionDef(func) => interpreter.register_func(name, func),
                _ => {
                    println!("Can only define functions")
                }
            }
        } else {
            println!(
                "Return Value: {:?}, evaluated in {:?}",
                interpreter.run_anonym_func(ast, vec![], false),
                before.elapsed()
            );
        }
        print!("> ");
        std::io::stdout().flush().unwrap();
    }
    Ok(())
}
