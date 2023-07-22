use crate::ast_parser::{FunctionDef, ProgramAST};
use std::{collections::LinkedList, fmt::Debug};

#[derive(Clone)]
pub enum InterpreterFunctionDef {
    BuiltIn {
        name: String,
        arg_count: usize,
        func: fn(&mut InterpreterContext, Vec<Value>) -> Result<Value, RuntimeError>,
    },
    FunctionDef {
        name: String,
        def: FunctionDef,
    },
}

impl Debug for InterpreterFunctionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterFunctionDef::BuiltIn {
                name,
                arg_count,
                func: _,
            } => f.write_fmt(format_args!("BuiltInFunction({}, {})", name, arg_count)),
            InterpreterFunctionDef::FunctionDef { name, def } => {
                f.write_fmt(format_args!("FunctionDef({}, {:?})", name, def))
            }
        }
    }
}

impl FunctionDef {
    fn get_ith_arg_name(&self, index: usize) -> Option<String> {
        self.arg_tokens.get(index).map(|e| e.clone())
    }
}

impl InterpreterFunctionDef {
    fn get_ith_arg_name(&self, index: usize) -> Option<String> {
        match self {
            InterpreterFunctionDef::BuiltIn {
                name: _,
                arg_count: _,
                func: _,
            } => None,
            InterpreterFunctionDef::FunctionDef { name: _, def } => def.get_ith_arg_name(index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueFunction {
    pub func: InterpreterFunctionDef,
    pub bound_variables: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(usize),
    Function(ValueFunction),
}

pub struct FunctionContext(pub Vec<(String, Value)>);

pub struct InterpreterContext {
    builtins: Vec<InterpreterFunctionDef>,
    pub function_context: LinkedList<FunctionContext>,
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedFunctionReference(String),
    ValueNotAFunction(usize),
    EmptyFunction,
    ExplicitlyRaised,
    ExplicitlyRaisedMessage(&'static str),
}

impl InterpreterContext {
    pub fn new() -> Self {
        Self {
            builtins: vec![],
            function_context: LinkedList::new(),
        }
    }

    pub fn register_builtin(
        &mut self,
        name: String,
        arg_count: usize,
        func: fn(&mut InterpreterContext, Vec<Value>) -> Result<Value, RuntimeError>,
    ) {
        self.builtins.insert(
            0,
            InterpreterFunctionDef::BuiltIn {
                name,
                arg_count,
                func,
            },
        )
    }

    pub fn register_func(&mut self, name: String, func: FunctionDef) {
        self.builtins
            .insert(0, InterpreterFunctionDef::FunctionDef { name, def: func })
    }

    fn run_func(
        &mut self,
        func: FunctionDef,
        args: Vec<(String, Value)>,
    ) -> Result<Value, RuntimeError> {
        let mut last_value = None;
        self.function_context
            .push_back(FunctionContext(args.clone()));
        for s in func.block {
            last_value = Some(self.run(s)?);
        }
        self.function_context.pop_back();
        last_value.ok_or(RuntimeError::EmptyFunction)
    }

    pub fn run_anonym_func(
        &mut self,
        program: ProgramAST,
        args: Vec<(String, Value)>,
    ) -> Result<Value, RuntimeError> {
        match program {
            ProgramAST::FunctionDef(func_def) => self.run_func(func_def, args),
            _ => Err(RuntimeError::ValueNotAFunction(0)),
        }
    }

    pub fn run_func_value(
        &mut self,
        mut func: ValueFunction,
        mut args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        func.bound_variables.append(&mut args);
        match &func.func {
            InterpreterFunctionDef::BuiltIn {
                name,
                arg_count,
                func: builtin_func,
            } => builtin_func(self, func.bound_variables),
            InterpreterFunctionDef::FunctionDef { name, def } => self.run_func(
                def.clone(),
                func.bound_variables
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        (
                            func.func
                                .get_ith_arg_name(i)
                                .unwrap_or("".to_owned())
                                .clone(),
                            e.clone(),
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub fn run(&mut self, program: ProgramAST) -> Result<Value, RuntimeError> {
        match program {
            ProgramAST::FunctionCall { function, arg } => {
                let function_to_run = self.run(*function)?;
                match function_to_run {
                    Value::Number(n) => Err(RuntimeError::ValueNotAFunction(n)),
                    Value::Function(ValueFunction {
                        func,
                        mut bound_variables,
                    }) => {
                        match func {
                            InterpreterFunctionDef::FunctionDef { name, def: func } => {
                                bound_variables.push(self.run(*arg)?);
                                if bound_variables.len() >= func.arg_tokens.len() {
                                    // call
                                    self.run_func(
                                        func.clone(),
                                        bound_variables
                                            .iter()
                                            .enumerate()
                                            .map(|(i, e)| {
                                                (
                                                    func.get_ith_arg_name(i)
                                                        .unwrap_or("".to_owned())
                                                        .clone(),
                                                    e.clone(),
                                                )
                                            })
                                            .collect(),
                                    )
                                } else {
                                    Ok(Value::Function(ValueFunction {
                                        func: InterpreterFunctionDef::FunctionDef {
                                            name,
                                            def: func.clone(),
                                        },
                                        bound_variables,
                                    }))
                                }
                            }
                            InterpreterFunctionDef::BuiltIn {
                                name,
                                arg_count,
                                func,
                            } => {
                                bound_variables.push(self.run(*arg)?);
                                if bound_variables.len() >= arg_count {
                                    func(self, bound_variables)
                                } else {
                                    Ok(Value::Function(ValueFunction {
                                        func: InterpreterFunctionDef::BuiltIn {
                                            name,
                                            arg_count,
                                            func,
                                        },
                                        bound_variables,
                                    }))
                                }
                            }
                        }
                    }
                }
            }
            ProgramAST::FunctionDef(func_def) => Ok(Value::Function(ValueFunction {
                func: InterpreterFunctionDef::FunctionDef {
                    name: "anonymous".to_owned(),
                    def: func_def,
                },
                bound_variables: vec![],
            })),
            ProgramAST::Assignment { name, value } => {
                todo!()
            }
            ProgramAST::FunctionRef { token } => {
                let mut found_value = None;
                for context in self.function_context.iter() {
                    for (name, value) in context.0.iter() {
                        if *name == token {
                            found_value = Some(value);
                        }
                    }
                }
                if let Some(value) = found_value {
                    return Ok(value.clone());
                }

                if let Some(value) = self.builtins.iter().find(|f| match f {
                    InterpreterFunctionDef::BuiltIn {
                        name,
                        arg_count: _,
                        func: _,
                    } => *name == token,
                    InterpreterFunctionDef::FunctionDef { name, def: _ } => *name == token,
                }) {
                    return Ok(Value::Function(ValueFunction {
                        func: value.clone(),
                        bound_variables: vec![],
                    }));
                }
                Err(RuntimeError::UndefinedFunctionReference(token))
            }
            ProgramAST::Value { value } => Ok(Value::Number(value)),
        }
    }
}
