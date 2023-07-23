use wasm_bindgen::prelude::wasm_bindgen;

pub mod ast_parser;
pub mod builtin;
pub mod interpreter;
pub mod str_ext;
pub mod tokenizer;

#[cfg(feature = "wasm")]
static mut INTERPRETER: Option<interpreter::InterpreterContext> = None;
#[cfg(feature = "wasm")]
static mut STDOUT: String = String::new();

#[cfg(feature = "wasm")]
fn get_interpreter() -> &'static mut interpreter::InterpreterContext {
    use builtin::register_builtins;

    use crate::interpreter::InterpreterContext;

    if unsafe { INTERPRETER.is_none() } {
        unsafe {
            INTERPRETER = Some(InterpreterContext::new());
            register_builtins(&mut INTERPRETER.as_mut().unwrap());
            INTERPRETER
                .as_mut()
                .unwrap()
                .register_builtin("print".to_owned(), 1, |_, mut args| {
                    STDOUT += &format!("{:?}\n", args[0]);
                    Ok(args.swap_remove(0))
                });
            INTERPRETER
                .as_mut()
                .unwrap()
                .register_builtin("helppredef".to_owned(), 0, |_, _| {
                    STDOUT += r#"+ a1 a2; // adds the two numeric values
- a1 a2; // subtracts the two values
not a1; // returns 1 when a1 is 0 and 0 otherwise
print a1; // prints the argument to stdout
let a1 a0; // defines a new variable (eg. let { a; } 4;) more info: github.com/dotjulia/stasis
if expr body; // executes the expression and then executes the body if it is not 0
"#;
                    Ok(interpreter::Value::Number(0))
                });
        }
    }
    let test = unsafe { INTERPRETER.as_mut().unwrap() };
    return test;
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn test(input: String) -> String {
    use tokenizer::Tokenizer;

    use crate::ast_parser::{ExpressionAST, ProgramAST};
    unsafe { STDOUT = String::new() };

    let interpreter = get_interpreter();
    let mut tokenizer = Tokenizer::new(&input);
    tokenizer.verify_syntax();

    let mut should_define_func = None;
    if let Some(tokenizer::Token::Token(token)) = tokenizer.next() {
        if token == ":" {
            should_define_func = if let tokenizer::Token::Token(name) = tokenizer.next().unwrap() {
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
        Err(e) => return format!("{:?}", e),
    };
    let mut ast = match ProgramAST::parse(ast) {
        Ok(o) => o,
        Err(e) => return format!("{:?}", e),
    };
    ast.finalize();

    let first_output = if let Some(name) = should_define_func {
        match ast {
            ProgramAST::FunctionDef(func) => {
                interpreter.register_func(name, func);
                "Ok".to_owned()
            }
            _ => "Can only define functions".to_owned(),
        }
    } else {
        format!(
            "Return Value: {:?}",
            interpreter.run_anonym_func(ast, vec![])
        )
    };
    format!("{}\n{:?}", unsafe { STDOUT.clone() }, first_output,)
}
