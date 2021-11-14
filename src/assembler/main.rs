
use std::fs::File;
use std::io::prelude::*;
use std::env;

use std::collections::HashMap;

struct Token {
    line: u16,
    identifier: String,
}

fn create_token(x: u16, s: String) -> Token {
    let t = Token {
        line: x,
        identifier: s,
    };
    t
}

fn get_char(s: &String, i: usize) -> char {
    let chars: Vec<char> = s.chars().skip(i).take(1).collect();
    let c: char = chars[0];
    c
}

fn peek_char(s: &String, i: usize) -> char {
    let chars: Vec<char> = s.chars().skip(i).take(1).collect();
    let c: char = chars[0];
    c
}

fn tokenise(src: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut line_number: u16 = 0;
    let mut token = String::new();
    let data_len = src.len();
    let mut i = 0;
    loop {
        if i == data_len { break; }
        let mut c = get_char(&src, i);
        
        while c.is_whitespace() {
            i = i + 1;
            c = get_char(&src, i);
            if c == '\n' {
                line_number = line_number + 1;
            }
        }

        if c == '/' {
            while c != '\n' {
                i = i + 1;
                c = get_char(&src, i);
            }
            if c == '\n' {
                line_number = line_number + 1;
            }
            i = i + 1;
            c = get_char(&src, i);
        }

        if c.is_ascii_alphanumeric() {
            if peek_char(&src, i + 1).is_ascii_alphanumeric() {
                if c.is_numeric() {
                    while c.is_numeric() {
                        token.push(c);
                        i = i + 1;
                        c = get_char(&src, i);
                    }
                    let t: Token = create_token(line_number, token.to_string());
                    tokens.push(t);
                    token = String::new();
                    continue;
                }
            } else {
                if peek_char(&src, i + 1) == ';' {
                    token.push(c);
                    i = i + 1;
                    let t: Token = create_token(line_number, token.to_string());
                    tokens.push(t);
                    token = String::new();
                    continue;
                }

                while c.is_ascii_alphabetic() {
                    token.push(c);
                    i = i + 1;
                    c = get_char(&src, i);
                }
                let t: Token = create_token(line_number, token.to_string());
                tokens.push(t);
                token = String::new();
                continue;
            }
        }

        token.push(c);
        i = i + 1;

        match &token[..] {
            "," => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            "$" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            ":" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            "#" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            "%" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            ";" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            _ => {},
        }
    }

    tokens
}

fn report_error(err: &str, line: u16){
    println!("{} at line {}", err, line + 1);
}

fn define_labels(tokens: &Vec<Token>) -> Option<HashMap<String, u8>> {
    let mut labels = HashMap::new();
    let mut rom_index: usize = 0;
    let mut i = 0;
    let token_length = tokens.len();
    let mut had_error: bool = false;

    loop {
        if i == token_length { break; }
        if had_error { return None }
        let mut t: &Token = &tokens[i];

        match &t.identifier[..] {
            "MOV" => { 
                i = i + 2;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line);
                    had_error = true;
                }
                i = i + 1; 
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%" {
                     i = i + 1;
                }
                rom_index = rom_index + 2;
                i = i + 1; 
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "STR" => { 
                i = i + 2;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line);
                    had_error = true;
                }
                i = i + 1; 
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                     i = i + 1;
                     rom_index = rom_index + 2;
                } 
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "PUSH" => {
                i = i + 1;
                t = &tokens[i];
                if !(&t.identifier[..] == "A" || &t.identifier[..] == "A" || &t.identifier[..] == "A" || &t.identifier[..] == "A") {
                    rom_index = rom_index + 1;
                }
                rom_index = rom_index + 1;
                i = i + 1;
            },
            "POP" => {
                i = i + 2;
                rom_index = rom_index + 1;
            },
            "SWP" => {
                i = i + 2;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line);
                    had_error = true;
                }
                i = i + 2; 
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
                rom_index = rom_index + 1;
            },
            "JMP" => {
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 1;
                }
                rom_index = rom_index + 2;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "JEZ" => {
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 1;
                }
                rom_index = rom_index + 2;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "JNZ" => {
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 1;
                }
                rom_index = rom_index + 2;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "CALL" => {
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 1;
                }
                rom_index = rom_index + 2;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "RET" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "OUT" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "ADD" => {
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") || t.identifier[..].starts_with("B") || t.identifier[..].starts_with("C") || t.identifier[..].starts_with("D") {
                    rom_index = rom_index + 1;
                    i = i + 1;
                } else if  &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 2;
                    rom_index = rom_index + 2;
                } else {
                    i = i + 1;
                    rom_index = rom_index + 2;
                }
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "SUB" => {
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") || t.identifier[..].starts_with("B") || t.identifier[..].starts_with("C") || t.identifier[..].starts_with("D") {
                    rom_index = rom_index + 1;
                    i = i + 1;
                } else if  &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 2;
                    rom_index = rom_index + 2;
                } else {
                    i = i + 1;
                    rom_index = rom_index + 2;
                }
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "AND" => {
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") || t.identifier[..].starts_with("B") || t.identifier[..].starts_with("C") || t.identifier[..].starts_with("D") {
                    rom_index = rom_index + 1;
                    i = i + 1;
                } else if  &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 2;
                    rom_index = rom_index + 2;
                } else {
                    i = i + 1;
                    rom_index = rom_index + 2;
                }
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "OR" => {
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") || t.identifier[..].starts_with("B") || t.identifier[..].starts_with("C") || t.identifier[..].starts_with("D") {
                    rom_index = rom_index + 1;
                    i = i + 1;
                } else if  &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 2;
                    rom_index = rom_index + 2;
                } else {
                    i = i + 1;
                    rom_index = rom_index + 2;
                }
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "XOR" => {
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") || t.identifier[..].starts_with("B") || t.identifier[..].starts_with("C") || t.identifier[..].starts_with("D") {
                    rom_index = rom_index + 1;
                    i = i + 1;
                } else if  &t.identifier[..] == "$" ||  &t.identifier[..] == "#" ||  &t.identifier[..] == "%"{
                    i = i + 2;
                    rom_index = rom_index + 2;
                } else {
                    rom_index = rom_index + 2;
                    i = i + 1;
                }
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "NOT" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "DEC" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "INC" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "HALT" => {
                rom_index = rom_index + 1;
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            ":" => {
                i = i + 1;
                t = &tokens[i];
                labels.insert(t.identifier[..].to_string(), rom_index as u8);
            },
            _ => {},
        }
        i = i + 1;
    }
    
    Some(labels)
}

