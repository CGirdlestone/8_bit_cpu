
struct CPU {
    pc: u8,
    a: u8,
    b: u8,
    c: u8,
    alu: u8,
    mar: u8,
    mdr: u8,
    ir: u8,
    bus: u8,
    sp: u8,
    eeprom: [u16; 8192],
    ram: [u8; 256],
    rom: [u8; 256],
    halt: u8, 
    flags: u8,
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
    let mut z: u8 = 0x00;
    let mut carry: u8 = 0x00;

    if subtract & 0x01 == 0x01 {
        b = add(not(b), 0x01, &mut 0x00, 0x00);
    }

    for bit in 0..8 {
        let out: u8 = xor(xor((a >> bit) & 0x01, (b >> bit) & 0x01), carry);
        z |= out << bit;
        carry = or(and((a >> bit) & 0x01, (b >> bit) & 0x01), and(xor((a >> bit) & 0x01, (b >> bit) & 0x01), carry));
    }

    *flags = *flags | (carry << 0);
    if z == 0x00 {
        *flags = *flags | (1 << 1);
    } else {
        *flags = *flags | (0 << 1);
    }
    z
}

fn get_address(_cpu: &mut CPU, inst: u8, t: u8) -> u16 {
    let address: u16 = (t as u16) << 8 | (inst as u16) | (_cpu.flags as u16);
    return address;
}

fn load_code(_cpu: &mut CPU, code: [u8; 256]){
    for i in 0..255 {
        _cpu.rom[i] = code[i];
    }
}

