use crate::memory::Memory;

/// Number of different instructions (assuming 37 opcodes)
pub const DISASM_INS_COUNT: usize = 37;

#[derive(Debug, Clone, Copy)]
pub enum ArgType {
    None,       // No argument
    Register,   // Register: r0..r7 on 3 bits
    Direction,  // Direction: left/right on 1 bit
    Condition,  // Condition: various on 3 bits
    Address,    // Address: on 9, 18, 35 or 67 bits
    LConst,     // Constants: on 2, 18, 35, or 67 bits
    AConst,     // Arithmetic (signed) constants
    Shift,      // Shifts: 1 bit or 7 bits
    Size,       // Size: 2 or 3 bits
    Pointer,    // Pointer: PC, SP, A0, or A1 on 2 bits
}

#[derive(Debug, Clone, Copy)]
pub enum Category {
    Arithmetic,
    Test,
    Let,
    Jump,
    Memory,
    Control,
}

pub struct DisasmFormat {
    pub arg1: ArgType,
    pub arg2: ArgType,
    pub arg3: ArgType,
    pub category: Category,
    pub mnemonic: &'static str,
}

/// Read an instruction code (opcode) from memory and return the format
pub fn disasm_opcode(memory: &Memory, ptr: &mut u64) -> (u32, Option<DisasmFormat>) {
    let opcode = memory.read_u32(*ptr);
    *ptr += 4;  // Advance the pointer

    let format = disasm_format(opcode);
    (opcode, format)
}

/// Get the format for a given instruction (based on opcode)
pub fn disasm_format(opcode: u32) -> Option<DisasmFormat> {
    match opcode {
        0x00 => Some(DisasmFormat {
            arg1: ArgType::None,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Control,
            mnemonic: "NOP",
        }),
        0x01 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Address,
            arg3: ArgType::None,
            category: Category::Memory,
            mnemonic: "LOAD",
        }),
        0x02 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::LConst,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "ADD",
        }),
        0x03 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "SUB",
        }),
        0x04 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "MUL",
        }),
        0x05 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "DIV",
        }),
        0x06 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::AConst,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "MOD",
        }),
        0x07 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "AND",
        }),
        0x08 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "OR",
        }),
        0x09 => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "XOR",
        }),
        0x0A => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::Shift,
            category: Category::Arithmetic,
            mnemonic: "SHL",
        }),
        0x0B => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Register,
            arg3: ArgType::Shift,
            category: Category::Arithmetic,
            mnemonic: "SHR",
        }),
        0x0C => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Arithmetic,
            mnemonic: "NEG",
        }),
        0x0D => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Condition,
            arg3: ArgType::None,
            category: Category::Test,
            mnemonic: "CMP",
        }),
        0x0E => Some(DisasmFormat {
            arg1: ArgType::Register,
            arg2: ArgType::Pointer,
            arg3: ArgType::None,
            category: Category::Memory,
            mnemonic: "STORE",
        }),
        0x0F => Some(DisasmFormat {
            arg1: ArgType::None,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Control,
            mnemonic: "HALT",
        }),
        0x10 => Some(DisasmFormat {
            arg1: ArgType::Address,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Jump,
            mnemonic: "JMP",
        }),
        0x11 => Some(DisasmFormat {
            arg1: ArgType::Condition,
            arg2: ArgType::Address,
            arg3: ArgType::None,
            category: Category::Jump,
            mnemonic: "JZ",
        }),
        0x12 => Some(DisasmFormat {
            arg1: ArgType::Condition,
            arg2: ArgType::Address,
            arg3: ArgType::None,
            category: Category::Jump,
            mnemonic: "JNZ",
        }),
        0x13 => Some(DisasmFormat {
            arg1: ArgType::None,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Control,
            mnemonic: "RET",
        }),
        0x24 => Some(DisasmFormat {
            arg1: ArgType::None,
            arg2: ArgType::None,
            arg3: ArgType::None,
            category: Category::Control,
            mnemonic: "END",
        }),
        _ => None,  // Return None for unknown opcode
    }
}

/// Read a register number (3 bits)
pub fn disasm_reg(memory: &Memory, ptr: &mut u64) -> u32 {
    let reg = memory.read_bits(*ptr, 3);
    *ptr += 3;
    reg
}

/// Read a shift direction bit
pub fn disasm_dir(memory: &Memory, ptr: &mut u64) -> u32 {
    let dir = memory.read_bits(*ptr, 1);
    *ptr += 1;
    dir
}

/// Read a jump condition type (3 bits)
pub fn disasm_cond(memory: &Memory, ptr: &mut u64) -> u32 {
    let cond = memory.read_bits(*ptr, 3);
    *ptr += 3;
    cond
}

/// Read a relative address (optional pointer to size)
pub fn disasm_addr(memory: &Memory, ptr: &mut u64, size: Option<&mut u32>) -> i64 {
    let addr_size = memory.read_bits(*ptr, 9);  // Example: read 9 bits for address
    if let Some(size_ptr) = size {
        *size_ptr = addr_size;
    }
    *ptr += 9;
    memory.read_signed(*ptr, addr_size as usize)
}

/// Read a zero-extended constant
pub fn disasm_lconst(memory: &Memory, ptr: &mut u64, size: Option<&mut u32>) -> u64 {
    let const_size = memory.read_bits(*ptr, 9);  // Example: read 9 bits for constant size
    if let Some(size_ptr) = size {
        *size_ptr = const_size;
    }
    *ptr += 9;
    memory.read_unsigned(*ptr, const_size as usize)
}

/// Read a sign-extended constant
pub fn disasm_aconst(memory: &Memory, ptr: &mut u64, size: Option<&mut u32>) -> i64 {
    let const_size = memory.read_bits(*ptr, 9);  // Example: read 9 bits for constant size
    if let Some(size_ptr) = size {
        *size_ptr = const_size;
    }
    *ptr += 9;
    memory.read_signed(*ptr, const_size as usize)
}

/// Read a shift constant (6 bits)
pub fn disasm_shift(memory: &Memory, ptr: &mut u64) -> u32 {
    let shift = memory.read_bits(*ptr, 6);
    *ptr += 6;
    shift
}

/// Read a memory operation size (e.g., 1, 4, 8, 16, 32, or 64 bits)
pub fn disasm_size(memory: &Memory, ptr: &mut u64) -> u32 {
    let size = memory.read_bits(*ptr, 3);
    *ptr += 3;
    size
}

/// Read a pointer id (2 bits)
pub fn disasm_pointer(memory: &Memory, ptr: &mut u64) -> u32 {
    let pointer = memory.read_bits(*ptr, 2);
    *ptr += 2;
    pointer
}