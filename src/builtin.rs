use std::process::Command;

use crate::{
    ast_parser::ProgramAST,
    interpreter::{
        self, InterpreterContext, InterpreterFunctionDef, RuntimeError, Value, ValueFunction,
    },
};

#[derive(Debug)]
struct Allocation {
    start_addr: usize,
    data: Vec<usize>,
}

#[derive(Debug)]
struct BuiltinState {
    heap: Vec<Allocation>,
}

impl BuiltinState {
    fn alloc(&mut self, size: usize) -> usize {
        match self.heap.last() {
            Some(s) => {
                let start_addr = s.start_addr + s.data.len();
                self.heap.push(Allocation {
                    start_addr,
                    data: vec![0; size],
                });
                start_addr
            }
            None => {
                self.heap.push(Allocation {
                    start_addr: 0x1024,
                    data: vec![0; size],
                });
                0x1024
            }
        }
    }

    fn get(&self, addr: usize) -> Option<usize> {
        for allocation in &self.heap {
            if addr >= allocation.start_addr && addr < allocation.start_addr + allocation.data.len()
            {
                return Some(allocation.data[addr - allocation.start_addr]);
            }
        }
        None
    }

    fn set(&mut self, addr: usize, value: usize) -> bool {
        for allocation in &mut self.heap {
            if addr >= allocation.start_addr && addr < allocation.start_addr + allocation.data.len()
            {
                allocation.data[addr - allocation.start_addr] = value;
                return true;
            }
        }
        false
    }
}

pub fn create_builtin_interpreter() -> InterpreterContext {
    let mut interpreter = InterpreterContext::new();
    interpreter.state = Box::from(BuiltinState { heap: vec![] });
    register_builtins(&mut interpreter);
    interpreter
}

