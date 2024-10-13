use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{self, Write};
use std::error::Error;
use std::fmt;

// Define errors
#[derive(Debug)]
pub struct BackEndError(String);

impl fmt::Display for BackEndError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BackEndError: {}", self.0)
    }
}

impl Error for BackEndError {}

// Utility Queue (similar to Python's Queue)
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Define Enums for Value Types and NB_BIT_REG
#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Register,
    Other,
}

pub const NB_BIT_REG: usize = 8;  // Number of bits for register (placeholder)

// Trait to define common methods for BackEnd types
pub trait BackEnd {
    fn to_file(&mut self, filename: &str) -> io::Result<()>;
    fn to_output(&mut self);
    fn handle_line(&mut self, line: &Line) -> Result<(), BackEndError>;
    fn post_packets(&mut self) -> Option<Vec<u8>>;
}

// Placeholder struct for Line to simulate `line.funcname`, `line.typed_args`, etc.
pub struct Line {
    pub funcname: String,
    pub typed_args: Vec<TypedArg>,
    pub linenumber: usize,
}

pub struct TypedArg {
    pub typ: ValueType,
    pub raw_value: u64,
}

// Base BackEnd Implementation
pub struct BaseBackEnd {
    line_gene: Vec<Line>,
    out_queue: Queue<String>,
    huffman_tree: HashMap<String, String>,
    write_mode: String,
}

impl BaseBackEnd {
    pub fn new(huffman_tree: HashMap<String, String>, line_gene: Vec<Line>) -> Self {
        BaseBackEnd {
            line_gene,
            out_queue: Queue::new(),
            huffman_tree,
            write_mode: "w+".to_string(),
        }
    }

    fn packets(&mut self) -> impl Iterator<Item = String> + '_ {
        self.line_gene.iter().map(move |line| {
            self.handle_line(line).ok();
            while !self.out_queue.is_empty() {
                if let Some(packet) = self.out_queue.pop() {
                    return packet;
                }
            }
            String::new()
        })
    }
}

// Implementation for MemonicBackEnd
pub struct MemonicBackEnd {
    base: BaseBackEnd,
}

impl MemonicBackEnd {
    pub fn new(huffman_tree: HashMap<String, String>, line_gene: Vec<Line>) -> Self {
        MemonicBackEnd {
            base: BaseBackEnd::new(huffman_tree, line_gene),
        }
    }
}

impl BackEnd for MemonicBackEnd {
    fn to_file(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for packet in self.base.packets() {
            if !self.base.write_mode.contains("b") {
                writeln!(file, "{}", packet)?;
            } else {
                file.write_all(packet.as_bytes())?;
            }
        }
        Ok(())
    }

    fn to_output(&mut self) {
        for packet in self.base.packets() {
            println!("{}", packet);
        }
    }

    fn handle_line(&mut self, line: &Line) -> Result<(), BackEndError> {
        let funcname = &line.funcname;
        let typed_args = &line.typed_args;

        let funcname = match funcname.as_str() {
            "jumpl" | "calll" | "jumpifl" => funcname.trim_end_matches('l').to_string(),
            _ => funcname.clone(),
        };

        if funcname == "label" {
            self.base.out_queue.push(format!("{}:", typed_args[0].raw_value));
            return Ok(());
        }

        let formatted_func = format!("{:<7}", funcname);
        let realize_line: Vec<String> = typed_args
            .iter()
            .map(|arg| {
                if arg.typ == ValueType::Register {
                    format!("r{}", arg.raw_value)
                } else {
                    arg.raw_value.to_string()
                }
            })
            .collect();
        
        self.base.out_queue.push(format!("    {} {}", formatted_func, realize_line.join(" ")));
        Ok(())
    }

    fn post_packets(&mut self) -> Option<Vec<u8>> {
        None
    }
}

// CleartextBitcodeBackEnd implementation (simplified)
pub struct CleartextBitcodeBackEnd {
    base: BaseBackEnd,
    ctr: HashMap<String, String>,
    direction: HashMap<String, String>,
    conditions: HashMap<String, String>,
}

