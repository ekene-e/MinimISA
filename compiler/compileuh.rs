use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{self, Write};
use regex::Regex;
use itertools::Itertools;
use std::collections::HashMap;
use crate::enums::{ValueType, LexType};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::util::huffman;
use crate::back_end::MemonicBackEnd;

type VT = ValueType;

// Language specification

lazy_static! {
    static ref POSSIBLE_TRANSITION: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("add", vec!["add2", "add2i", "add3", "add3i"]);
        m.insert("and", vec!["and2", "and2i", "and3", "and3i"]);
        m.insert("sub", vec!["sub2", "sub2i", "sub3", "sub3i"]);
        m.insert("or", vec!["or2", "or2i", "or3", "or3i"]);
        m.insert("xor", vec!["xor3", "xor3i"]);
        m.insert("cmp", vec!["cmp", "cmpi"]);
        m.insert("let", vec!["let", "leti"]);
        m.insert("shift", vec!["shift"]);
        m.insert("readze", vec!["readze"]);
        m.insert("readse", vec!["readse"]);
        m.insert("jump", vec!["jump", "jumpif", "jumpl", "jumpifl"]);
        m.insert("write", vec!["write"]);
        m.insert("call", vec!["call", "calll"]);
        m.insert("setctr", vec!["setctr"]);
        m.insert("getctr", vec!["getctr"]);
        m.insert("push", vec!["push"]);
        m.insert("return", vec!["return"]);
        m.insert("asr", vec!["asr3"]);
        m.insert("pop", vec!["pop"]);
        m.insert("label", vec!["label"]);
        m.insert("const", vec!["const"]);
        m.insert("sleep", vec!["sleep"]);
        m.insert("rand", vec!["rand"]);
        m
    };
}

lazy_static! {
    static ref TYPE_SPECS: HashMap<LexType, Vec<ValueType>> = {
        let mut m = HashMap::new();
        m.insert(LexType::NUMBER, vec![VT::UCONSTANT, VT::SCONSTANT, VT::RADDRESS, VT::AADDRESS, VT::SHIFTVAL, VT::SIZE]);
        m.insert(LexType::DIRECTION, vec![VT::DIRECTION]);
        m.insert(LexType::CONDITION, vec![VT::CONDITION]);
        m.insert(LexType::MEMCOUNTER, vec![VT::MEMCOUNTER]);
        m.insert(LexType::REGISTER, vec![VT::REGISTER]);
        m.insert(LexType::LABEL, vec![VT::LABEL]);
        m.insert(LexType::BINARY, vec![VT::BINARY]);
        m
    };
}