fn assemble(tokens: &Vec<Token>, labels: &HashMap<String, u8>) -> Option<[u8; 256]> {
    let mut i = 0;
    let mut rom_index: usize = 0;
    let token_length = tokens.len();
    let mut rom: [u8; 256] = [0; 256];
    let mut had_error: bool = false;

    loop {
        if i == token_length { break; }
        let mut t: &Token = &tokens[i];
        let mut opcode: u8 = 0;
        
        match &t.identifier[..] {
            "MOV" => {
                opcode = opcode | (0x0 << 4); 
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "A" {
                    opcode = opcode | (0x00);
                } else if &t.identifier[..] == "B" {
                    opcode = opcode | (0x01);
                } else if &t.identifier[..] == "C" {
                    opcode = opcode | (0x02);
                } else if &t.identifier[..] == "D" {
                    opcode = opcode | (0x03);
                } else {
                    report_error("Invalid operand", t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" {
                    opcode = opcode | (0x00 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier,16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                    continue;
                }
            },
            "STR" =>{
                opcode = opcode | (0x1 << 4); 
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "A" {
                    opcode = opcode | (0x00);
                } else if &t.identifier[..] == "B" {
                    opcode = opcode | (0x01);
                } else if &t.identifier[..] == "C" {
                    opcode = opcode | (0x02);
                } else if &t.identifier[..] == "D" {
                    opcode = opcode | (0x03);
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" {
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } 

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "PUSH" =>{
                opcode = opcode | (0x2 << 4); 
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "A" {
                    opcode = opcode | (0x00);
                } else if &t.identifier[..] == "B" {
                    opcode = opcode | (0x01);
                } else if &t.identifier[..] == "C" {
                    opcode = opcode | (0x02);
                } else if &t.identifier[..] == "D" {
                    opcode = opcode | (0x03);
                } else {
                    opcode = opcode | (0x04);
                }

                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                if opcode & 0x04 == 0x04 {
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } 

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "POP" =>{
                opcode = opcode | (0x3 << 4); 
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "A" {
                    opcode = opcode | (0x00);
                } else if &t.identifier[..] == "B" {
                    opcode = opcode | (0x01);
                } else if &t.identifier[..] == "C" {
                    opcode = opcode | (0x02);
                } else if &t.identifier[..] == "D" {
                    opcode = opcode | (0x03);
                }

                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "SWP" => {
                opcode = opcode | (0x4 << 4); 
                i = i + 1;
                t = &tokens[i];

                if &t.identifier[..] == "A" {
                    opcode = opcode | (0x00 << 2);
                } else if &t.identifier[..] == "B" {
                    opcode = opcode | (0x01 << 2);
                } else if &t.identifier[..] == "C" {
                    opcode = opcode | (0x02 << 2);
                } else if &t.identifier[..] == "D" {
                    opcode = opcode | (0x03 << 2);
                }

                let reg: String = t.identifier.clone();

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    report_error("Expected comma", t.line)
                }

                i = i + 1;
                t = &tokens[i];

                if t.identifier == reg {
                    rom[rom_index] = 0xF0; // NOP
                } else {
                    if &t.identifier[..] == "A" {
                        opcode = opcode | (0x00);
                    } else if &t.identifier[..] == "B" {
                        opcode = opcode | (0x01);
                    } else if &t.identifier[..] == "C" {
                        opcode = opcode | (0x02);
                    } else if &t.identifier[..] == "D" {
                        opcode = opcode | (0x03);
                    }
                    rom[rom_index] = opcode;
                }
                rom_index = rom_index + 1;    

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "JMP" =>{
                opcode = opcode | (0x5 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" || &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if labels.contains_key(&t.identifier[..].to_string()) {
                    rom[rom_index] = labels[&t.identifier[..].to_string()] as u8;
                    rom_index = rom_index + 1;
                } else {
                    // handle error
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "JEZ" =>{
                opcode = opcode | (0x6 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" || &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if labels.contains_key(&t.identifier[..].to_string()) {
                    rom[rom_index] = labels[&t.identifier[..].to_string()] as u8;
                    rom_index = rom_index + 1;
                } else {
                    // handle error
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "JNZ" =>{
                opcode = opcode | (0x7 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" || &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if labels.contains_key(&t.identifier[..].to_string()) {
                    rom[rom_index] = labels[&t.identifier[..].to_string()] as u8;
                    rom_index = rom_index + 1;
                } else {
                    // handle error
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "CALL" =>{
                opcode = opcode | (0x8 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "$" || &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if labels.contains_key(&t.identifier[..].to_string()) {
                    rom[rom_index] = labels[&t.identifier[..].to_string()] as u8;
                    rom_index = rom_index + 1;
                } else {
                    // handle error
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "RET" =>{
                opcode = opcode | (0x9 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "OUT" => {
                opcode = opcode | (0xa << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "ADD" => {
                opcode = opcode | (0xb << 4); 
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") {
                    opcode = opcode | (0x00);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("B") {
                    opcode = opcode | (0x01);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("C") {
                    opcode = opcode | (0x02);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("D") {
                    opcode = opcode | (0x03);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if  &t.identifier[..] == "$"{
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if  &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    } 
                } else {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "SUB" => {
                opcode = opcode | (0xc << 4); 
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") {
                    opcode = opcode | (0x00);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("B") {
                    opcode = opcode | (0x01);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("C") {
                    opcode = opcode | (0x02);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("D") {
                    opcode = opcode | (0x03);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if  &t.identifier[..] == "$"{
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if  &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "AND" => {
                opcode = opcode | (0xd << 4); 
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") {
                    opcode = opcode | (0x00);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("B") {
                    opcode = opcode | (0x01);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("C") {
                    opcode = opcode | (0x02);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("D") {
                    opcode = opcode | (0x03);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if  &t.identifier[..] == "$" {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "OR" => {
                opcode = opcode | (0xd << 4);
                opcode = opcode | 0x4; 
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") {
                    opcode = opcode | (0x00);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("B") {
                    opcode = opcode | (0x01);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("C") {
                    opcode = opcode | (0x02);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("D") {
                    opcode = opcode | (0x03);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if  &t.identifier[..] == "$" {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "XOR" => {
                opcode = opcode | (0xe << 4); 
                i = i + 1;
                t = &tokens[i];
                if t.identifier[..].starts_with("A") {
                    opcode = opcode | (0x00);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("B") {
                    opcode = opcode | (0x01);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("C") {
                    opcode = opcode | (0x02);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if t.identifier[..].starts_with("D") {
                    opcode = opcode | (0x03);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;
                } else if  &t.identifier[..] == "$" {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else if &t.identifier[..] == "%" {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 2) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 3);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 10) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "NOT" => {
                opcode = opcode | (0xe << 4);
                opcode = opcode | 0x4; 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "INC" => {
                opcode = 0xA2; 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "DEC" => {
                opcode = 0xA1; 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            "HALT" => { 
                rom[rom_index] = 0xff; 
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    report_error("Expected semicolon", t.line);
                    had_error = true;
                }
            },
            ":" => {
                i = i + 1;
            },
            _ => {},
        }
        i = i + 1;
    }

    if had_error {
        None
    } else {
        Some(rom)
    }
}

fn validate_filetype(src: &String) -> bool {
    let v: Vec<&str> = src.split(".").collect();
    v[1] == "rsm"
}

fn main() ->std::io::Result<()> {
    
    let mut src_str = String::new();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    if !validate_filetype(&filename) {
        println!("Invalid file type. Only .rsm files can be assembled.");
        return Ok(());
    }

    let mut f = File::open(filename)?;
    f.read_to_string(&mut src_str)?;

    let tokens: Vec<Token> = tokenise(&src_str);
    
    let labels = define_labels(&tokens);
    match labels {
        None =>  { println!{"Failed to assemble source code."}; },
        Some(labels) => {
            let rom = assemble(&tokens, &labels);
            match rom {
                Some(rom) => {
                    if args.len() == 3 {
                        if args[2] == "DEBUG" {
                            for i in 0..256 {
                                println!("ROM [{}] -- {}", i, rom[i]);
                            }
                        }
                    }
        
                    let output_filename = filename.split(".").next();
                    match output_filename {
                        Some(output_filename) => { 
                            let mut name: String = output_filename.to_owned();
                            name.push_str(".rbin");
                            let mut output = File::create(name)?;
                            output.write(&rom)?; 
                
                        },
                        None => {},
                    }
                },
                None => { println!{"Failed to assemble source code."}; },
            }
        }  
    }
    
    Ok(())
}

