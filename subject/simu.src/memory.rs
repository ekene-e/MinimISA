use std::fs::File;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::fmt;

pub const MEMSIZE: usize = 1 << 24; 
pub const PC: usize = 0;
pub const SP: usize = 1;
pub const A0: usize = 2;
pub const A1: usize = 3;

pub type UWord = u32;

pub struct Memory {
    pub counter: [usize; 4],  
    pub m: [u64; MEMSIZE / 64], 
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            counter: [0; 4], 
            m: [0; MEMSIZE / 64], 
        }
    }

    pub fn read_bit(&mut self, ctr: usize) -> u64 {
        let word_addr = self.counter[ctr] >> 6;
        let word = self.m[word_addr]; 
        let shift = self.counter[ctr] & 63; 
        let bit = (word >> shift) & 1; 
        self.counter[ctr] += 1;
        bit
    }

    pub fn write_bit(&mut self, ctr: usize, bit: u64) {
        if bit != 0 && bit != 1 {
            panic!("Expecting a bit (0 or 1)");
        }
        let word_addr = self.counter[ctr] >> 6;
        let mut word = self.m[word_addr]; 
        let shift = self.counter[ctr] & 63; 
        let bit64 = bit << shift;
        let mask = !(1u64 << shift);
        word = (word & mask) | bit64;
        self.m[word_addr] = word;
        self.counter[ctr] += 1;
    }

    pub fn set_counter(&mut self, ctr: usize, val: UWord) {
        self.counter[ctr] = val as usize;
    }

    pub fn fill_with_obj_file(&mut self, filename: &str) {
        println!("Loading...");
        self.counter[0] = 0; 
        let file = File::open(filename).expect("Failed to open file.");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            for ch in line.unwrap().chars() {
                match ch {
                    '0' => {
                        print!("{}", ch);
                        self.write_bit(0, 0);
                    }
                    '1' => {
                        print!("{}", ch);
                        self.write_bit(0, 1);
                    }
                    _ => continue, 
                }
            }
        }
        println!(" Done.");
        self.counter[0] = 0; 
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory {{ counter: {:?}, m: [memory... of size {}] }}", self.counter, MEMSIZE)
    }
}
