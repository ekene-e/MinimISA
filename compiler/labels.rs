use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use crate::back_end::{CleartextBitcodeBackEnd, BinaryBitcodeBackEnd};
use crate::enums::Line;
use crate::errors::{BackEndError, ImpossibleError};
use crate::util::Queue;

pub struct LabelsClearTextBackEnd {
    base: CleartextBitcodeBackEnd,
    bit_cost: HashMap<u64, u64>,
    bit_prefix: HashMap<u64, String>,
}

impl LabelsClearTextBackEnd {
    pub fn new(base: CleartextBitcodeBackEnd) -> Self {
        let mut bit_cost = HashMap::new();
        bit_cost.insert(8, 9);
        bit_cost.insert(16, 18);
        bit_cost.insert(32, 35);
        bit_cost.insert(64, 67);

        let mut bit_prefix = HashMap::new();
        bit_prefix.insert(8, "0".to_string());
        bit_prefix.insert(16, "10".to_string());
        bit_prefix.insert(32, "110".to_string());
        bit_prefix.insert(64, "111".to_string());

        LabelsClearTextBackEnd { base, bit_cost, bit_prefix }
    }

    pub fn get_fullcode(&mut self) -> Vec<(usize, String)> {
        let mut fullcode = vec![(0, "".to_string())];
        let mut acc = String::new();

        for line in &self.base.line_gene {
            if !["jumpl", "jumpifl", "calll", "label"].contains(&line.funcname.as_str()) {
                self.base.handle_line(line.clone()).unwrap();

                while !self.base.out_queue.is_empty() {
                    acc.push_str(&(self.base.out_queue.pop().unwrap() + "\n"));
                }
            } else {
                fullcode.push((acc.split_whitespace().collect::<String>().len(), acc.clone()));

                let bitcode = if line.funcname == "label" {
                    "".to_string()
                } else {
                    self.base.huffman_tree[&line.funcname[..line.funcname.len()-1]].clone()
                };

                if line.funcname == "jumpl" || line.funcname == "calll" {
                    fullcode.push((bitcode.len(), line.clone()));
                } else if line.funcname == "jumpifl" {
                    fullcode.push((bitcode.len() + 3, line.clone()));
                }

                acc.clear();
            }
        }

        fullcode.push((acc.len(), acc));
        fullcode
    }

    pub fn get_label_pos(&self, fullcode: &[(usize, String)]) -> HashMap<u64, usize> {
        let mut label_dict = HashMap::new();

        for (i, (_, x)) in fullcode.iter().enumerate() {
            if let Some(line) = self.base.line_gene.iter().find(|line| line.funcname == "label") {
                let label = line.typed_args[0].raw_value;
                label_dict.insert(label, i);
            }
        }

        label_dict
    }

    pub fn count_bytes(&self, fullcode: &[(usize, String)], addr_values: &HashMap<usize, (u64, i64)>, i: usize, j: usize) -> i64 {
        let mut s = 0;
        if j < i {
            for k in (j + 1)..i {
                s += fullcode[k].0 as i64;
                if let Some(&(nb_bit, _)) = addr_values.get(&k) {
                    s += *self.bit_cost.get(&nb_bit).unwrap() as i64;
                }
            }
            s
        } else {
            for k in i..=j {
                s += fullcode[k].0 as i64;
                if let Some(&(nb_bit, _)) = addr_values.get(&k) {
                    s += *self.bit_cost.get(&nb_bit).unwrap() as i64;
                }
            }
            -s
        }
    }

    pub fn packets(&mut self) -> Vec<String> {
        let fullcode = self.get_fullcode();
        let label_dict = self.get_label_pos(&fullcode);

        let mut addr_values: HashMap<usize, (u64, i64)> = HashMap::new();

        for (j, (_, x)) in fullcode.iter().enumerate() {
            if let Some(line) = self.base.line_gene.get(j) {
                if ["jumpl", "jumpifl", "calll"].contains(&line.funcname.as_str()) {
                    addr_values.insert(j, (8, 0));
                }
            }
        }

        loop {
            let mut change = false;

            for (j, (_, x)) in fullcode.iter().enumerate() {
                if let Some(line) = self.base.line_gene.get(j) {
                    if line.funcname == "jumpl" || line.funcname == "jumpifl" {
                        let label = if line.funcname == "jumpl" {
                            line.typed_args[0].raw_value
                        } else {
                            line.typed_args[1].raw_value
                        };

                        if !label_dict.contains_key(&label) {
                            panic!("Undefined label '{}'", label);
                        }

                        let i = label_dict[&label];
                        let (nb_bit, old_s) = addr_values[&j];
                        let s = self.count_bytes(&fullcode, &addr_values, i, j);

                        if s < -(1 << (nb_bit - 1)) || s >= (1 << (nb_bit - 1)) {
                            if nb_bit == 64 {
                                panic!("Jump too long");
                            }
                            addr_values.insert(j, (nb_bit * 2, s));
                            change = true;
                            break;
                        } else {
                            addr_values.insert(j, (nb_bit, s));
                        }
                    } else if line.funcname == "calll" {
                        let label = line.typed_args[0].raw_value;

                        if !label_dict.contains_key(&label) {
                            panic!("Undefined label '{}'", label);
                        }

                        let i = label_dict[&label];
                        let (nb_bit, old_s) = addr_values[&j];
                        let s = self.count_bytes(&fullcode, &addr_values, i, 0);

                        if s < -(1 << (nb_bit - 1)) || s >= (1 << (nb_bit - 1)) {
                            if nb_bit == 64 {
                                panic!("Address too big");
                            }
                            addr_values.insert(j, (nb_bit * 2, s));
                            change = true;
                            break;
                        } else {
                            addr_values.insert(j, (nb_bit, s));
                        }
                    }
                }
            }

            if !change {
                break;
            }
        }

        let mut endcode = vec![];

        for (i, (_, x)) in fullcode.iter().enumerate() {
            if x.is_empty() {
                continue;
            }

            let line = self.base.line_gene.get(i).unwrap();

            if ["jumpl", "jumpifl", "calll"].contains(&line.funcname.as_str()) {
                let mut bitcode = " ".to_string() + &self.base.huffman_tree[&line.funcname[..line.funcname.len() - 1]];

                if line.funcname == "jumpifl" {
                    let cond = line.typed_args[0].raw_value;
                    bitcode.push_str(&format!(" {}", self.base.bin_condition(cond)));
                }

                let (k, n) = addr_values[&i];
                bitcode.push_str(&format!(" {}{}", self.bit_prefix[&k], self.base.binary_repr(n, k, true)));
                endcode.push(bitcode);
            } else {
                endcode.push(x.clone());
            }
        }

        endcode
    }
}

pub struct LabelsBinaryBackEnd {
    base: LabelsClearTextBackEnd,
    write_mode: String,
}

impl LabelsBinaryBackEnd {
    pub fn new(base: LabelsClearTextBackEnd) -> Self {
        LabelsBinaryBackEnd {
            base,
            write_mode: "wb".to_string(),
        }
    }

    pub fn to_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let bitcode = self.base.packets().join("");
        let text_size = bitcode.len();
        let padded_bitcode = bitcode + &"0".repeat((8 - (bitcode.len() % 8)) % 8);
        let q = padded_bitcode.len() / 8;

        let mut file = File::create(filename)?;

        file.write_all(&text_size.to_be_bytes())?;

        for k in 0..q {
            let byte = u8::from_str_radix(&padded_bitcode[8 * k..8 * (k + 1)], 2)?;
            file.write_all(&[byte])?;
        }

        Ok(())
    }
}
