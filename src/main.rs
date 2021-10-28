
use std::fs::File;
use std::io::prelude::*;
use std::env;

const B_IN: u16 = 0x1;
const A_IN: u16 = 0x2;
const C_IN: u16 = 0x3;
const IR_IN: u16 = 0x4;
const MDR_IN: u16 = 0x5;
const MAR_IN: u16 = 0x6;
const PC_IN: u16 = 0x7;

const ALU_OUT: u16 = 0x1 << 3;
const B_OUT: u16 = 0x2 << 3;
const A_OUT: u16 = 0x3 << 3;
const MDR_OUT: u16 = 0x4 << 3;
const RAM_OUT: u16 = 0x5 << 3;
const ROM_OUT: u16 = 0x6 << 3;
const PC_OUT: u16 = 0x7 << 3;

const NOT_OUT: u16 = 0x1 << 6;
const OR_OUT: u16 = 0x2 << 6;
const SP_DEC: u16 = 0x3 << 6;
const SP_INC: u16 = 0x4 << 6;
const SUB: u16 = 0x5 << 6;
const AND_OUT: u16 = 0x6 << 6;
const PC_INC: u16 = 0x7 << 6;

const C_OUT: u16 = 0x1 << 9;
const RAM_IN: u16 = 0x1 << 10;
const XOR_OUT: u16 = 0x1 << 11;
const D_IN: u16 = 0x1 << 12;
const D_OUT: u16 = 0x1 << 13;


struct CPU {
    pc: u8, // program counter
    a: u8, // a(ccumulator) register
    b: u8, // b register (general purpose)
    c: u8, // c register (output)
    d: u8,
    alu: u8, // arithmetic logic unit
    _and: u8,
    _or: u8,
    _xor: u8,
    _not: u8,
    mar: u8, // memory address register
    mdr: u8, // memory data register
    ir: u8, // instruction register
    bus: u8,
    sp: u8, // stack pointer
    eeprom: [u16; 8192], // eeprom containing the cpu control signals
    ram: [u8; 256], // random access memory (upper 128 bytes used by the cpu stack)
    rom: [u8; 256], // read only memory - contains the program code
    halt: u8, // program halt signal
    flags: u8, // cpu flags - currently only two are used (zero and carry) XXXX XXZC
    had_error: bool,
}

fn xor (a: u8, b: u8) -> u8 {
    a ^ b
}

fn and (a: u8, b :u8) -> u8 {
    a & b
}

fn or (a: u8, b: u8) -> u8 {
    a | b
}

fn not (a: u8) -> u8 {
    !a
}

fn add (a: u8, mut b: u8, flags: &mut u8, subtract: u8) -> u8 {
    let mut sum: u8 = 0x00;
    let mut carry: u8 = 0x00;

    if subtract == 0x01 {
        b = add(not(b), 0x01, &mut 0x00, 0x00);
    }

    for bit in 0..8 {
        let out: u8 = xor(xor((a >> bit) & 0x01, (b >> bit) & 0x01), carry);
        sum |= out << bit;
        carry = or(and((a >> bit) & 0x01, (b >> bit) & 0x01), and(xor((a >> bit) & 0x01, (b >> bit) & 0x01), carry));
    }

    // set carry flag
    *flags = *flags | (carry << 0);
    
    // set zero flag
    if sum == 0x00 {
        *flags = *flags | (1 << 1);
    } else {
        *flags = *flags | (0 << 1);
    }
    sum
}

fn get_address(_cpu: &mut CPU, inst: u8, t: u8) -> u16 {
    let address: u16 = (t as u16) << 8 | (inst as u16) | (_cpu.flags as u16) << 11;
    return address;
}


