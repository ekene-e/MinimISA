use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process;

static mut LINE: usize = 0;
static mut CURRENT_ADDR: u64 = 0;
static mut LABELS: Option<HashMap<String, u64>> = None;

fn error(e: &str) -> ! {
    unsafe {
        panic!("Error at line {}: {}", LINE, e);
    }
}

fn asm_reg(s: &str) -> String {
    if !s.starts_with('r') {
        error("Invalid register");
    }
    let val: u32 = s[1..].parse().expect("Failed to parse register number");
    if val > 7 {
        error("Invalid register number");
    }
    format!("{:03b} ", val) // 3 bits
}

fn asm_addr_signed(s: &str) -> String {
    let val: i64 = s.parse().expect("Failed to parse address");
    if (-128..=127).contains(&val) {
        format!("0 {:08b} ", val)
    } else if (-32768..=32767).contains(&val) {
        format!("10 {:016b} ", val)
    } else if (-2i64.pow(31)..=2i64.pow(31) - 1).contains(&val) {
        format!("110 {:032b} ", val)
    } else {
        format!("111 {:064b} ", val)
    }
}

fn asm_const_unsigned(s: &str) -> String {
    let val: u64 = if s.starts_with("0x") {
        u64::from_str_radix(&s[2..], 16).expect("Failed to parse hexadecimal constant")
    } else {
        s.parse().expect("Failed to parse constant")
    };

    if val <= 1 {
        format!("0 {}", val)
    } else if val < 256 {
        format!("10 {:08b} ", val)
    } else if val < 2u64.pow(32) {
        format!("110 {:032b} ", val)
    } else {
        format!("111 {:064b} ", val)
    }
}

fn asm_condition(cond: &str) -> String {
    let condlist = HashMap::from([
        ("eq", "000"), ("z", "000"), ("neq", "001"), ("nz", "001"),
        ("sgt", "010"), ("slt", "011"), ("gt", "100"), ("ge", "101"),
        ("nc", "101"), ("lt", "110"), ("c", "110"), ("le", "111")
    ]);

    condlist.get(cond).unwrap_or_else(|| error("Invalid condition")).to_string()
}

fn asm_counter(ctr: &str) -> String {
    let codelist = HashMap::from([
        ("pc", "00"), ("sp", "01"), ("a0", "10"), ("a1", "11"),
        ("0", "00"), ("1", "01"), ("2", "10"), ("3", "11")
    ]);

    codelist.get(ctr).unwrap_or_else(|| error("Invalid counter")).to_string()
}

fn asm_size(s: &str) -> String {
    let codelist = HashMap::from([
        ("1", "00"), ("4", "01"), ("8", "100"), ("16", "101"),
        ("32", "110"), ("64", "111")
    ]);

    codelist.get(s).unwrap_or_else(|| error("Invalid size")).to_string()
}

fn asm_pass(iteration: u32, s_file: &str) -> Vec<String> {
    let mut code = vec![];
    let mut current_address = 0;

    println!("\nPASS {}", iteration);

    let file = File::open(s_file).expect("Cannot open source file");
    let reader = BufReader::new(file);

    for source_line in reader.lines() {
        let source_line = source_line.unwrap();
        println!("processing {}", source_line.trim());

        let mut instruction_encoding = String::new();
        let line_content = source_line.split(';').next().unwrap_or("").to_string();
        let tokens: Vec<&str> = line_content.split_whitespace().collect();

        if !tokens.is_empty() {
            if let Some(label) = tokens.get(0) {
                if label.ends_with(':') {
                    unsafe {
                        LABELS.get_or_insert(HashMap::new()).insert(label.trim_end_matches(':'), current_address);
                    }
                }
            }
        }

        if !tokens.is_empty() {
            let opcode = tokens[0];
            let token_count = tokens.len();
            match opcode {
                "add2" if token_count == 3 => {
                    instruction_encoding = format!("0000 {} {}", asm_reg(tokens[1]), asm_reg(tokens[2]));
                }
                "add2i" if token_count == 3 => {
                    instruction_encoding = format!("0001 {} {}", asm_reg(tokens[1]), asm_const_unsigned(tokens[2]));
                }
                "jump" if token_count == 2 => {
                    instruction_encoding = format!("1010 {}", asm_addr_signed(tokens[1]));
                }
                _ => {
                    error("Unknown opcode or incorrect token count");
                }
            }

            if !instruction_encoding.is_empty() {
                let compact_encoding: String = instruction_encoding.split_whitespace().collect();
                let instr_size = compact_encoding.len();
                println!(
                    "... @{} {:016b} : {}",
                    current_address, current_address, compact_encoding
                );
                println!("{} size={}", instruction_encoding, instr_size);
                current_address += instr_size as u64;
            }
        }

        unsafe {
            LINE += 1;
        }
        code.push(instruction_encoding);
    }

    code
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: asm <source file>");
        process::exit(1);
    }

    let filename = &args[1];
    let basefilename = Path::new(filename).file_stem().unwrap().to_str().unwrap();
    let obj_file = format!("{}.obj", basefilename);

    let code = asm_pass(1, filename);

    let mut outfile = File::create(obj_file).expect("Cannot create output file");
    for instr in &code {
        writeln!(outfile, "{}", instr).expect("Failed to write to file");
    }

    println!("Average instruction size: {}", unsafe { CURRENT_ADDR } as f64 / code.len() as f64);
}
