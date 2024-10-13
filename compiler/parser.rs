use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::process;
use std::fmt;

// Define Token and Value structs
#[derive(Debug, Clone)]
struct Token {
    typ: LexType,
    value: String,
    filename: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct Value {
    typ: ValueType,
    raw_value: String,
}

#[derive(Debug, Clone)]
struct Line {
    funcname: String,
    typed_args: Vec<Value>,
    linenumber: usize,
    filename: String,
}

// LexType and ValueType enums
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum LexType {
    Operation,
    Comment,
    EndFile,
    NewLine,
    Label,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum ValueType {
    MemCounter,
    Direction,
    Condition,
    UConstant,
    SConstant,
    RAddress,
    ShiftVal,
    Size,
    Register,
    Label,
    Binary,
}

#[derive(Debug)]
struct ParserError(String);

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParserError: {}", self.0)
    }
}

impl std::error::Error for ParserError {}

// Utility functions for stack and queue management
struct Stack<T> {
    inner: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { inner: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.inner.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn peek(&self) -> Option<&T> {
        self.inner.last()
    }
}

struct Queue<T> {
    inner: VecDeque<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Queue { inner: VecDeque::new() }
    }

    fn push(&mut self, item: T) {
        self.inner.push_back(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// The parser structure
struct Parser<'a> {
    lexer_gen: &'a mut dyn Iterator<Item = Token>,
    stack: Stack<Token>,
    out_stack: Stack<Line>,
    functions: HashMap<String, HashMap<Vec<LexType>, (String, Vec<ValueType>)>>,
    labels: HashMap<String, usize>,
}

impl<'a> Parser<'a> {
    fn new(
        lexer_gen: &'a mut dyn Iterator<Item = Token>,
        possible_transitions: &HashMap<String, Vec<String>>,
        asr_specs: &HashMap<String, Vec<ValueType>>,
        types_specs: &HashMap<LexType, Vec<ValueType>>,
    ) -> Self {
        let mut functions = HashMap::new();
        let rev_types_specs = inv_dict_list(types_specs);

        for (funcname, list_asr_funcname) in possible_transitions {
            let mut func_map = HashMap::new();
            for asr_funcname in list_asr_funcname {
                let asr_args = asr_specs.get(asr_funcname).unwrap();
                let preasr_args = asr_args
                    .iter()
                    .map(|x| rev_types_specs.get(x).unwrap().clone())
                    .collect::<Vec<LexType>>();
                func_map.insert(preasr_args, (asr_funcname.clone(), asr_args.clone()));
            }
            functions.insert(funcname.clone(), func_map);
        }

        Parser {
            lexer_gen,
            stack: Stack::new(),
            out_stack: Stack::new(),
            functions,
            labels: HashMap::new(),
        }
    }

    fn run(&mut self) -> Result<(), ParserError> {
        for token in self.lexer_gen {
            match token.typ {
                LexType::Comment => continue,
                LexType::EndFile => continue,
                LexType::NewLine => {
                    self.handle_one()?;
                    while let Some(out_line) = self.out_stack.pop() {
                        println!("{:?}", out_line);
                    }
                }
                _ => self.stack.push(token),
            }
        }
        Ok(())
    }

    fn unstack_until_operation(&mut self) -> Result<Vec<Token>, ParserError> {
        let mut res = Queue::new();

        while let Some(token) = self.stack.pop() {
            if token.typ != LexType::Operation {
                res.push(token);
            } else {
                return Ok(res.inner.into_iter().collect());
            }
        }

        Err(ParserError("Couldn't find operation on the stack".to_string()))
    }

    fn handle_one(&mut self) -> Result<(), ParserError> {
        let res = self.unstack_until_operation()?;

        let fun_name = &res[0].value;
        let args_types = res.iter().skip(1).map(|x| x.typ).collect::<Vec<LexType>>();

        if let Some(func_map) = self.functions.get(fun_name) {
            if let Some((funcname, goal_args_type)) = func_map.get(&args_types) {
                let args_values = res.iter().skip(1).map(|x| x.value.clone()).collect::<Vec<_>>();
                let mut typed_args = Vec::new();

                if args_values.len() != goal_args_type.len() {
                    return Err(ParserError(format!(
                        "Incorrect number of arguments for function {}",
                        funcname
                    )));
                }

                for (value, goal_type) in args_values.iter().zip(goal_args_type) {
                    let method_name = format!("read_{}", goal_type.to_string().to_lowercase());
                    if let Some(typed_value) = self.read_value(goal_type, value)? {
                        typed_args.push(typed_value);
                    } else {
                        return Err(ParserError(format!(
                            "Couldn't read {}",
                            goal_type.to_string()
                        )));
                    }
                }

                self.out_stack.push(Line {
                    funcname: funcname.clone(),
                    typed_args,
                    linenumber: res[0].line,
                    filename: res[0].filename.clone(),
                });

                Ok(())
            } else {
                Err(ParserError(format!(
                    "Arguments types don't match function: {}",
                    fun_name
                )))
            }
        } else {
            Err(ParserError(format!("Function not found: {}", fun_name)))
        }
    }

    fn read_value(&self, goal_type: &ValueType, value: &str) -> Result<Option<Value>, ParserError> {
        match goal_type {
            ValueType::MemCounter => Ok(Some(Value {
                typ: *goal_type,
                raw_value: value.to_string(),
            })),
            ValueType::Direction => Ok(Some(Value {
                typ: *goal_type,
                raw_value: value.to_string(),
            })),
            ValueType::Condition => Ok(Some(Value {
                typ: *goal_type,
                raw_value: value.to_string(),
            })),
            ValueType::UConstant => {
                let parsed_value = value.parse::<u64>().map_err(|_| {
                    ParserError("Couldn't parse unsigned constant".to_string())
                })?;
                if parsed_value < (1 << 64) {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("UConstant out of range".to_string()))
                }
            }
            ValueType::SConstant => {
                let parsed_value = value.parse::<i64>().map_err(|_| {
                    ParserError("Couldn't parse signed constant".to_string())
                })?;
                if parsed_value >= -(1 << 63) && parsed_value < (1 << 63) {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("SConstant out of range".to_string()))
                }
            }
            ValueType::RAddress => {
                let parsed_value = value.parse::<i64>().map_err(|_| {
                    ParserError("Couldn't parse relative address".to_string())
                })?;
                if parsed_value >= -(1 << 63) && parsed_value < (1 << 63) {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("RAddress out of range".to_string()))
                }
            }
            ValueType::ShiftVal => {
                let parsed_value = value.parse::<u64>().map_err(|_| {
                    ParserError("Couldn't parse shift value".to_string())
                })?;
                if parsed_value < (1 << 6) {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("ShiftVal out of range".to_string()))
                }
            }
            ValueType::Size => {
                let parsed_value = value.parse::<u64>().map_err(|_| {
                    ParserError("Couldn't parse size value".to_string())
                })?;
                let valid_sizes = [1, 4, 8, 16, 32, 64];
                if valid_sizes.contains(&parsed_value) {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("Size out of range".to_string()))
                }
            }
            ValueType::Register => {
                let parsed_value = value.parse::<u64>().map_err(|_| {
                    ParserError("Couldn't parse register value".to_string())
                })?;
                if parsed_value < NB_REG as u64 {
                    Ok(Some(Value {
                        typ: *goal_type,
                        raw_value: parsed_value.to_string(),
                    }))
                } else {
                    Err(ParserError("Register out of range".to_string()))
                }
            }
            ValueType::Label => Ok(Some(Value {
                typ: *goal_type,
                raw_value: value.to_string(),
            })),
            ValueType::Binary => Ok(Some(Value {
                typ: *goal_type,
                raw_value: value[1..].to_string(),
            })),
        }
    }
}

fn inv_dict_list(
    types_specs: &HashMap<LexType, Vec<ValueType>>,
) -> HashMap<ValueType, LexType> {
    let mut inv_map = HashMap::new();
    for (key, value) in types_specs {
        for val_type in value {
            inv_map.insert(*val_type, *key);
        }
    }
    inv_map
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer_gen: Box<dyn Iterator<Item = Token>> = Box::new(vec![].into_iter());
    let possible_transitions = HashMap::new(); 
    let asr_specs = HashMap::new();
    let types_specs = HashMap::new(); 

    let mut parser = Parser::new(&mut lexer_gen, &possible_transitions, &asr_specs, &types_specs);

    parser.run()?;
    Ok(())
}
