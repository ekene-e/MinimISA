use std::sync::{Arc, Mutex};
use std::fmt;
use crate::memory::Memory;
use crate::disasm::disasm_opcode;

/// Some names for the memory pointers
pub const PC: usize = 0;
pub const SP: usize = 1;
pub const A0: usize = 2;
pub const A1: usize = 3;

/// CPU struct holding registers, pointers, flags, and associated memory
pub struct CPU {
    pub mem: Arc<Mutex<Memory>>,  // Memory associated with the CPU (shared)

    pub r: [u64; 8],  // General purpose registers r0..r7

    // Flags
    pub z: bool,   // Zero: x == y
    pub n: bool,   // Negative: (int) x < (int) y
    pub c: bool,   // Carry: (uint) x < (uint) y
    pub v: bool,   // Overflow: integer overflow

    // Debugger flags
    pub h: bool,    // Halt: detects loops of one instruction
    pub m: bool,    // Memory: indicates changes to memory
    pub t: bool,    // Counter: signals counter changes
    pub s: bool,    // Stop: indicates stop orders from user
    pub sleep: bool,  // Current sleeping state

    pub ptr: [u64; 4],  // Pointers: PC, SP, A0, A1

    pub instruction_count: [usize; DISASM_INS_COUNT],  
}

impl CPU {
    pub fn new(mem: Arc<Mutex<Memory>>) -> CPU {
        CPU {
            mem,
            r: [0; 8],
            z: false,
            n: false,
            c: false,
            v: false,
            h: false,
            m: false,
            t: false,
            s: false,
            sleep: false,
            ptr: [0; 4],
            instruction_count: [0; DISASM_INS_COUNT],
        }
    }

    pub fn destroy(self) {
        ;
    }

    pub fn dump(&self) -> String {
        format!(
            "CPU State:\nRegisters: {:?}\nPC: {:#x}\nSP: {:#x}\nFlags: Z:{} N:{} C:{} V:{}\n",
            self.r, self.ptr[PC], self.ptr[SP], self.z, self.n, self.c, self.v
        )
    }

    pub fn execute(&mut self) {
        let pc = self.ptr[PC];
        let mut memory = self.mem.lock().unwrap();

        let (opcode, format) = disasm_opcode(&memory, &mut self.ptr[PC]);

        if (opcode as usize) < DISASM_INS_COUNT {
            self.instruction_count[opcode as usize] += 1;
        }

        match opcode {
            0x01 => {
                let reg = memory.read_u64(self.ptr[PC]);  
                let addr = memory.read_u64(self.ptr[PC] + 8);  
                self.r[reg as usize] = memory.read_u64(addr);  
                self.ptr[PC] += 16;  
            }
            0x02 => {
                let reg1 = memory.read_bits(self.ptr[PC], 3);
                let reg2 = memory.read_bits(self.ptr[PC] + 3, 3);
                self.r[reg1 as usize] = self.r[reg1 as usize].wrapping_add(self.r[reg2 as usize]);
                self.ptr[PC] += 6;  
            }
            _ => {
                self.h = true;  
            }
        }

        self.update_flags();
    }

    fn update_flags(&mut self) {
        self.z = self.r[0] == 0;  
        self.n = (self.r[0] as i64) < 0;  
    }

    pub fn counts(&self) -> &[usize; DISASM_INS_COUNT] {
        &self.instruction_count
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.dump())
    }
}