lazy_static! {
    static ref ASR_SPECS: HashMap<&'static str, Vec<ValueType>> = {
        let mut m = HashMap::new();
        m.insert("add2", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("add2i", vec![VT::REGISTER, VT::UCONSTANT]);
        m.insert("add3", vec![VT::REGISTER, VT::REGISTER, VT::REGISTER]);
        m.insert("add3i", vec![VT::REGISTER, VT::REGISTER, VT::UCONSTANT]);

        m.insert("sub2", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("sub2i", vec![VT::REGISTER, VT::UCONSTANT]);
        m.insert("sub3", vec![VT::REGISTER, VT::REGISTER, VT::REGISTER]);
        m.insert("sub3i", vec![VT::REGISTER, VT::REGISTER, VT::UCONSTANT]);

        m.insert("cmp", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("cmpi", vec![VT::REGISTER, VT::SCONSTANT]);

        m.insert("let", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("leti", vec![VT::REGISTER, VT::SCONSTANT]);

        m.insert("shift", vec![VT::DIRECTION, VT::REGISTER, VT::SHIFTVAL]);

        m.insert("readze", vec![VT::MEMCOUNTER, VT::SIZE, VT::REGISTER]);

        m.insert("readse", vec![VT::MEMCOUNTER, VT::SIZE, VT::REGISTER]);

        m.insert("jump", vec![VT::RADDRESS]);
        m.insert("jumpif", vec![VT::CONDITION, VT::RADDRESS]);
        m.insert("jumpl", vec![VT::LABEL]);
        m.insert("jumpifl", vec![VT::CONDITION, VT::LABEL]);

        m.insert("or2", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("or2i", vec![VT::REGISTER, VT::UCONSTANT]);
        m.insert("or3", vec![VT::REGISTER, VT::REGISTER, VT::REGISTER]);
        m.insert("or3i", vec![VT::REGISTER, VT::REGISTER, VT::UCONSTANT]);

        m.insert("and2", vec![VT::REGISTER, VT::REGISTER]);
        m.insert("and2i", vec![VT::REGISTER, VT::UCONSTANT]);
        m.insert("and3", vec![VT::REGISTER, VT::REGISTER, VT::REGISTER]);
        m.insert("and3i", vec![VT::REGISTER, VT::REGISTER, VT::UCONSTANT]);

        m.insert("write", vec![VT::MEMCOUNTER, VT::SIZE, VT::REGISTER]);
        m.insert("call", vec![VT::RADDRESS]);
        m.insert("calll", vec![VT::LABEL]);
        m.insert("setctr", vec![VT::MEMCOUNTER, VT::REGISTER]);
        m.insert("getctr", vec![VT::MEMCOUNTER, VT::REGISTER]);
        m.insert("push", vec![VT::SIZE, VT::REGISTER]);
        m.insert("pop", vec![VT::SIZE, VT::REGISTER]);
        m.insert("return", vec![]);

        m.insert("xor3", vec![VT::REGISTER, VT::REGISTER, VT::REGISTER]);
        m.insert("xor3i", vec![VT::REGISTER, VT::REGISTER, VT::UCONSTANT]);

        m.insert("asr3", vec![VT::REGISTER, VT::REGISTER, VT::SHIFTVAL]);

        m.insert("label", vec![VT::LABEL]);
        m.insert("const", vec![VT::UCONSTANT, VT::BINARY]);
        m.insert("sleep", vec![VT::UCONSTANT]);
        m.insert("rand", vec![VT::REGISTER]);
        m
    };
}

lazy_static! {
    static ref DEFAULT_OPCODE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("add2", "0000");
        m.insert("add2i", "0001");
        m.insert("sub2", "0010");
        m.insert("sub2i", "0011");
        m.insert("cmp", "0100");
        m.insert("cmpi", "0101");
        m.insert("let", "0110");
        m.insert("leti", "0111");
        m.insert("shift", "1000");
        m.insert("readze", "10010");
        m.insert("pop", "1001001");
        m.insert("readse", "10011");
        m.insert("jump", "1010");
        m.insert("jumpif", "1011");
        m.insert("or2", "110000");
        m.insert("or2i", "110001");
        m.insert("and2", "110010");
        m.insert("and2i", "110011");
        m.insert("write", "110100");
        m.insert("call", "110101");
        m.insert("setctr", "110110");
        m.insert("getctr", "110111");
        m.insert("push", "1110000");
        m.insert("return", "1110001");
        m.insert("add3", "1110010");
        m.insert("add3i", "1110011");
        m.insert("sub3", "1110100");
        m.insert("sub3i", "1110101");
        m.insert("and3", "1110110");
        m.insert("and3i", "1110111");
        m.insert("or3", "1111000");
        m.insert("or3i", "1111001");
        m.insert("xor3", "1111010");
        m.insert("xor3i", "1111011");
        m.insert("asr3", "1111100");
        m.insert("sleep", "1111101");
        m.insert("rand", "1111110");
        m.insert("reserved3", "1111111");
        m
    };
}

fn count_operations(c: &mut HashMap<String, usize>, it: impl Iterator<Item = Line>) {
    for line in it {
        let entry = c.entry(line.funcname.clone()).or_insert(0);
        *entry += 1;
    }
}

pub fn compile_asm(s: &str, generate_tree: bool, directory: &str, filename: &str) -> MemonicBackEnd {
    // Replace transitions in the pre-assembly code
    let mut s = s.to_string();
    for (new, olds) in POSSIBLE_TRANSITION.iter() {
        let sorted_olds: Vec<&str> = olds.iter().sorted_by_key(|s| s.len()).map(|s| *s).collect();
        let pattern = format!("({})", sorted_olds.join("|"));
        let re = Regex::new(&pattern).unwrap();
        s = re.replace_all(&s, *new).into();
    }

    // Tokenize the pre-asm
    let lexer = Lexer::new(&POSSIBLE_TRANSITION);
    let gen_lex = lexer.lex(&s, filename, directory);

    // Parse to convert into assembly
    let parser = Parser::new(&gen_lex, &POSSIBLE_TRANSITION, &ASR_SPECS, &TYPE_SPECS);
    let mut hufftree: HashMap<String, String>;

    if generate_tree {
        // Duplicate the iterator for huffman tree
        let (par1, par2) = gen_lex.tee();

        let mut c = HashMap::new();
        for key in DEFAULT_OPCODE.keys() {
            if !key.starts_with("reserved") {
                c.insert(key.to_string(), 0);
            }
        }

        count_operations(&mut c, par1);
        hufftree = huffman(&c).into_iter().collect();

        let mut file = File::create("opcode.txt").unwrap();
        for (opcode, memonic) in hufftree.iter() {
            writeln!(file, "{} {}", memonic, opcode).unwrap();
        }
    } else {
        hufftree = DEFAULT_OPCODE.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    }

    let out = MemonicBackEnd::new(hufftree, parser.run());
    out
}
