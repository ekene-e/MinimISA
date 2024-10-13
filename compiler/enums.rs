use std::collections::HashMap;
use std::fmt;
use std::cmp::Ordering;
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: LexType,
    pub value: String,
    pub filename: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(typ: LexType, value: String, filename: String, line: usize, column: usize) -> Self {
        Token { typ, value, filename, line, column }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    pub typ: ValueType,
    pub raw_value: u64,
}

impl Value {
    pub fn new(typ: ValueType, raw_value: u64) -> Self {
        Value { typ, raw_value }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub funcname: String,
    pub typed_args: Vec<Value>,
    pub linenumber: usize,
    pub filename: String,
}

impl Line {
    pub fn new(funcname: String, typed_args: Vec<Value>, linenumber: usize, filename: String) -> Self {
        Line { funcname, typed_args, linenumber, filename }
    }
}

pub const NB_REG: usize = 8;
pub const NB_BIT_REG: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexType {
    MEMCOUNTER,
    OPERATION,
    DIRECTION,
    CONDITION,
    REGISTER,
    COMMENT,
    NEWLINE,
    ENDFILE,
    INCLUDE,
    NUMBER,
    LABEL,
    SKIP,
    BINARY,
    CONS,
    MISMATCH,
}

impl fmt::Display for LexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexType::MEMCOUNTER => write!(f, "MEMCOUNTER"),
            LexType::OPERATION => write!(f, "OPERATION"),
            LexType::DIRECTION => write!(f, "DIRECTION"),
            LexType::CONDITION => write!(f, "CONDITION"),
            LexType::REGISTER => write!(f, "REGISTER"),
            LexType::COMMENT => write!(f, "COMMENT"),
            LexType::NEWLINE => write!(f, "NEWLINE"),
            LexType::ENDFILE => write!(f, "ENDFILE"),
            LexType::INCLUDE => write!(f, "INCLUDE"),
            LexType::NUMBER => write!(f, "NUMBER"),
            LexType::LABEL => write!(f, "LABEL"),
            LexType::SKIP => write!(f, "SKIP"),
            LexType::BINARY => write!(f, "BINARY"),
            LexType::CONS => write!(f, "CONS"),
            LexType::MISMATCH => write!(f, "MISMATCH"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    MEMCOUNTER,
    DIRECTION,
    CONDITION,
    UCONSTANT,  
    SCONSTANT,  
    RADDRESS,   
    AADDRESS,   
    SHIFTVAL,
    REGISTER,
    LABEL,
    SIZE,
    BINARY,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::MEMCOUNTER => write!(f, "MEMCOUNTER"),
            ValueType::DIRECTION => write!(f, "DIRECTION"),
            ValueType::CONDITION => write!(f, "CONDITION"),
            ValueType::UCONSTANT => write!(f, "UCONSTANT"),
            ValueType::SCONSTANT => write!(f, "SCONSTANT"),
            ValueType::RADDRESS => write!(f, "RADDRESS"),
            ValueType::AADDRESS => write!(f, "AADDRESS"),
            ValueType::SHIFTVAL => write!(f, "SHIFTVAL"),
            ValueType::REGISTER => write!(f, "REGISTER"),
            ValueType::LABEL => write!(f, "LABEL"),
            ValueType::SIZE => write!(f, "SIZE"),
            ValueType::BINARY => write!(f, "BINARY"),
        }
    }
}
