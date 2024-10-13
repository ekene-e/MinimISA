use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;
use crate::enums::{Token, LexType};
use crate::errors::TokenError;
use crate::util::{Stack, huffman, sub};

pub struct Lexer {
    rexp: Regex,
    aliases: HashMap<LexType, HashMap<String, String>>,
    possible_transitions: HashMap<String, Vec<String>>,
    includes: HashSet<String>,
}

impl Lexer {
    pub fn new(possible_transitions: HashMap<String, Vec<String>>) -> Self {
        let mut token_specification = HashMap::new();

        token_specification.insert(LexType::OPERATION, 
            r"\b(?:add|sub|cmp|let|shift|readze|readse|jump|or|and|write|call|setctr|getctr|push|return|xor|asr|pop|sleep|rand)\b");
        
        token_specification.insert(LexType::COMMENT, r";(?:.|[ \t])*");
        token_specification.insert(LexType::REGISTER, r"\b(?:r|R)[0-9]+\b");
        token_specification.insert(LexType::DIRECTION, r"\b(?:left|right)\b");
        token_specification.insert(LexType::NUMBER, r"[+-]?(?:0x[0-9A-Fa-f]+|[0-9]+)\b");
        token_specification.insert(LexType::CONDITION, 
            r"\b(?:eq|z|neq|nz|sgt|slt|gt|ge|nc|lt|c|v|le)\b");
        token_specification.insert(LexType::MEMCOUNTER, r"\b(?:pc|sp|a0|a1)\b");

        token_specification.insert(LexType::LABEL, r"\b[a-zA-Z_][a-z_A-Z0-9]*:?");
        token_specification.insert(LexType::INCLUDE, r"\.include\s+[a-zA-Z_][a-z_A-Z0-9\.]*\b");
        token_specification.insert(LexType::CONS, r"\.const");
        token_specification.insert(LexType::BINARY, r"#[01]+");

        token_specification.insert(LexType::NEWLINE, r"\n");
        token_specification.insert(LexType::SKIP, r"[ \t]+");
        token_specification.insert(LexType::ENDFILE, r"$");
        token_specification.insert(LexType::MISMATCH, r".+");

        let tok_regex = token_specification.iter()
            .map(|(name, re)| format!("(?P<{}>{})", format!("{:?}", name), re))
            .collect::<Vec<String>>()
            .join("|");

        let rexp = Regex::new(&tok_regex).unwrap();

        // Define aliases for conditions
        let mut aliases = HashMap::new();
        let mut condition_aliases = HashMap::new();
        condition_aliases.insert("z".to_string(), "eq".to_string());
        condition_aliases.insert("nz".to_string(), "neq".to_string());
        condition_aliases.insert("nc".to_string(), "ge".to_string());
        condition_aliases.insert("c".to_string(), "lt".to_string());
        condition_aliases.insert("le".to_string(), "v".to_string());

        aliases.insert(LexType::CONDITION, condition_aliases);

        Lexer {
            rexp,
            aliases,
            possible_transitions,
            includes: HashSet::new(),
        }
    }

    pub fn lex(&mut self, code: &str, name: &str, directory: &str) -> impl Iterator<Item = Result<Token, TokenError>> {
        if self.includes.contains(name) {
            return vec![].into_iter(); // Return empty iterator if file already included
        }

        self.includes.insert(name.to_string());
        let mut line_num = 1;
        let mut line_start = 0;

        let tokens = self.rexp.find_iter(code).map(move |mat| {
            let kindname = mat.as_str();
            let value = mat.as_str().to_string();
            let kind = LexType::from_str(kindname).unwrap_or(LexType::MISMATCH);
            let column = mat.start() - line_start;

            let value = self.lex_alias(kind, value.clone());
            let value = self.lex_value(kindname, value.clone());

            match kind {
                LexType::NEWLINE | LexType::ENDFILE => {
                    line_start = mat.end();
                    line_num += 1;
                    Ok(Token::new(LexType::NEWLINE, None, name.to_string(), line_num - 1, column))
                }
                LexType::SKIP => Ok(Token::new(LexType::SKIP, None, name.to_string(), line_num, column)),
                LexType::MISMATCH => Err(TokenError::new(format!("Invalid syntax at line {} : {}", line_num, value))),
                LexType::LABEL => Ok(Token::new(LexType::LABEL, Some(value), name.to_string(), line_num, column)),
                LexType::CONS => Ok(Token::new(LexType::OPERATION, Some("const".to_string()), name.to_string(), line_num, column)),
                LexType::INCLUDE => {
                    let filename = format!("{}/{}", directory, value[9..].to_string());
                    let mut file = File::open(&filename).map_err(|e| {
                        println!("Lexer Error in file \"{}\" line {}: {}", filename, line_num, e);
                        exit(1);
                    })?;

                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;

                    // Recursively lex the included file
                    self.lex(&contents, &filename, directory).for_each(|t| {});
                    Ok(Token::new(LexType::INCLUDE, Some(value), name.to_string(), line_num, column))
                }
                _ => Ok(Token::new(kind, Some(value), name.to_string(), line_num, column)),
            }
        });

        tokens
    }

    fn lex_alias(&self, kind: LexType, value: String) -> String {
        if let Some(alias_map) = self.aliases.get(&kind) {
            if let Some(alias) = alias_map.get(&value) {
                return alias.clone();
            }
        }
        value
    }

    fn lex_value(&self, kindname: &str, value: String) -> String {
        match kindname {
            "NUMBER" => self.lex_value_NUMBER(value),
            "REGISTER" => self.lex_value_REGISTER(value),
            "LABEL" => self.lex_value_LABEL(value),
            _ => value,
        }
    }

    fn lex_value_NUMBER(&self, value: String) -> String {
        if value.to_lowercase().starts_with("0x") {
            return format!("{}", i64::from_str_radix(&value[2..], 16).unwrap());
        }
        value
    }

    fn lex_value_REGISTER(&self, value: String) -> String {
        value[1..].to_string()  // Remove 'r' or 'R' prefix
    }

    fn lex_value_LABEL(&self, value: String) -> String {
        if value.ends_with(':') {
            value[..value.len() - 1].to_string()  // Remove trailing ':'
        } else {
            value
        }
    }
}