fn execute_micro_instruction(_cpu: &mut CPU, step: u8) {
    if _cpu.ir == 0xff {
        _cpu.halt = 0x00;
        return;
    }

    let micro: u16 = _cpu.eeprom[get_address(_cpu, _cpu.ir, step) as usize];

    match micro {
        micro if micro == PC_OUT | MAR_IN /*0x03E*/ => { _cpu.bus = _cpu.pc; _cpu.mar = _cpu.bus; },
        micro if micro == PC_OUT | MAR_IN | PC_INC /*0x1FE*/ => { _cpu.bus = _cpu.pc; _cpu.mar = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == ROM_OUT | IR_IN | PC_INC /*0x1F4*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.ir = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == ROM_OUT | MAR_IN | PC_INC /*0x1F6*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.mar = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == RAM_OUT | MDR_IN /*0x02D*/ => { _cpu.bus = _cpu.ram[_cpu.mar as usize]; _cpu.mdr = _cpu.bus; },
        micro if micro == MDR_OUT | A_IN /*0x022*/ => { _cpu.bus = _cpu.mdr; _cpu.a = _cpu.bus; },
        micro if micro == MDR_OUT | B_IN /*0x021*/ => { _cpu.bus = _cpu.mdr; _cpu.b = _cpu.bus; },
        micro if micro == MDR_OUT | C_IN /*0x023*/ => { _cpu.bus = _cpu.mdr; _cpu.c = _cpu.bus; },
        micro if micro == MDR_OUT | D_IN /*0x1020*/ => { _cpu.bus = _cpu.mdr; _cpu.d = _cpu.bus; },
        micro if micro == ALU_OUT | A_IN /*0x00A*/ => { _cpu.alu = add(_cpu.a, _cpu.b, &mut _cpu.flags, 0x00); _cpu.bus = _cpu.alu; _cpu.a = _cpu.bus; },
        micro if micro == ALU_OUT | A_IN | SUB /*0x14A*/ => { _cpu.alu = add(_cpu.a, _cpu.b, &mut _cpu.flags, 0x01); _cpu.bus = _cpu.alu; _cpu.a = _cpu.bus; },
        micro if micro == AND_OUT | A_IN /*0x182*/  => { _cpu._and = and(_cpu.a, _cpu.b); _cpu.bus = _cpu._and; _cpu.a = _cpu.bus; },
        micro if micro == OR_OUT | A_IN /*0x082*/  => { _cpu._or = or(_cpu.a, _cpu.b); _cpu.bus = _cpu._or; _cpu.a = _cpu.bus; },
        micro if micro == XOR_OUT | A_IN /*0x802*/  => { _cpu._xor = xor(_cpu.a, _cpu.b); _cpu.bus = _cpu._xor; _cpu.a = _cpu.bus; },
        micro if micro == NOT_OUT | A_IN /*0x042*/  => { _cpu._not = not(_cpu.a); _cpu.bus = _cpu._not; _cpu.a = _cpu.bus; },
        micro if micro == ROM_OUT | PC_IN /*0x037*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.pc = _cpu.bus; },
        micro if micro == A_OUT | C_IN /*0x01B*/ => { _cpu.bus = _cpu.a; _cpu.c = _cpu.bus; println!("{}", _cpu.c); },
        micro if micro == ROM_OUT | A_IN | PC_INC /*0x1F2*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.a = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == ROM_OUT | B_IN | PC_INC /*0x1F1*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.b = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == ROM_OUT | C_IN | PC_INC /*0x1F3*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.c = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == ROM_OUT | D_IN | PC_INC /*0x11F0*/ => {_cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.d = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        micro if micro == A_OUT | RAM_IN /*0x418*/ => { _cpu.bus = _cpu.a; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        micro if micro == B_OUT | RAM_IN /*0x410*/ => { _cpu.bus = _cpu.b; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        micro if micro == C_OUT | RAM_IN /*0x600*/ => { _cpu.bus = _cpu.c; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        micro if micro == D_OUT | RAM_IN /*0x2400*/ => { _cpu.bus = _cpu.d; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        micro if micro == A_OUT | RAM_IN | SP_INC /*0x5D8*/ => { _cpu.bus = _cpu.a; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == B_OUT | RAM_IN | SP_INC /*0x5D0*/ => { _cpu.bus = _cpu.b; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == C_OUT | RAM_IN | SP_INC /*0x7C0*/ => { _cpu.bus = _cpu.c; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == D_OUT | RAM_IN | SP_INC /*0x25C0*/ => { _cpu.bus = _cpu.d; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == ROM_OUT | RAM_IN | SP_INC /*0x530*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.ram[(_cpu.sp)  as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == SP_DEC | RAM_OUT | A_IN /*0x0EA*/ => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp) as usize]; _cpu.a = _cpu.bus; },
        micro if micro == SP_DEC | RAM_OUT | B_IN /*0x0E9*/ => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp) as usize]; _cpu.b = _cpu.bus; },
        micro if micro == SP_DEC | RAM_OUT | C_IN /*0x0EB*/ => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp) as usize]; _cpu.c = _cpu.bus; },
        micro if micro == SP_DEC | RAM_OUT | D_IN /*0x10E8*/ => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp) as usize]; _cpu.d = _cpu.bus; },
        micro if micro == B_OUT | RAM_IN | SP_INC /*0x2A0*/ => { _cpu.bus = _cpu.b; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == PC_OUT | RAM_IN | SP_INC /*0x1B8*/ => { _cpu.bus = _cpu.pc; _cpu.ram[(_cpu.sp) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        micro if micro == SP_DEC | RAM_OUT | PC_IN /*0x0EF*/ => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp) as usize]; _cpu.pc = _cpu.bus; },
        micro if micro == ROM_OUT | RAM_IN /*0x1B0*/ => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.ram[(_cpu.sp)  as usize] = _cpu.bus; },
        micro if micro == A_OUT | B_IN /*0x019*/ => { _cpu.bus = _cpu.a; _cpu.b = _cpu.bus; },
        micro if micro == C_OUT | B_IN /*0x219*/ => { _cpu.bus = _cpu.c; _cpu.b = _cpu.bus; },
        micro if micro == PC_INC /*0x1C0*/ => { _cpu.pc = _cpu.pc + 1; },
        micro if micro == SP_INC /*0x100*/ => { _cpu.sp = _cpu.sp + 1; },
        _ => return,
    }
}

