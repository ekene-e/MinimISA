use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::num::ParseIntError;
use regex::Regex;

// Structs equivalent to namedtuples
#[derive(Debug, Clone)]
struct Command {
    opcode: String,
    operands: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct Condition {
    opcode: String,
}

// Commands equivalent to the Python named tuples
fn init_commands() -> HashMap<&'static str, Command> {
    let mut commands = HashMap::new();
    commands.insert("add2", Command { opcode: "0000".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("add2i", Command { opcode: "0001".to_string(), operands: vec!["reg", "const"] });
    commands.insert("sub2", Command { opcode: "0010".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("sub2i", Command { opcode: "0011".to_string(), operands: vec!["reg", "const"] });
    commands.insert("cmp", Command { opcode: "0100".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("cmpi", Command { opcode: "0101".to_string(), operands: vec!["reg", "sconst"] });
    commands.insert("let", Command { opcode: "0110".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("leti", Command { opcode: "0111".to_string(), operands: vec!["reg", "sconst"] });
    commands.insert("shift", Command { opcode: "1000".to_string(), operands: vec!["dir", "reg", "shiftval"] });
    commands.insert("readze", Command { opcode: "10010".to_string(), operands: vec!["ctr", "size", "reg"] });
    commands.insert("pop", Command { opcode: "1001001".to_string(), operands: vec!["size", "reg"] });
    commands.insert("readse", Command { opcode: "10011".to_string(), operands: vec!["ctr", "size", "reg"] });
    commands.insert("jump", Command { opcode: "1010".to_string(), operands: vec!["addr_signed"] });
    commands.insert("jumpif", Command { opcode: "1011".to_string(), operands: vec!["cond", "addr_signed"] });
    commands.insert("or2", Command { opcode: "110000".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("or2i", Command { opcode: "110001".to_string(), operands: vec!["reg", "const"] });
    commands.insert("and2", Command { opcode: "110010".to_string(), operands: vec!["reg", "reg"] });
    commands.insert("and2i", Command { opcode: "110011".to_string(), operands: vec!["reg", "const"] });
    commands.insert("write", Command { opcode: "110100".to_string(), operands: vec!["ctr", "size", "reg"] });
    commands.insert("call", Command { opcode: "110101".to_string(), operands: vec!["addr_signed"] });
    commands.insert("setctr", Command { opcode: "110110".to_string(), operands: vec!["ctr", "reg"] });
    commands.insert("getctr", Command { opcode: "110111".to_string(), operands: vec!["ctr", "reg"] });
    commands.insert("push", Command { opcode: "1110000".to_string(), operands: vec!["size", "reg"] });
    commands.insert("return", Command { opcode: "1110001".to_string(), operands: vec![] });
    commands.insert("add3", Command { opcode: "1110010".to_string(), operands: vec!["reg", "reg", "reg"] });
    commands.insert("add3i", Command { opcode: "1110011".to_string(), operands: vec!["reg", "reg", "const"] });
    commands.insert("sub3", Command { opcode: "1110100".to_string(), operands: vec!["reg", "reg", "reg"] });
    commands.insert("sub3i", Command { opcode: "1110101".to_string(), operands: vec!["reg", "reg", "const"] });
    commands.insert("and3", Command { opcode: "1110110".to_string(), operands: vec!["reg", "reg", "reg"] });
    commands.insert("and3i", Command { opcode: "1110111".to_string(), operands: vec!["reg", "reg", "const"] });
    commands.insert("or3", Command { opcode: "1111000".to_string(), operands: vec!["reg", "reg", "reg"] });
    commands.insert("or3i", Command { opcode: "1111001".to_string(), operands: vec!["reg", "reg", "const"] });
    commands.insert("xor3", Command { opcode: "1111010".to_string(), operands: vec!["reg", "reg", "reg"] });
    commands.insert("xor3i", Command { opcode: "1111011".to_string(), operands: vec!["reg", "reg", "const"] });
    commands.insert("asr3", Command { opcode: "1111100".to_string(), operands: vec!["reg", "reg", "shiftval"] });
    commands.insert("rese1", Command { opcode: "1111101".to_string(), operands: vec![] });
    commands.insert("rese2", Command { opcode: "1111110".to_string(), operands: vec![] });
    commands.insert("rese3", Command { opcode: "1111111".to_string(), operands: vec![] });
    commands
}

// Conditions equivalent to the Python named tuples
fn init_conditions() -> HashMap<&'static str, Condition> {
    let mut conditions = HashMap::new();
    conditions.insert("eq", Condition { opcode: "000".to_string() });
    conditions.insert("z", Condition { opcode: "000".to_string() });
    conditions.insert("neq", Condition { opcode: "001".to_string() });
    conditions.insert("nz", Condition { opcode: "001".to_string() });
    conditions.insert("sgt", Condition { opcode: "010".to_string() });
    conditions.insert("slt", Condition { opcode: "011".to_string() });
    conditions.insert("gt", Condition { opcode: "100".to_string() });
    conditions.insert("ge", Condition { opcode: "101".to_string() });
    conditions.insert("nc", Condition { opcode: "101".to_string() });
    conditions.insert("lt", Condition { opcode: "110".to_string() });
    conditions.insert("c", Condition { opcode: "110".to_string() });
    conditions.insert("v", Condition { opcode: "111".to_string() });
    conditions
}

#[derive(Debug)]
struct TokenError(String);

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenError: {}", self.0)
    }
}

impl std::error::Error for TokenError {}

const NB_REG: u32 = 8;
const NB_BIT_REG: u32 = (NB_REG as f64).log2().ceil() as u32;

fn binary_repr(n: i64, k: u32, signed: bool) -> Result<String, TokenError> {
    if signed && (n < -(1 << (k - 1)) || n >= (1 << (k - 1))) {
        return Err(TokenError("Number not in range".to_string()));
    }

    let mut n = if signed { (1 << k) + n } else { n } as u64;
    let unfilled = format!("{:b}", n);
    if unfilled.len() > k as usize {
        return Err(TokenError("Too long binary".to_string()));
    }

    Ok(format!("{:0>width$}", unfilled, width = k as usize))
}

// Regular expressions
lazy_static! {
    static ref RE_REG: Regex = Regex::new(r"^r([0-9]+)$").unwrap();
    static ref RE_CONST: Regex = Regex::new(r"^([+-]?0x[0-9A-Fa-f]+)|([+-]?[0-9]+)$").unwrap();
    static ref RE_DIR: Regex = Regex::new(r"(left)|(right)").unwrap();
    static ref RE_SHIFTVAL: Regex = Regex::new(r"^(0x[0-9A-Fa-f]+)|([0-9]+)$").unwrap();
    static ref RE_CTR: Regex = Regex::new(r"(pc|sp|a0|a1)").unwrap();
    static ref RE_SIZE: Regex = Regex::new(r"^(0x[0-9A-Fa-f]+)|([0-9]+)$").unwrap();
    static ref RE_ADDR_SIGNED: Regex = Regex::new(r"^([+-]?0x[0-9A-Fa-f]+)|([+-]?[0-9]+)$").unwrap();
    static ref RE_COND: Regex = Regex::new(r"(eq|z|neq|nz|sgt|slt|gt|ge|nc|lt|c|v)").unwrap();
}

fn asm_reg(s: &str) -> Result<String, TokenError> {
    let res = RE_REG.captures(s).ok_or(TokenError("Invalid register syntax".to_string()))?;
    let val: u32 = res[1].parse().map_err(|_| TokenError("Invalid register number".to_string()))?;
    if val >= NB_REG {
        return Err(TokenError("Invalid register number".to_string()));
    }
    binary_repr(val as i64, NB_BIT_REG, false)
}

fn asm_const(s: &str) -> Result<String, TokenError> {
    let res = RE_CONST.captures(s).ok_or(TokenError("Invalid constant syntax".to_string()))?;
    let val: i64 = if let Some(hex_val) = res.get(1) {
        i64::from_str_radix(hex_val.as_str().trim_start_matches("0x"), 16)?
    } else {
        res.get(2).unwrap().as_str().parse().map_err(|_| TokenError("Invalid decimal constant".to_string()))?
    };

    if val < (1 << 1) {
        Ok(format!("0{}", binary_repr(val, 1, false)?))
    } else if val < (1 << 8) {
        Ok(format!("10{}", binary_repr(val, 8, false)?))
    } else if val < (1 << 32) {
        Ok(format!("110{}", binary_repr(val, 32, false)?))
    } else if val < (1 << 64) {
        Ok(format!("111{}", binary_repr(val, 64, false)?))
    } else {
        Err(TokenError("Invalid constant: not in range".to_string()))
    }
}

fn asm_shiftval(s: &str) -> Result<String, TokenError> {
    let res = RE_SHIFTVAL.captures(s).ok_or(TokenError("Invalid shiftval syntax".to_string()))?;
    let val: u64 = if let Some(hex_val) = res.get(1) {
        u64::from_str_radix(hex_val.as_str().trim_start_matches("0x"), 16)?
    } else {
        res.get(2).unwrap().as_str().parse()?
    };

    if val == 1 {
        Ok(binary_repr(val as i64, 1, false)?)
    } else if val < (1 << 6) {
        Ok(format!("0{}", binary_repr(val as i64, 6, false)?))
    } else {
        Err(TokenError("Invalid shiftval: not in range".to_string()))
    }
}

fn asm_line(s: &str, commands: &HashMap<&str, Command>) -> Result<String, TokenError> {
    let cmds: Vec<&str> = s.split_whitespace().collect();
    if cmds.is_empty() {
        return Ok("".to_string());
    }

    let cmd = commands.get(cmds[0]).ok_or(TokenError("Unknown command".to_string()))?;
    let args = &cmds[1..];

    let mut linecode = vec![cmd.opcode.clone()];

    if cmd.operands.len() != args.len() {
        return Err(TokenError("Incorrect number of arguments".to_string()));
    }

    for (&operand, &arg) in cmd.operands.iter().zip(args.iter()) {
        let code = match operand {
            "reg" => asm_reg(arg)?,
            "const" => asm_const(arg)?,
            "shiftval" => asm_shiftval(arg)?,
            _ => return Err(TokenError(format!("Unknown operand type: {}", operand))),
        };
        linecode.push(code);
    }

    Ok(linecode.join(" "))
}

fn asm_doc(s: &str, commands: &HashMap<&str, Command>) -> Result<String, TokenError> {
    let mut bitcode = vec![];

    for (line_nb, line) in s.lines().enumerate() {
        match asm_line(line, commands) {
            Ok(bitline) => bitcode.push(bitline),
            Err(e) => {
                eprintln!("/!\\ error at line {}: {}", line_nb + 1, e);
                eprintln!("{}", line);
                return Err(e);
            }
        }
    }

    Ok(bitcode.join("\n"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source file>", args[0]);
        return Err(Box::new(TokenError("No source file provided".to_string())));
    }

    let filename = &args[1];
    let mut file = File::open(format!("{}.s", filename))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let commands = init_commands();
    let bitcode = asm_doc(&contents, &commands)?;

    let mut debug_file = File::create(format!("{}.debug", filename))?;
    debug_file.write_all(bitcode.as_bytes())?;

    let res = bitcode.replace(" ", "");
    let padded_res = format!("{:0<8}", res);
    let bin = u64::from_str_radix(&padded_res, 2)?.to_be_bytes();

    let mut bin_file = File::create(format!("{}.bin", filename))?;
    bin_file.write_all(&bin)?;

    Ok(())
}