fn execute_micro_instruction(_cpu: &mut CPU, step: u8) {
    if _cpu.ir == 0xff {
        _cpu.halt = 0x00;
        return;
    }

    match _cpu.eeprom[get_address(_cpu, _cpu.ir, step)  as usize] {
        0x07C => { _cpu.bus = _cpu.pc; _cpu.mar = _cpu.bus; },
        0x07D => { _cpu.bus = _cpu.pc; _cpu.mar = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        0x069 => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.ir = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        0x06D => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.mar = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        0x05A => { _cpu.bus = _cpu.ram[_cpu.mar as usize]; _cpu.mdr = _cpu.bus; },
        0x044 => { _cpu.bus = _cpu.mdr; _cpu.a = _cpu.bus; },
        0x042 => { _cpu.bus = _cpu.mdr; _cpu.b = _cpu.bus; },
        0x014 => { _cpu.alu = add(_cpu.a, _cpu.b, &mut _cpu.flags, 0x00); _cpu.bus = _cpu.alu; _cpu.a = _cpu.bus; },
        0x314 => { _cpu.alu = add(_cpu.a, _cpu.b, &mut _cpu.flags, 0x01); _cpu.bus = _cpu.alu; _cpu.a = _cpu.bus; },
        0x06E => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.pc = _cpu.bus; },
        0x034 => { _cpu.bus = _cpu.a; _cpu.c = _cpu.bus; println!("{}", _cpu.c); },
        0x065 => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.a = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        0x063 => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.b = _cpu.bus; _cpu.pc = _cpu.pc + 1; },
        0x0B1 => { _cpu.bus = _cpu.a; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        0x0B0 => { _cpu.bus = _cpu.b; _cpu.ram[_cpu.mar as usize] = _cpu.bus; },
        0x2B0 => { _cpu.bus = _cpu.a; _cpu.ram[(_cpu.sp + 128) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        0xAE1 => { _cpu.bus = _cpu.rom[_cpu.mar as usize]; _cpu.ram[(_cpu.sp + 128)  as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; _cpu.pc = _cpu.pc + 1; },
        0x454 => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp + 128) as usize]; _cpu.a = _cpu.bus; },
        0x2A0 => { _cpu.bus = _cpu.b; _cpu.ram[(_cpu.sp + 128) as usize] = _cpu.bus; _cpu.sp = _cpu.sp + 1; },
        0x452 => { _cpu.sp = _cpu.sp - 1; _cpu.bus = _cpu.ram[(_cpu.sp + 128) as usize]; _cpu.b = _cpu.bus; },
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
    for i in 0..255 {
		// any flags state
		for j in 0..4 {
			_cpu.eeprom[(j << 11) | i] = 0x07C; // (PC out, MAR in) for all instructions at microstep 0 --> X X 000 XXXX XXXX
			_cpu.eeprom[(j << 11) | (0x1 << 8) | i] = 0x069; // (ROM out, IR in, PC inc) for all instructions at microstep 1 --> --> X X 001 XXXX XXXX
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

	// LOAD A OPCODE 0000 0001
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x01] = 0x07C; // (PC out, MAR in)			X X 010 0000 0001
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x01] = 0x06D; // (ROM out, MAR in, pc inc) X X 011 0000 0001
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x01] = 0x05A; // (RAM out, MDR in)			X X 100 0000 0001
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x01] = 0x044; // (MDR out, A in)			X X 101 0000 0001
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x01] = 0x000; //							X X 110 0000 0001
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x01] = 0x000; //							X X 111 0000 0001
	}

	// LOAD B OPCODE 0000 0010
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x02] = 0x07C; // (PC out, MAR in)
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x02] = 0x06D; // (ROM out, MAR in, pc inc)
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x02] = 0x05A; // (RAM out, MDR in)
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x02] = 0x042; // (MDR out, B in)
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x02] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x02] = 0x000;
	}

	// ADD OPCODE 0000 0011
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x03] = 0x014; // (ALU out, A in)
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x03] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x03] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x03] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x03] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x03] = 0x000;
	}

	// SUB OPCODE 0000 0111
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x07] = 0x314; // (SUB, ALU out, A in)
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x07] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x07] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x07] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x07] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x07] = 0x000;
	}

	// LOAD IMMEDIATE A OPCODE 0000 0100
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x04] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x04] = 0x065; // ROM out, A in, PC inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x04] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x04] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x04] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x04] = 0x000;
	}

	// LOAD IMMEDIATE B OPCODE 0000 0101
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x05] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x05] = 0x063; // ROM out, B in, PC inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x05] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x05] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x05] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x05] = 0x000;
	}

	// PUSH (from register A) OPCODE 0000 1000
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x08] = 0x2B0; // A out, RAM in, SP inc
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x08] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x08] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x08] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x08] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x08] = 0x000;
	}

	// POP (to register A) OPCODE 0000 1001
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x09] = 0x454; // RAM out, A in, SP dec
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x09] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x09] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x09] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x09] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x09] = 0x000;
	}

	// PUSH (from register B) OPCODE 0000 1100
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x0c] = 0x2A0; // B out, RAM in, SP inc
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x0c] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x0c] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x0c] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x0c] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x0c] = 0x000;
	}

	// POP (to register B) OPCODE 0000 1101
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x0d] = 0x452; // RAM out, B in, SP dec
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x0d] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x0d] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x0d] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x0d] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x0d] = 0x000;
	}

	// PUSH (immediate) OPCODE 0000 1010
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x0A] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x0A] = 0xAE1; // ROM out, RAM in, PC inc, SP inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x0A] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x0A] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x0A] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x0A] = 0x000;
	}

	// STORE A OPCODE 0000 0110
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x06] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x06] = 0x06d; // ROM out, MAR in, PC inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x06] = 0x0B1; // A out, RAM in
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x06] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x06] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x06] = 0x000;
	}

	// STORE A OPCODE 0000 1110
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x0e] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x0e] = 0x06d; // ROM out, MAR in, PC inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x0e] = 0x0B0; // A out, RAM in
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x0e] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x0e] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x0e] = 0x000;
	}

	// JUMP OPCODE 0001 0000
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x10] = 0x07D; // PC out, MAR in, PC inc
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x10] = 0x06e; // ROM out, PC in
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x10] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x10] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x10] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x10] = 0x000;
	}

	// JUMP EQUAL ZERO OPCODE 0001 0001
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x11] = 0x07D; // PC out, MAR in, PC inc
		if i == 2 || i == 3 {
			_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x11] = 0x06e; // ROM out, PC in
		}
		else {
			_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x11] = 0x000; // do nothing when the zero flag is not set --> 0 X 011 0001 0010
		}
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x11] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x11] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x11] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x11] = 0x000;
	}

	// JUMP NOT EQUAL ZERO OPCODE 0001 0010
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x12] = 0x07D; // PC out, MAR in, PC inc
		if i == 0 || i == 1 {
			_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x12] = 0x06e; // ROM out, PC in		
		}
		else {
			_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x12] = 0x000; // do nothing when the zero flag is set --> 1 X 011 0001 0010
		}
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x12] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x12] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x12] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x12] = 0x000;
	}

	// OUT OPCODE 1000 0000
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x80] = 0x034; // A out, C in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x80] = 0x000;
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x80] = 0x000;
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x80] = 0x000;
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x80] = 0x000;
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x80] = 0x000;
	}
}

fn create_cpu() -> CPU {
    let mut _cpu = CPU {
        pc: 0,
        a: 0,
        b: 0,
        c: 0,
        alu: 0,
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
    };
    load_eeprom(&mut _cpu);
    return _cpu;
}

fn main() {
 
    let mut _cpu = create_cpu();

    _cpu.rom[0] = 0x04;
    _cpu.rom[1] = 0x08;
    _cpu.rom[2] = 0x05;
    _cpu.rom[3] = 0x08;
    _cpu.rom[4] = 0x03;
    _cpu.rom[5] = 0x80;
    _cpu.rom[6] = 0xFF;

    execute_program(&mut _cpu);
}