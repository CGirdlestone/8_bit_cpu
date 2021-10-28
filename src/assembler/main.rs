
use std::fs::File;
use std::io::prelude::*;
use std::env;

struct Token {
    line: u8,
    identifier: String,
}

fn create_token(x: u8, s: String) -> Token {
    let t = Token {
        line: x,
        identifier: s,
    };
    t
}

fn get_char(s: &String, i: usize) -> char {
    let chars: Vec<char> = s.chars().skip(i).take(1).collect();
    let c: char = chars[0];
    return c;
}

fn peek_char(s: &String, i: usize) -> char {
    let chars: Vec<char> = s.chars().skip(i).take(1).collect();
    let c: char = chars[0];
    return c;
}

fn main() ->std::io::Result<()> {
    
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename)?;
    let mut data_str = String::new();
    f.read_to_string(&mut data_str)?;

    let mut tokens: Vec<Token> = Vec::new();
    let mut line_number: u8 = 0;
    let mut token = String::new();
    let data_len = data_str.len();
    let mut i = 0;
    loop {
        if i == data_len { break; }
        let mut c = get_char(&data_str, i);
        while c.is_whitespace() {
            i = i + 1;
            c = get_char(&data_str, i);
            if c == '\n' {
                line_number = line_number + 1;
            }
        }

        if c == '%' {
            while c != '\n' {
                i = i + 1;
                c = get_char(&data_str, i);
            }
            if c == '\n' {
                line_number = line_number + 1;
            }
            i = i + 1;
            c = get_char(&data_str, i);
        }

        if c.is_ascii_alphanumeric() {
            if peek_char(&data_str, i + 1).is_ascii_alphanumeric() {
                if c.is_numeric() {
                    while c.is_numeric() {
                        token.push(c);
                        i = i + 1;
                        c = get_char(&data_str, i);
                    }
                    let t: Token = create_token(line_number, token.to_string());
                    tokens.push(t);
                    token = String::new();
                    continue;
                }
            } else {
                if peek_char(&data_str, i + 1) == ';' {
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
                    c = get_char(&data_str, i);
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
            ";" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            "#" => { tokens.push(create_token(line_number, token.to_string())); token = String::new(); continue; },
            _ => {},
        }
    }

    let mut rom_index: usize = 0;
    let mut rom: [u8; 256] = [0; 256];
    let mut had_error: bool = false;
    i = 0;
    let token_length = tokens.len();
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
                    println!("Invalid operand at line {}.", &t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != "," {
                    println!("Expected comma at line {}.", &t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    opcode = opcode | (0x00 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    opcode = opcode | (0x01 << 2);
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("-- {}", &t.identifier);
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
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
                    println!("Expected comma at line {}.", &t.line);
                    had_error = true;
                }

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    rom[rom_index] = opcode;
                    rom_index = rom_index + 1;

                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    // handle error
                }
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            "PSH" =>{
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
                }

                rom[rom_index] = opcode;
                rom_index = rom_index + 1;

                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                    println!("Expected comma.")
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
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            "JMP" =>{
                opcode = opcode | (0x5 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    // handle error
                }
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            "JEZ" =>{
                opcode = opcode | (0x6 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    // handle error
                }
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            "JNZ" =>{
                opcode = opcode | (0x7 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    // handle error
                }
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            "CLL" =>{
                opcode = opcode | (0x8 << 4); 
                rom[rom_index] = opcode;
                rom_index = rom_index + 1;
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] == "#" {
                    i = i + 1;
                    t = &tokens[i];
                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                } else {
                    // handle error
                }
                
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
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

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
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

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
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

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
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

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                } else if  &t.identifier[..] == "#" {
                    opcode = opcode | (0x01 << 2);
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

                    if let Ok(y) = u8::from_str_radix(&t.identifier, 16) {
                        rom[rom_index] = y;
                        rom_index = rom_index + 1;
                    }
                }
                i = i + 1;
                t = &tokens[i];
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
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
                   // handle error
                }
            },
            "HLT" => { 
                rom[rom_index] = 0xff; 
                i = i + 1; 
                t = &tokens[i]; 
                if &t.identifier[..] != ";" {
                    println!("Expected semicolon at end of line {}.", &t.line);
                    had_error = true;
                }
            },
            _ => {},
        }
        i = i + 1;
    }

    for i in 0..256 {
        println!("ROM [{}] -- {}", i, rom[i]);
    }

    let output_filename = filename.split(".").next();
    match output_filename {
        Some(output_filename) => { 
            let mut name: String = output_filename.to_owned();
            name.push_str(".rbin");
            println!("{}", name);
            let mut output = File::create(name)?;
            output.write(&rom); 

        },
        None => {},
    }

    Ok(())
}