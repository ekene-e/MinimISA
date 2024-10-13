//---
// emu:memory - emulate a random-access bit-addressable memory
//
// This module provides routines for manipulating the bit-addressable
// memory used by the fictional CPU. 
//---

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

// Default memory geometry
const MEMORY_DEFAULT_TEXT: u64 = 32 << 10;
const MEMORY_DEFAULT_STACK: u64 = 16 << 10;
const MEMORY_DEFAULT_DATA: u64 = 16 << 10;
const MEMORY_DEFAULT_VRAM: u64 = 327680;

#[derive(Debug)]
pub struct Memory {
    memsize: u64,   // Total memory size
    text: u64,      // Size of text segment
    stack: u64,     // Bottom stack address
    data: u64,      // Address of the data segment
    vram: u64,      // Address of the VRAM segment
    mem: Vec<u64>,  // Actual chunk of data
}

impl Memory {
    pub fn new(text: u64, stack: u64, data: u64, vram: u64) -> Memory {
        let memsize = text + stack + data + vram;
        let mem = vec![0u64; (memsize as usize) / 64]; 

        Memory {
            memsize,
            text: if text != 0 { text } else { MEMORY_DEFAULT_TEXT },
            stack: if stack != 0 { stack } else { MEMORY_DEFAULT_STACK },
            data: if data != 0 { data } else { MEMORY_DEFAULT_DATA },
            vram: if vram != 0 { vram } else { MEMORY_DEFAULT_VRAM },
            mem,
        }
    }

    // Load a program from a file into memory
    pub fn load_program(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        if (buffer.len() * 8) > self.text as usize {
            panic!("Program does not fit in the code/stack segment");
        }

        self.mem[..buffer.len()].copy_from_slice(&buffer.iter().map(|&b| b as u64).collect::<Vec<u64>>()[..]);

        Ok(())
    }

    // Load a text program into memory
    pub fn load_text(&mut self, filename: &str) -> io::Result<()> {
        self.load_program(filename)
    }

    // Load an additional file into memory at the given address
    pub fn load_file(&mut self, address: u64, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if (buffer.len() * 8) > (self.memsize - address) as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "File does not fit in memory"));
        }

        let offset = (address / 64) as usize;
        self.mem[offset..offset + buffer.len()].copy_from_slice(&buffer.iter().map(|&b| b as u64).collect::<Vec<u64>>()[..]);

        Ok(())
    }

    // Free the memory object (automatically done in Rust)
    // Rust will handle memory cleanup, so no need for an explicit destroy function

    // Read n bits from an address (up to 64)
    pub fn read(&self, address: u64, n: usize) -> u64 {
        assert!(n <= 64);
        let bit_pos = address % 64;
        let word_index = (address / 64) as usize;

        let mut result = self.mem[word_index] >> (64 - n - bit_pos);

        if bit_pos + n > 64 && word_index + 1 < self.mem.len() {
            result |= self.mem[word_index + 1] << (64 - bit_pos);
        }

        result
    }

    // Write n bits to an address (up to 64)
    pub fn write(&mut self, address: u64, value: u64, n: usize) {
        assert!(n <= 64);
        let bit_pos = address % 64;
        let word_index = (address / 64) as usize;

        let mask = (1u64 << n) - 1;
        self.mem[word_index] &= !(mask << (64 - n - bit_pos));
        self.mem[word_index] |= (value & mask) << (64 - n - bit_pos);

        if bit_pos + n > 64 && word_index + 1 < self.mem.len() {
            self.mem[word_index + 1] &= !(mask >> (64 - bit_pos));
            self.mem[word_index + 1] |= (value & mask) >> (64 - bit_pos);
        }
    }
}