pub fn register_builtins(interpreter: &mut InterpreterContext) {
    interpreter.register_builtin("alloc".to_owned(), 1, |interpreter, args| match interpreter
        .state
        .downcast_mut::<BuiltinState>(
    ) {
        Some(state) => match &args[0] {
            Value::Number(n) => Ok(Value::Number(state.alloc(*n))),
            Value::Function(_) => Err(RuntimeError::ExplicitlyRaisedMessage(
                "alloc param should be number",
            )),
        },
        None => Err(RuntimeError::ExplicitlyRaisedMessage(
            "Unexpected interpreter state",
        )),
    });
    interpreter.register_builtin("len".to_owned(), 1, |interpreter, args| {
        let addr = match args[0] {
            Value::Number(n) => n,
            Value::Function(_) => {
                return Err(RuntimeError::ExplicitlyRaisedMessage(
                    "len needs addr as parameter",
                ))
            }
        };
        match interpreter.state.downcast_mut::<BuiltinState>() {
            Some(state) => match state.heap.iter().find(|e| e.start_addr == addr) {
                Some(a) => Ok(Value::Number(a.data.len())),
                None => Err(RuntimeError::ExplicitlyRaisedMessage(
                    "allocation for addr not found",
                )),
            },
            None => Err(RuntimeError::ExplicitlyRaisedMessage(
                "interpreter in invalid state",
            )),
        }
    });
    interpreter.register_builtin("*".to_owned(), 1, |interpreter, args| {
        let addr = match args[0] {
            Value::Number(n) => n,
            Value::Function(_) => {
                return Err(RuntimeError::ExplicitlyRaisedMessage(
                    "* needs addr as parameter",
                ))
            }
        };
        match interpreter.state.downcast_mut::<BuiltinState>() {
            Some(state) => match state.get(addr) {
                Some(value) => Ok(Value::Number(value as usize)),
                None => Err(RuntimeError::ExplicitlyRaisedMessage(
                    "Deref addr not found",
                )),
            },
            None => Err(RuntimeError::ExplicitlyRaisedMessage(
                "interpreter in invalid state",
            )),
        }
    });
    interpreter.register_builtin("=".to_owned(), 2, |interpreter, args| {
        let addr = match args[0] {
            Value::Number(n) => n,
            Value::Function(_) => {
                return Err(RuntimeError::ExplicitlyRaisedMessage(
                    "len needs addr as parameter",
                ))
            }
        };
        let value = match args[1] {
            Value::Number(n) => n,
            Value::Function(_) => {
                return Err(RuntimeError::ExplicitlyRaisedMessage(
                    "len needs addr as parameter",
                ))
            }
        };
        match interpreter.state.downcast_mut::<BuiltinState>() {
            Some(state) => match state.set(addr, value) {
                true => Ok(Value::Number(1)),
                false => Err(RuntimeError::ExplicitlyRaisedMessage(
                    "address not previously allocd",
                )),
            },
            None => Err(RuntimeError::ExplicitlyRaisedMessage(
                "interpreter in invalid state",
            )),
        }
    });
    interpreter.register_builtin("number?".to_owned(), 1, |_, args| match args[0] {
        Value::Number(_) => Ok(Value::Number(1)),
        Value::Function(_) => Ok(Value::Number(0)),
    });
    interpreter.register_builtin("bind".to_owned(), 2, |interpreter, args| {
        if let (Value::Function(func), Value::Function(to_return)) = (&args[0], &args[1]) {
            if let InterpreterFunctionDef::FunctionDef { name: _, def } = &func.func {
                let mut tokens_to_bind = Vec::new();
                for a in def.block.iter() {
                    match a {
                        ProgramAST::FunctionRef { token } => tokens_to_bind.push(token.clone()),
                        _ => {}
                    };
                }
                let mut retval = to_return.clone();
                for (name, value) in tokens_to_bind
                    .into_iter()
                    .map(|t| (t.clone(), interpreter.lookup(&t)))
                {
                    if let Some(value) = value {
                        retval.bound_context.push((name, value));
                    } else {
                        return Err(RuntimeError::ExplicitlyRaisedMessage(
                            "token to bind not found",
                        ));
                    }
                }
                return Ok(Value::Function(retval));
            }
        }
        Err(RuntimeError::ExplicitlyRaisedMessage(
            "Wrong parameter to bind",
        ))
    });
    interpreter.register_builtin(
        "printstr".to_owned(),
        1,
        |interpreter, args| match interpreter.state.downcast_mut::<BuiltinState>() {
            Some(state) => match args[0] {
                Value::Number(n) => match state.heap.iter().find(|a| a.start_addr == n) {
                    Some(v) => {
                        println!(
                            "{}",
                            std::str::from_utf8(
                                &v.data.iter().map(|&e| e as u8).collect::<Vec<u8>>()
                            )
                            .unwrap()
                        );
                        Ok(Value::Number(n))
                    }
                    None => Err(RuntimeError::ExplicitlyRaisedMessage(
                        "allocation not found",
                    )),
                },
                Value::Function(_) => Err(RuntimeError::ExplicitlyRaisedMessage(
                    "need ptr to print string",
                )),
            },
            None => Err(RuntimeError::ExplicitlyRaisedMessage(
                "invalid interpreter state",
            )),
        },
    );
    interpreter.register_builtin("+".to_owned(), 2, |_, args| {
        if let Value::Number(num) = args[0] {
            if let Value::Number(num2) = args[1] {
                return Ok(Value::Number(num + num2));
            }
        }
        Err(interpreter::RuntimeError::ExplicitlyRaised)
    });
    interpreter.register_builtin("mul".to_owned(), 2, |_, args| {
        if let Value::Number(num) = args[0] {
            if let Value::Number(num2) = args[1] {
                return Ok(Value::Number(num * num2));
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
        Ok(args[0].clone())
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
    interpreter.register_builtin("if".to_owned(), 3, |interpreter, args| {
        let expression = args.get(0).unwrap();
        let body = args.get(1).unwrap();
        let elseval = args.get(2).unwrap();
        if let (Value::Function(expr), Value::Function(body), Value::Function(elseval)) =
            (expression, body, elseval)
        {
            if let Value::Number(n) = interpreter.run_func_value(expr.clone(), vec![])? {
                if n != 0 {
                    return interpreter.run_func_value(body.clone(), vec![]);
                } else {
                    return interpreter.run_func_value(elseval.clone(), vec![]);
                }
            }
        }
        if let (Value::Number(expr), Value::Function(body), Value::Function(elseval)) =
            (expression, body, elseval)
        {
            if *expr != 0 {
                return interpreter.run_func_value(body.clone(), vec![]);
            } else {
                return interpreter.run_func_value(elseval.clone(), vec![]);
            }
        }
        Ok(Value::Number(0))
    });
    interpreter.register_builtin("read".to_owned(), 0, |interpreter, _| {
        let stdin = std::io::stdin();
        let read = std::io::BufRead::lines(stdin.lock())
            .next()
            .unwrap()
            .unwrap();
        match interpreter.state.downcast_mut::<BuiltinState>() {
            Some(state) => {
                let addr = state.alloc(read.len());
                for (i, c) in read.chars().enumerate() {
                    state.set(addr + i, c as usize);
                }
                Ok(Value::Number(addr))
            }
            None => Err(RuntimeError::ExplicitlyRaisedMessage(
                "interpreter in invalid state",
            )),
        }
    });
    interpreter.register_builtin("exec".to_owned(), 1, |interpreter, args| {
        if let Value::Number(strptr) = args[0] {
            match interpreter.state.downcast_mut::<BuiltinState>() {
                Some(state) => match state.heap.iter().find(|a| a.start_addr == strptr) {
                    Some(alloc) => {
                        let command =
                            String::from_utf8(alloc.data.iter().map(|e| *e as u8).collect())
                                .unwrap();
                        let return_str = match Command::new("sh").arg("-c").arg(command).output() {
                            Ok(ok) => String::from_utf8(ok.stdout).unwrap(),
                            Err(e) => e.to_string(),
                        };
                        let addr = state.alloc(return_str.len());
                        for (i, e) in return_str.chars().enumerate() {
                            state.set(addr + i, e as usize);
                        }
                        return Ok(Value::Number(addr));
                    }
                    _ => {}
                },
                None => {
                    return Err(RuntimeError::ExplicitlyRaisedMessage(
                        "invalid interpreter state",
                    ))
                }
            }
        }
        Err(RuntimeError::ExplicitlyRaisedMessage("Invalid str ptr"))
    });
    interpreter.register_builtin("inspect".to_owned(), 1, |interpreter, args| {
        if let Value::Function(ValueFunction {
            func: InterpreterFunctionDef::FunctionDef { name: _, def },
            bound_context: _,
            bound_variables: _,
        }) = &args[0]
        {
            let mut list = Vec::new();
            for statement in &def.block {
                match &statement {
                    ProgramAST::Value { value } => list.push(value.to_string()),
                    ProgramAST::FunctionRef { token } => list.push(token.clone()),
                    _ => {}
                }
            }
            match interpreter.state.downcast_mut::<BuiltinState>() {
                Some(state) => {
                    let addr = state.alloc(list.len());
                    for (i, e) in list.into_iter().enumerate() {
                        let addr_str = state.alloc(e.len());
                        state.set(addr + i, addr_str);
                        for (i, e) in e.chars().enumerate() {
                            state.set(addr_str + i, e as usize);
                        }
                    }
                    return Ok(Value::Number(addr));
                }
                None => {
                    return Err(RuntimeError::ExplicitlyRaisedMessage(
                        "invalid interpreter state",
                    ))
                }
            }
        }
        Err(RuntimeError::ExplicitlyRaisedMessage(
            "Parameter needs to be a function of values",
        ))
    });
}
