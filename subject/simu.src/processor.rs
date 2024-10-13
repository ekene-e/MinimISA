extern crate std;

use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

pub const WORDSIZE: usize = 32;
pub type UWord = u32;
pub type SWord = i32;
pub type DoubleWord = u64;

#[derive(Debug)]
pub struct Memory {
    pub m: Vec<u64>,
    pub counter: [UWord; 4],
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            m: vec![0; size],
            counter: [0; 4],
        }
    }

    pub fn read_bit(&self, _pc: usize) -> u64 {
        0
    }

    pub fn set_counter(&mut self, idx: usize, value: UWord) {
        self.counter[idx] = value;
    }
}

pub struct Processor {
    m: Arc<Mutex<Memory>>,
    pc: UWord,
    sp: UWord,
    a1: UWord,
    a2: UWord,
    r: [UWord; 8],
    zflag: bool,
    cflag: bool,
    nflag: bool,
}

impl Processor {
    pub fn new(m: Arc<Mutex<Memory>>) -> Self {
        Processor {
            m,
            pc: 0,
            sp: 0,
            a1: 0,
            a2: 0,
            r: [0; 8],
            zflag: false,
            cflag: false,
            nflag: false,
        }
    }

    pub fn von_neumann_step(&mut self, debug: bool) {
        let mut opcode = 0;
        let mut regnum1 = 0;
        let mut regnum2 = 0;
        let mut shiftval = 0;
        let mut condcode = 0;
        let mut counter = 0;
        let mut size = 0;
        let mut offset: UWord = 0;
        let mut constop: u64 = 0;
        let mut dir = 0;
        let mut uop1: UWord;
        let mut uop2: UWord;
        let mut ur: UWord = 0;
        let mut fullr: DoubleWord;
        let mut manage_flags = false;
        let instr_pc = self.pc;

        // Read 4 bits for opcode
        self.read_bit_from_pc(&mut opcode);
        self.read_bit_from_pc(&mut opcode);
        self.read_bit_from_pc(&mut opcode);
        self.read_bit_from_pc(&mut opcode);

        match opcode {
            0x0 => { // add2
                self.read_reg_from_pc(&mut regnum1);
                self.read_reg_from_pc(&mut regnum2);
                uop1 = self.r[regnum1 as usize];
                uop2 = self.r[regnum2 as usize];
                fullr = uop1 as DoubleWord + uop2 as DoubleWord; // for flags
                ur = uop1 + uop2;
                self.r[regnum1 as usize] = ur;
                manage_flags = true;
            }
            0x1 => { // add2i
                self.read_reg_from_pc(&mut regnum1);
                self.read_const_from_pc(&mut constop);
                uop1 = self.r[regnum1 as usize];
                uop2 = constop as UWord;
                fullr = uop1 as DoubleWord + uop2 as DoubleWord; // for flags
                ur = uop1 + uop2;
                self.r[regnum1 as usize] = ur;
                manage_flags = true;
            }
            0xa => { // jump
                self.read_addr_from_pc(&mut offset);
                self.pc += offset;
                let mut mem = self.m.lock().unwrap();
                mem.set_counter(0, self.pc);
                manage_flags = false;
            }
            0x8 => { // shift
                self.read_bit_from_pc(&mut dir);
                self.read_reg_from_pc(&mut regnum1);
                self.read_shiftval_from_pc(&mut shiftval);
                uop1 = self.r[regnum1 as usize];
                if dir == 1 {
                    ur = uop1 >> shiftval;
                    self.cflag = ((uop1 >> (shiftval - 1)) & 1) == 1;
                } else {
                    self.cflag = ((uop1 << (shiftval - 1)) & (1 << (WORDSIZE - 1))) != 0;
                    ur = uop1 << shiftval;
                }
                self.r[regnum1 as usize] = ur;
                self.zflag = ur == 0;
                manage_flags = false;
            }
            0xc | 0xd => {
                self.read_bit_from_pc(&mut opcode);
                self.read_bit_from_pc(&mut opcode);
                if opcode == 0b110100 {
                    // Handle write operation
                    self.handle_write_operation();
                }
            }
            0xe | 0xf => {
                self.read_bit_from_pc(&mut opcode);
                self.read_bit_from_pc(&mut opcode);
                self.read_bit_from_pc(&mut opcode);
                // Handle additional cases if needed
            }
            _ => {}
        }

        // Flag management
        if manage_flags {
            self.zflag = ur == 0;
            self.cflag = fullr > (1u64 << WORDSIZE);
            self.nflag = (ur as SWord) < 0;
        }

        if debug {
            self.debug_output(opcode, instr_pc);
        }
    }