impl CleartextBitcodeBackEnd {
    pub fn new(huffman_tree: HashMap<String, String>, line_gene: Vec<Line>) -> Self {
        let ctr = vec![("pc", "00"), ("sp", "01"), ("a0", "10"), ("a1", "11")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        
        let direction = vec![("left", "0"), ("right", "1")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        
        let conditions = vec![
            ("eq", "000"), ("neq", "001"), ("sgt", "010"), ("slt", "011"),
            ("gt", "100"), ("ge", "101"), ("lt", "110"), ("v", "111"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        CleartextBitcodeBackEnd {
            base: BaseBackEnd::new(huffman_tree, line_gene),
            ctr,
            direction,
            conditions,
        }
    }

    fn binary_repr(&self, n: i64, k: usize, signed: bool) -> Result<String, BackEndError> {
        if signed && !(n >= -(2i64.pow((k - 1) as u32)) && n < 2i64.pow((k - 1) as u32)) {
            return Err(BackEndError("Number not in range".to_string()));
        }

        let mut n = if signed { (2i64.pow(k as u32) + n) % 2i64.pow(k as u32) } else { n };

        let mut binary = format!("{:b}", n);
        if binary.len() > k {
            return Err(BackEndError("Too long binary".to_string()));
        }

        while binary.len() < k {
            binary.insert(0, '0');
        }
        Ok(binary)
    }

    fn bin_register(&self, val: u64) -> Result<String, BackEndError> {
        self.binary_repr(val as i64, NB_BIT_REG, false)
    }

    fn bin_uconstant(&self, val: u64) -> Result<String, BackEndError> {
        match val {
            0..=1 => Ok("0".to_string() + &self.binary_repr(val as i64, 1, false)?),
            2..=255 => Ok("10".to_string() + &self.binary_repr(val as i64, 8, false)?),
            256..=4294967295 => Ok("110".to_string() + &self.binary_repr(val as i64, 32, false)?),
            _ => Err(BackEndError("Invalid constant: Not in range".to_string())),
        }
    }

    // Helper methods like `bin_sconstant`, `bin_direction`, etc.
}

impl BackEnd for CleartextBitcodeBackEnd {
    fn to_file(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for packet in self.base.packets() {
            if !self.base.write_mode.contains("b") {
                writeln!(file, "{}", packet)?;
            } else {
                file.write_all(packet.as_bytes())?;
            }
        }
        Ok(())
    }

    fn to_output(&mut self) {
        for packet in self.base.packets() {
            println!("{}", packet);
        }
    }

    fn handle_line(&mut self, line: &Line) -> Result<(), BackEndError> {
        let funcname = &line.funcname;
        let typed_args = &line.typed_args;

        let realize_line = vec![self
            .base
            .huffman_tree
            .get(funcname)
            .ok_or_else(|| BackEndError("Function not found".to_string()))?
            .clone()];

        for arg in typed_args {
            let method_name = match arg.typ {
                ValueType::Register => self.bin_register(arg.raw_value)?,
                _ => arg.raw_value.to_string(),
            };
            realize_line.push(method_name);
        }

        self.base.out_queue.push(realize_line.join(" "));
        Ok(())
    }

    fn post_packets(&mut self) -> Option<Vec<u8>> {
        None
    }
}

// BinaryBitcodeBackEnd (inherits from CleartextBitcodeBackEnd)
pub struct BinaryBitcodeBackEnd {
    base: CleartextBitcodeBackEnd,
    binary: String,
}

impl BinaryBitcodeBackEnd {
    pub fn new(huffman_tree: HashMap<String, String>, line_gene: Vec<Line>) -> Self {
        BinaryBitcodeBackEnd {
            base: CleartextBitcodeBackEnd::new(huffman_tree, line_gene),
            binary: String::new(),
        }
    }
}

impl BackEnd for BinaryBitcodeBackEnd {
    fn to_file(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for packet in self.base.base.packets() {
            file.write_all(packet.as_bytes())?;
        }
        Ok(())
    }

    fn to_output(&mut self) {
        for packet in self.base.base.packets() {
            println!("{}", packet);
        }
    }

    fn handle_line(&mut self, line: &Line) -> Result<(), BackEndError> {
        let funcname = &line.funcname;
        let typed_args = &line.typed_args;

        self.base.handle_line(line)?;

        while !self.base.base.out_queue.is_empty() {
            self.binary.push_str(&self.base.base.out_queue.pop().unwrap().replace(" ", ""));
        }

        let q = self.binary.len() / 8;
        let bitline = usize::from_str_radix(&self.binary[..q * 8], 2)
            .unwrap()
            .to_be_bytes();

        self.binary = self.binary[q * 8..].to_string();

        self.base.base.out_queue.push(hex::encode(bitline));

        Ok(())
    }

    fn post_packets(&mut self) -> Option<Vec<u8>> {
        if !self.binary.is_empty() {
            self.binary.push_str(&"0".repeat(8 - self.binary.len()));
            let bitline = usize::from_str_radix(&self.binary, 2)
                .unwrap()
                .to_be_bytes();
            self.binary.clear();
            Some(bitline.to_vec())
        } else {
            None
        }
    }
}
