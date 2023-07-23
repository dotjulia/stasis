use crate::{
    ast_parser::ProgramAST,
    interpreter::{self, InterpreterContext, RuntimeError, Value},
};

pub fn register_builtins(interpreter: &mut InterpreterContext) {
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
}
