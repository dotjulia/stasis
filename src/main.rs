use std::{
    env, fs,
    io::{BufRead, Write},
};

use ast_parser::{ExpressionAST, ParsingError, ProgramAST};
use interpreter::InterpreterContext;
use tokenizer::Tokenizer;

use crate::{
    interpreter::{RuntimeError, Value},
    tokenizer::Token,
};

mod ast_parser;
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
    interpreter.register_builtin("+".to_owned(), 2, |_, args| {
        if let Value::Number(num) = args[0] {
            if let Value::Number(num2) = args[1] {
                return Ok(Value::Number(num + num2));
            }
        }
        Err(interpreter::RuntimeError::ExplicitlyRaised)
    });
    interpreter.register_builtin("-".to_owned(), 2, |_, args| {
        if let Value::Number(num) = args[0] {
            if let Value::Number(num2) = args[1] {
                if num2 > num {
                    return Ok(Value::Number(usize::MAX - (num2 - num)));
                }
                return Ok(Value::Number(num - num2));
            }
        }
        Ok(Value::Number(0))
    });
    interpreter.register_builtin("not".to_owned(), 1, |_, args| {
        if let Value::Number(num) = args[0] {
            return Ok(Value::Number(if num == 0 { 1 } else { 0 }));
        }
        Err(interpreter::RuntimeError::ExplicitlyRaised)
    });
    interpreter.register_builtin("print".to_owned(), 1, |_, args| {
        println!("{:?}", args[0]);
        Ok(Value::Number(0))
    });
    interpreter.register_builtin("panic".to_owned(), 1, |_, _| {
        Err(interpreter::RuntimeError::ExplicitlyRaised)
    });
    interpreter.register_builtin("let".to_owned(), 2, |interpreter, args| match &args[0] {
        Value::Number(n) => Err(interpreter::RuntimeError::ValueNotAFunction(*n)),
        Value::Function(fname) => match &fname.func {
            interpreter::InterpreterFunctionDef::BuiltIn {
                name: _,
                arg_count: _,
                func: _,
            } => Err(interpreter::RuntimeError::ExplicitlyRaised),
            interpreter::InterpreterFunctionDef::FunctionDef { name: _, def } => {
                match def.block.get(0).ok_or(RuntimeError::ExplicitlyRaised)? {
                    ProgramAST::FunctionRef { token } => {
                        interpreter
                            .function_context
                            .back_mut()
                            .unwrap()
                            .0
                            .push((token.clone(), args[1].clone()));
                        Ok(Value::Number(0))
                    }
                    _ => Err(RuntimeError::ExplicitlyRaisedMessage(
                        "Let name has to be a function containing one token",
                    )),
                }
            }
        },
    });
    interpreter.register_builtin("if".to_owned(), 2, |interpreter, args| {
        let expression = args.get(0).unwrap();
        let body = args.get(1).unwrap();
        if let (Value::Function(expr), Value::Function(body)) = (expression, body) {
            if let Value::Number(n) = interpreter.run_func_value(expr.clone(), vec![])? {
                if n != 0 {
                    return interpreter.run_func_value(body.clone(), vec![]);
                }
            }
        }
        if let (Value::Number(expr), Value::Function(body)) = (expression, body) {
            if *expr != 0 {
                return interpreter.run_func_value(body.clone(), vec![]);
            }
        }
        Ok(Value::Number(0))
    });

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
