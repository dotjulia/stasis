use std::{fmt::Debug, iter::Peekable};

use crate::tokenizer::{Token, Tokenizer};

#[derive(Clone)]
pub enum ExpressionAST {
    SubExpression(Vec<ExpressionAST>),
    CodeBlock(Vec<String>, Vec<Vec<ExpressionAST>>),
    Terminal(Token),
}

impl Debug for ExpressionAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionAST::SubExpression(expr) => {
                f.write_str("(\n")?;
                for expr in expr {
                    f.write_fmt(format_args!("{:?},\n", expr))?;
                }
                f.write_str(")")
            }
            ExpressionAST::CodeBlock(args, cb) => {
                f.write_fmt(format_args!("{} => {{\n", args.join(" ")))?;
                for expr in cb {
                    f.write_fmt(format_args!(
                        "{:?};\n",
                        ExpressionAST::SubExpression(expr.clone())
                    ))?;
                }
                f.write_str("}")
            }
            ExpressionAST::Terminal(t) => f.write_fmt(format_args!("{:?}", t)),
        }
    }
}

#[derive(Debug)]
pub struct ExpressionTreeParsingError(usize);

impl ExpressionAST {
    pub fn parse(mut tokenizer: Tokenizer) -> Result<Self, ExpressionTreeParsingError> {
        Self::parse_in(&mut tokenizer, true)
    }
    fn parse_in(
        tokenizer: &mut Tokenizer,
        can_end: bool,
    ) -> Result<Self, ExpressionTreeParsingError> {
        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next() {
            match token {
                Token::Token(t) => tokens.push(ExpressionAST::Terminal(Token::Token(t))),
                Token::OpeningBracket => tokens.push(ExpressionAST::parse_in(tokenizer, false)?),
                Token::ClosingBracket => break,
                Token::EndStatement => {
                    tokenizer.back();
                    if !can_end {
                        return Err(ExpressionTreeParsingError(tokenizer.pos()));
                    }
                    break;
                }
                Token::OpeningCodeBlock => {
                    let mut cbtokens: Vec<Vec<ExpressionAST>> = Vec::new();
                    cbtokens.push(Vec::new());
                    let mut args = Vec::new();
                    if tokenizer.lookahead_until(&["}", "{"], "=>") {
                        while let Some(token) = tokenizer.next() {
                            match token {
                                Token::Token(token) => {
                                    if token == "=>" {
                                        break;
                                    }
                                    args.push(token);
                                }
                                _ => {
                                    println!("Found unexpected {:?}", token);
                                    return Err(ExpressionTreeParsingError(tokenizer.pos()));
                                }
                            }
                        }
                    }
                    while let Some(token) = tokenizer.next() {
                        match token {
                            Token::ClosingCodeBlock => break,
                            Token::EndStatement => cbtokens.push(Vec::new()),
                            _ => {
                                tokenizer.back();
                                cbtokens
                                    .last_mut()
                                    .unwrap()
                                    .push(ExpressionAST::parse_in(tokenizer, true)?);
                            }
                        }
                    }
                    // CodeBlock should be empty if no statement exists instead of an empty statement
                    if cbtokens.len() >= 1 && cbtokens.last().unwrap().len() == 0 {
                        cbtokens.pop();
                    }
                    // Remove double vec
                    cbtokens = cbtokens
                        .into_iter()
                        .map(|expr| {
                            if expr.len() == 1 && matches!(expr[0], ExpressionAST::SubExpression(_))
                            {
                                if let ExpressionAST::SubExpression(se) = &expr[0] {
                                    se.clone()
                                } else {
                                    panic!();
                                }
                            } else {
                                expr
                            }
                        })
                        .collect();
                    tokens.push(ExpressionAST::CodeBlock(args, cbtokens));
                }
                Token::ClosingCodeBlock => break,
            }
        }
        Ok(match tokens.len() {
            1 => tokens.swap_remove(0),
            _ => ExpressionAST::SubExpression(tokens),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub arg_tokens: Vec<String>,
    pub block: Vec<ProgramAST>,
}

#[derive(Debug, Clone)]
pub enum ProgramAST {
    FunctionCall {
        function: Box<ProgramAST>,
        arg: Box<ProgramAST>,
    },
    FunctionDef(FunctionDef),
    FunctionRef {
        token: String,
    },
    Value {
        value: usize,
    },
}

#[derive(Debug)]
pub enum ParsingError {
    UnexpectedNonFunctionToken(Token),
    UnexpectedExpressionTokenInLogicParsingPhase(Token),
    DidntParseWholeInput(ExpressionAST),
    UnexpectedEmptyExpression,
    UnexpectedTopLevelExpression(Vec<ExpressionAST>),
}

impl ProgramAST {
    fn parse_expression_iter<I>(expr: &mut Peekable<I>) -> Result<Self, ParsingError>
    where
        I: Iterator<Item = ExpressionAST>,
    {
        let mut prev_result = None;
        while let Some(first_item) = expr.next() {
            let curr_result = match first_item {
                ExpressionAST::SubExpression(sub_expression) => {
                    Ok(Self::parse_expression(sub_expression)?)
                }
                ExpressionAST::CodeBlock(args, code_block) => {
                    Ok(ProgramAST::FunctionDef(FunctionDef {
                        arg_tokens: args,
                        block: code_block
                            .into_iter()
                            .map(Self::parse_expression)
                            .collect::<Result<Vec<ProgramAST>, ParsingError>>()?,
                    }))
                }
                ExpressionAST::Terminal(terminal) => {
                    if let Token::Token(token) = terminal {
                        if let Ok(value) = token.parse::<usize>() {
                            Ok(ProgramAST::Value { value })
                        } else {
                            Ok(ProgramAST::FunctionRef { token })
                        }
                    } else {
                        Err(ParsingError::UnexpectedExpressionTokenInLogicParsingPhase(
                            terminal,
                        ))
                    }
                }
            }?;
            if prev_result.is_some() {
                if let ProgramAST::FunctionRef { token } = &curr_result {
                    if token.starts_with("`") {
                        prev_result = Some(ProgramAST::FunctionCall {
                            function: Box::from(curr_result),
                            arg: Box::from(prev_result.unwrap()),
                        });
                    } else {
                        prev_result = Some(ProgramAST::FunctionCall {
                            function: Box::from(prev_result.unwrap()),
                            arg: Box::from(curr_result),
                        });
                    }
                } else {
                    prev_result = Some(ProgramAST::FunctionCall {
                        function: Box::from(prev_result.unwrap()),
                        arg: Box::from(curr_result),
                    });
                }
            } else {
                prev_result = Some(curr_result);
            }
        }
        Ok(prev_result.ok_or(ParsingError::UnexpectedEmptyExpression)?)
        // Ok(prev_result.unwrap_or(Ok(ProgramAST::Value { value: 0 })?))
    }

    fn parse_expression(expr: Vec<ExpressionAST>) -> Result<Self, ParsingError> {
        let mut iterator = expr.into_iter().peekable();
        let result = Self::parse_expression_iter(&mut iterator)?;
        if let Some(a) = iterator.next() {
            Err(ParsingError::DidntParseWholeInput(a))
        } else {
            Ok(result)
        }
    }

    pub fn parse(ast: ExpressionAST) -> Result<Self, ParsingError> {
        match ast {
            // top level expressions not supported
            ExpressionAST::SubExpression(tle) => {
                Err(ParsingError::UnexpectedTopLevelExpression(tle))
            }
            // Top level code blocks accept no parameters
            ExpressionAST::CodeBlock(args, cb) => {
                let mut fd = FunctionDef {
                    arg_tokens: args,
                    block: Vec::new(),
                };
                for statement in cb {
                    fd.block.push(Self::parse_expression(statement)?);
                }
                Ok(ProgramAST::FunctionDef(fd))
            }
            ExpressionAST::Terminal(terminal) => match terminal {
                Token::Token(terminal) => {
                    if let Ok(value) = terminal.parse::<usize>() {
                        Ok(ProgramAST::Value { value })
                    } else {
                        Ok(ProgramAST::FunctionRef { token: terminal })
                    }
                }
                _ => Err(ParsingError::UnexpectedNonFunctionToken(terminal)),
            },
        }
    }

    pub fn finalize(&mut self) {
        match self {
            ProgramAST::FunctionCall { function, arg } => {
                function.finalize();
                arg.finalize();
            }
            ProgramAST::FunctionDef(def) => {
                def.block.iter_mut().for_each(|e| e.finalize());
            }
            ProgramAST::FunctionRef { token } => {
                if token.starts_with("`") {
                    token.remove(0);
                }
            }
            ProgramAST::Value { value: _ } => {}
        }
    }

    fn print_ast_in(&self, indentation: usize) {
        match self {
            ProgramAST::FunctionCall { function, arg } => {
                function.print_ast_in(indentation + 1);
                print!("(");
                arg.print_ast_in(indentation + 1);
                print!(")");
            }
            ProgramAST::FunctionDef(def) => {
                print!("{{ {} =>\n", def.arg_tokens.join(" "));
                for statement in &def.block {
                    print!("{}", " ".repeat((indentation + 1) * 2));
                    statement.print_ast_in(indentation + 1);
                    print!(";\n")
                }
                print!("{}}}", " ".repeat(indentation * 2));
            }
            ProgramAST::FunctionRef { token } => print!("{}", token),
            ProgramAST::Value { value } => print!("N({})", value),
        }
    }
    pub fn print_ast(&self) {
        self.print_ast_in(0)
    }
}