fn execute_program(_cpu: &mut CPU) {
    loop {
        for i in 0..8 {
            execute_micro_instruction(_cpu, i as u8);
        }
        if _cpu.halt == 0x00 {
            break;
        }
    }
}

fn load_eeprom(_cpu: &mut CPU) {
    // all opcodes must fetch instruction and increment program counter
    for i in 0..255 {
        // any flags state
        for j in 0..4 {
            _cpu.eeprom[(j << 11) | i] = PC_OUT | MAR_IN; // (PC out, MAR in) for all instructions at microstep 0 --> X X 000 XXXX XXXX
            _cpu.eeprom[(j << 11) | (0x1 << 8) | i] = ROM_OUT | IR_IN | PC_INC; // (rom out, IR in, PC inc) for all instructions at microstep 1 --> X X 001 XXXX XXXX
        }
    }

    // HALT OPCODE 1111 1111
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xff] = 0x000; // (halt)
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xff] = 0x000; // 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xff] = 0x000; // 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xff] = 0x000; // 
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xff] = 0x000; // 
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xff] = 0x000; // 
    }

    // LOAD A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x00] = PC_OUT | MAR_IN; // (PC out, MAR in)          
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x00] = ROM_OUT | MAR_IN | PC_INC; // (rom out, MAR in, pc inc) 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x00] = RAM_OUT | MDR_IN; //0x02D; // (RAM out, MDR in)         
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x00] = MDR_OUT | A_IN; //0x022; // (MDR out, A in)           
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x00] = 0x000; //                           
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x00] = 0x000; //                           
    }

    // LOAD B
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x01] = PC_OUT | MAR_IN; // (PC out, MAR in)          
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x01] = ROM_OUT | MAR_IN | PC_INC; // (rom out, MAR in, pc inc) 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x01] = RAM_OUT | MDR_IN; // (RAM out, MDR in)         
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x01] = MDR_OUT | B_IN; // (MDR out, B in)           
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x01] = 0x000; //                           
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x01] = 0x000; //                           
    }

    // LOAD C
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x02] = PC_OUT | MAR_IN; // (PC out, MAR in)          
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x02] = ROM_OUT | MAR_IN | PC_INC; // (rom out, MAR in, pc inc) 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x02] = RAM_OUT | MDR_IN; // (RAM out, MDR in)         
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x02] = MDR_OUT | C_IN; // (MDR out, C in)           
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x02] = 0x000; //                           
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x02] = 0x000; //                           
    }

    // LOAD D
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x03] = PC_OUT | MAR_IN; // (PC out, MAR in)          
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x03] = ROM_OUT | MAR_IN | PC_INC; // (rom out, MAR in, pc inc) 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x03] = RAM_OUT | MDR_IN; // (RAM out, MDR in)         
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x03] = MDR_OUT | D_IN; // (MDR out, D in)           
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x03] = 0x000; //                           
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x03] = 0x000; //                           
    }

    // LOAD A IMMEDIATE 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x04] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x04] = ROM_OUT | A_IN | PC_INC; // rom out, A in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x04] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x04] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x04] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x04] = 0x000;
    }

    // LOAD B IMMEDIATE 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x05] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x05] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x05] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x05] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x05] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x05] = 0x000;
    }

    // LOAD C IMMEDIATE 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x06] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x06] = ROM_OUT | C_IN | PC_INC; // rom out, C in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x06] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x06] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x06] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x06] = 0x000;
    }

    // LOAD C IMMEDIATE 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x07] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x07] = ROM_OUT | D_IN | PC_INC; // rom out, D in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x07] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x07] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x07] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x07] = 0x000;
    }

    // STORE A OPCODE
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x10] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x10] = ROM_OUT | MAR_IN | PC_INC; // rom out, MAR in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x10] = A_OUT | RAM_IN; // A out, RAM in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x10] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x10] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x10] = 0x000;
    }

    // STORE B OPCODE
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x11] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x11] = ROM_OUT | MAR_IN | PC_INC; // rom out, MAR in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x11] = B_OUT | RAM_IN; // B out, RAM in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x11] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x11] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x11] = 0x000;
    }

    // STORE C OPCODE
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x12] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x12] = ROM_OUT | MAR_IN | PC_INC; // rom out, MAR in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x12] = C_OUT | RAM_IN; // C out, RAM in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x12] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x12] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x12] = 0x000;
    }

    // STORE D OPCODE
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x13] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x13] = ROM_OUT | MAR_IN | PC_INC; // rom out, MAR in, PC inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x13] = D_OUT | RAM_IN; // D out, RAM in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x13] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x13] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x13] = 0x000;
    }

    // PUSH (from register A)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x20] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x20] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x20] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x20] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x20] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x20] = 0x000;
    }

    // PUSH (from register B) 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x21] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x21] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x21] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x21] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x21] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x21] = 0x000;
    }

    // PUSH (from register C) 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x22] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x22] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x22] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x22] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x22] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x22] = 0x000;
    }

    // PUSH (from register D) 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x23] = D_OUT | RAM_IN | SP_INC; // D out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x23] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x23] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x23] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x23] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x23] = 0x000;
    }

    // PUSH (immediate)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x24] = PC_OUT | MAR_IN;
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x24] = ROM_OUT | RAM_IN | SP_INC;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x24] = PC_INC;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x24] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x24] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x24] = 0x000; 
    }

    // POP (to register A)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x30] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x30] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x30] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x30] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x30] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x30] = 0x000;
    }

    // POP (to register B)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x31] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x31] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x31] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x31] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x31] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x31] = 0x000;
    }

    // POP (to register C)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x32] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x32] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x32] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x32] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x32] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x32] = 0x000;
    }

    // POP (to register D)
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x33] = RAM_OUT | D_IN | SP_DEC; // RAM out, D in, SP dec
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x33] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x33] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x33] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x33] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x33] = 0x000;
    }

    // SWAP A with B
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x41] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x41] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x41] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x41] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x41] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x41] = 0x000;
    }

    // SWAP A with C
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x42] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x42] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x42] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x42] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x42] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x42] = 0x000;
    }

    // SWAP A with D
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x43] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x43] = D_OUT | RAM_IN | SP_INC; // D out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x43] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x43] = RAM_OUT | D_IN | SP_DEC; // RAM out, D in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x43] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x43] = 0x000;
    }

    // SWAP B with A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x44] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x44] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x44] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x44] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x44] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x44] = 0x000; 
    }

    // SWAP B with C
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x46] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x46] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x46] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x46] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x46] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x46] = 0x000;
    }
    
    // SWAP B with D
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x47] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x47] = D_OUT | RAM_IN | SP_INC; // D out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x47] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x47] = RAM_OUT | D_IN | SP_DEC; // RAM out, D in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x47] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x47] = 0x000;
    }

    // SWAP C with A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x48] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x48] = A_OUT | RAM_IN | SP_INC; // A out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x48] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x48] = RAM_OUT | A_IN | SP_DEC; // RAM out, A in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x48] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x48] = 0x000;
    }

    // SWAP C with B
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x49] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x49] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x49] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x49] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x49] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x49] = 0x000;
    }

    // SWAP C with D
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x4b] = C_OUT | RAM_IN | SP_INC; // C out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x4b] = D_OUT | RAM_IN | SP_INC; // D out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x4b] = RAM_OUT | C_IN | SP_DEC; // RAM out, C in, SP dec
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x4b] = RAM_OUT | D_IN | SP_DEC; // RAM out, D in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x4b] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x4b] = 0x000;
    }

     // JUMP
     for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x50] = PC_OUT | MAR_IN | PC_INC; // PC out, MAR in, PC inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x50] = ROM_OUT | PC_IN; // rom out, PC in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x50] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x50] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x50] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x50] = 0x000;
    }

    // JUMP EQUAL ZERO
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x60] = PC_OUT | MAR_IN | PC_INC; // PC out, MAR in, PC inc
        if i == 2 || i == 3 {
            _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x60] = ROM_OUT | PC_IN; // rom out, PC in
        }
        else {
            _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x60] = 0x000; // do nothing when the zero flag is not set --> 0 X 011 0001 0010
        }
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x60] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x60] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x60] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x60] = 0x000;
    }

    // JUMP NOT EQUAL ZERO OPCODE 0001 0010
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x70] = PC_OUT | MAR_IN | PC_INC; // PC out, MAR in, PC inc
        if i == 0 || i == 1 {
            _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x70] = ROM_OUT | PC_IN; // rom out, PC in        
        }
        else {
            _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x70] = 0x000; // do nothing when the zero flag is set --> 1 X 011 0001 0010
        }
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x70] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x70] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x70] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x70] = 0x000;
    }

    // OUT 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xA0] = A_OUT | C_IN; // A out, C in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xA0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xA0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xA0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xA0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xA0] = 0x000;
    }

    // CALL 
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x80] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x80] = PC_OUT | MAR_IN | PC_INC; // PC out, MAR in, PC inc 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x80] = PC_OUT | RAM_IN | SP_INC; // PC out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x80] = ROM_OUT | PC_IN; // rom out, PC in
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x80] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x80] = 0x000;
    }

    // RETURN OPCODE 0011 0000
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0x90] = RAM_OUT | PC_IN | SP_DEC; // RAM out, PC in, SP dec
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0x90] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0x90] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0x90] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0x90] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0x90] = 0x000;
    }

    // ADD A to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xB0] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xB0] = A_OUT | B_IN; // A out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xB0] = ALU_OUT | A_IN; // ALU out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xB0] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xB0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xB0] = 0x000; 
    }

    // ADD B to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xB1] = ALU_OUT | A_IN; // ALU out, A in 
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xB1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xB1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xB1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xB1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xB1] = 0x000; 
    }

    // ADD C to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xB2] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xB2] = C_OUT | B_IN; // C out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xB2] = ALU_OUT | A_IN; // ALU out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xB2] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xB2] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xB2] = 0x000; 
    }

    // ADD imm to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xB8] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xB8] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xB8] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xB8] = ALU_OUT | A_IN; // ALU out, A in
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xB8] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xB8] = 0x000;
    }

    // SUB A to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xC0] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xC0] = A_OUT | B_IN; // A out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xC0] = ALU_OUT | A_IN | SUB; // ALU out, A in, SUB
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xC0] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xC0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xC0] = 0x000; 
    }

    // SUB B to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xC1] = ALU_OUT | A_IN | SUB; // ALU out, A in, SUB
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xC1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xC1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xC1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xC1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xC1] = 0x000; 
    }

    // SUB C to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xC2] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xC2] = C_OUT | B_IN; // C out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xC2] = ALU_OUT | A_IN | SUB; // ALU out, A in, SUB
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xC2] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xC2] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xC2] = 0x000; 
    }

    // SUB imm to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xC8] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xC8] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xC8] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xC8] = ALU_OUT | A_IN | SUB; // ALU out, A in, SUB
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xC8] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xC8] = 0x000;
    }

    // AND A to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD0] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD0] = A_OUT | B_IN; // A out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD0] = AND_OUT | A_IN ; // AND out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD0] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD0] = 0x000; 
    }

    // AND B to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD1] = AND_OUT | A_IN ; // AND out, A in 
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD1] = 0x000; 
    }

    // AND C to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD2] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD2] = C_OUT | B_IN; // C out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD2] = AND_OUT | A_IN ; // AND out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD2] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD2] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD2] = 0x000; 
    }

    // AND imm to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD8] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD8] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD8] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD8] = AND_OUT | A_IN ; // AND out, A in
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD8] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD8] = 0x000;
    }

    // OR A to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD4] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD4] = A_OUT | B_IN; // A out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD4] = OR_OUT | A_IN ; // OR out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD4] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD4] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD4] = 0x000; 
    }

    // OR B to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD5] = OR_OUT | A_IN ; // OR out, A in 
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD5] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD5] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD5] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD5] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD5] = 0x000; 
    }

    // OR C to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xD6] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xD6] = C_OUT | B_IN; // C out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xD6] = OR_OUT | A_IN ; // OR out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xD6] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xD6] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xD6] = 0x000; 
    }

    // OR imm to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xDC] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xDC] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xDC] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xDC] = OR_OUT | A_IN ; // OR out, A in
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xDC] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xDC] = 0x000;
    }

    // XOR A to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xE0] = B_OUT | RAM_IN | SP_INC; // B out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xE0] = A_OUT | B_IN; // A out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xE0] = XOR_OUT | A_IN ; // XOR out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xE0] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xE0] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xE0] = 0x000; 
    }

    // XOR B to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xE1] = XOR_OUT | A_IN ; // XOR out, A in 
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xE1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xE1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xE1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xE1] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xE1] = 0x000; 
    }

    // XOR C to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xE2] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xE2] = C_OUT | B_IN; // C out, B in 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xE2] = XOR_OUT | A_IN ; // XOR out, A in
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xE2] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xE2] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xE2] = 0x000; 
    }

    // XOR imm to A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xE8] = B_OUT | RAM_IN | SP_INC; // B  out, RAM in, SP inc
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xE8] = PC_OUT | MAR_IN; // PC out, MAR in
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xE8] = ROM_OUT | B_IN | PC_INC; // rom out, B in, PC inc
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xE8] = XOR_OUT | A_IN ; // XOR out, A in
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xE8] = RAM_OUT | B_IN | SP_DEC; // RAM out, B in, SP dec
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xE8] = 0x000;
    }

    // NOT A
    for i in 0..4 {
        _cpu.eeprom[(i << 11) | (0x2 << 8) | 0xE4] = NOT_OUT | A_IN ; // NOT out, A in
        _cpu.eeprom[(i << 11) | (0x3 << 8) | 0xE4] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x4 << 8) | 0xE4] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x5 << 8) | 0xE4] = 0x000; 
        _cpu.eeprom[(i << 11) | (0x6 << 8) | 0xE4] = 0x000;
        _cpu.eeprom[(i << 11) | (0x7 << 8) | 0xE4] = 0x000; 
    }
}



fn create_cpu() -> CPU {
    let mut _cpu = CPU {
        pc: 0,
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        alu: 0,
        _and: 0,
        _or: 0,
        _xor: 0,
        _not: 0,
        mar: 0,
        mdr: 0,
        ir: 0,
        bus: 0,
        sp: 0x80,
        eeprom: [0; 8192],
        ram: [0; 256],
        rom: [0; 256],
        halt: 0x01,
        flags: 0,
        had_error: false,
    };
    load_eeprom(&mut _cpu);
    return _cpu;
}

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

    let mut _cpu = create_cpu();

    let mut f = File::open(filename)?;
    let mut rom = Vec::new();
    f.read_to_end(&mut rom);

   for i in 0..256 {
        _cpu.rom[i] = rom[i];
    }

    

    
    

    if !_cpu.had_error {
        execute_program(&mut _cpu);
    } 

    Ok(())
}