    fn handle_write_operation(&mut self) {
        let mut regnum = 0;
        let mut size = 0;
        self.read_reg_from_pc(&mut regnum);
        self.read_size_from_pc(&mut size);
        let value = self.r[regnum as usize];
        // Handle memory writing operation using size and value
    }

    fn debug_output(&self, opcode: i32, instr_pc: UWord) {
        let mem = self.m.lock().unwrap();
        print!(
            "after instr: {} at pc={:08x} (newpc={:08x} mpc={:08x} msp={:08x} ma0={:08x} ma1={:08x}) ",
            opcode, instr_pc, self.pc, mem.counter[0], mem.counter[1], mem.counter[2], mem.counter[3]
        );
        print!("zcn = {}{}{}", self.zflag as u8, self.cflag as u8, self.nflag as u8);
        for i in 0..8 {
            print!(" r{}={:08x}", i, self.r[i]);
        }
        println!();
    }

    // Helper methods

    fn read_bit_from_pc(&mut self, var: &mut i32) {
        let bit = self.m.lock().unwrap().read_bit(self.pc as usize);
        *var = (*var << 1) + bit as i32;
        self.pc += 1;
    }

    fn read_reg_from_pc(&mut self, var: &mut i32) {
        *var = 0;
        self.read_bit_from_pc(var);
        self.read_bit_from_pc(var);
        self.read_bit_from_pc(var);
    }

    fn read_const_from_pc(&mut self, var: &mut u64) {
        *var = 0;
        let mut header = 0;
        let mut size = 0;
        self.read_bit_from_pc(&mut header);
        if header == 0 {
            size = 1;
        } else {
            self.read_bit_from_pc(&mut header);
            if header == 2 {
                size = 8;
            } else {
                self.read_bit_from_pc(&mut header);
                size = if header == 6 { 32 } else { 64 };
            }
        }
        for _ in 0..size {
            *var = (*var << 1) + self.m.lock().unwrap().read_bit(self.pc as usize) as u64;
            self.pc += 1;
        }
    }

    fn read_addr_from_pc(&mut self, var: &mut UWord) {
        let mut header = 0;
        let mut size = 0;
        *var = 0;
        self.read_bit_from_pc(&mut header);
        size = if header == 0 { 8 } else {
            self.read_bit_from_pc(&mut header);
            if header == 2 { 16 } else {
                self.read_bit_from_pc(&mut header);
                if header == 6 { 32 } else { 64 }
            }
        };
        for _ in 0..size {
            *var = (*var << 1) + self.m.lock().unwrap().read_bit(self.pc as usize) as UWord;
            self.pc += 1;
        }
        let sign = (*var >> (size - 1)) & 1;
        for i in size..WORDSIZE {
            *var += sign << i;
        }
    }

    fn read_shiftval_from_pc(&mut self, var: &mut i32) {
        *var = 0;
        self.read_bit_from_pc(var);
        for _ in 0..6 {
            self.read_bit_from_pc(var);
        }
    }

    fn read_cond_from_pc(&mut self, var: &mut i32) {
        *var = 0;
        self.read_bit_from_pc(var);
        self.read_bit_from_pc(var);
        self.read_bit_from_pc(var);
    }

    fn cond_true(&self, cond: i32) -> bool {
        match cond {
            0 => self.zflag,
            1 => !self.zflag,
            _ => panic!("Unexpected condition code"),
        }
    }

    fn read_counter_from_pc(&mut self, var: &mut i32) {
        *var = 0;
        self.read_bit_from_pc(var);
        self.read_bit_from_pc(var);
    }

    fn read_size_from_pc(&mut self, size: &mut i32) {
        *size = 0;
        self.read_bit_from_pc(size);
        self.read_bit_from_pc(size);
    }
}
