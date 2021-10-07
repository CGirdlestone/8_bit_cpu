
struct CPU {
	pc: u8, // program counter
	a: u8, // a(ccumulator) register
	b: u8, // b register (general purpose)
	c: u8, // c register (output)
	alu: u8, // arithmetic logic unit
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

	// set carry flag
	*flags = *flags | (carry << 0);
	
	// set zero flag
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
	// all opcodes must fetch instruction and increment program counter
	for i in 0..255 {
		// any flags state
		for j in 0..4 {
			_cpu.eeprom[(j << 11) | i] = 0x07C; // (PC out, MAR in) for all instructions at microstep 0 --> X X 000 XXXX XXXX
			_cpu.eeprom[(j << 11) | (0x1 << 8) | i] = 0x069; // (ROM out, IR in, PC inc) for all instructions at microstep 1 --> X X 001 XXXX XXXX
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
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x01] = 0x07C; // (PC out, MAR in)          X X 010 0000 0001
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x01] = 0x06D; // (ROM out, MAR in, pc inc) X X 011 0000 0001
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x01] = 0x05A; // (RAM out, MDR in)         X X 100 0000 0001
		_cpu.eeprom[(i << 11) | (0x5 << 8) | 0x01] = 0x044; // (MDR out, A in)           X X 101 0000 0001
		_cpu.eeprom[(i << 11) | (0x6 << 8) | 0x01] = 0x000; //                           X X 110 0000 0001
		_cpu.eeprom[(i << 11) | (0x7 << 8) | 0x01] = 0x000; //                           X X 111 0000 0001
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

	// STORE B OPCODE 0000 1110
	for i in 0..4 {
		_cpu.eeprom[(i << 11) | (0x2 << 8) | 0x0e] = 0x07C; // PC out, MAR in
		_cpu.eeprom[(i << 11) | (0x3 << 8) | 0x0e] = 0x06d; // ROM out, MAR in, PC inc
		_cpu.eeprom[(i << 11) | (0x4 << 8) | 0x0e] = 0x0B0; // B out, RAM in
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

	// set up variables
	_cpu.rom[0] = 0x04; 	//	load A immediate
	_cpu.rom[1] = 0x00; 	//	value 0
	_cpu.rom[2] = 0x80;		//	output first term 
	_cpu.rom[3] = 0x06; 	//	store A
	_cpu.rom[4] = 0x00; 	//	memory location 0x00 - 'a'
	_cpu.rom[5] = 0x06;		//	store A
	_cpu.rom[6] = 0x02;		//	memory location 0x02 - 'b'
	_cpu.rom[7] = 0x04;		//	load A immediate
	_cpu.rom[8] = 0x01;		// 	value 1
	_cpu.rom[9] = 0x80;		//	output second term
	_cpu.rom[10] = 0x06;	//	store A
	_cpu.rom[11] = 0x01; 	// 	memory location 0x01 - 'c'
	_cpu.rom[12] = 0x04;	//	load A immediate
	_cpu.rom[13] = 0x0a;	//	value 10 - number of iterations
	_cpu.rom[14] = 0x06;	//	store A
	_cpu.rom[15] = 0x03;	//	memory location 0x03 - 'n'
	
	// add terms
	_cpu.rom[16] = 0x01; 	// load A from memory
	_cpu.rom[17] = 0x00;	// memory location 0x00 'a'
	_cpu.rom[18] = 0x02;	// load B from memory
	_cpu.rom[19] = 0x01;	// memory location 0x01 'b'
	_cpu.rom[20] = 0x03;	// add B to A
	_cpu.rom[21] = 0x06;	// store A
	_cpu.rom[22] = 0x02;	// memory location 0x02 'c'
	_cpu.rom[23] = 0x0e;	// store B - currently holds the value in variable 'b'
	_cpu.rom[24] = 0x00;	// memory location 0x00 'a'
	_cpu.rom[25] = 0x06;	// store A
	_cpu.rom[26] = 0x01;	// memory location 0x01 'b'

	// output
	_cpu.rom[27] = 0x80;	// output term

	// decrement counter
	_cpu.rom[28] = 0x01;	// load A from memory
	_cpu.rom[29] = 0x03;	// memory location 0x03 'n'
	_cpu.rom[30] = 0x05;	// load B immediate
	_cpu.rom[31] = 0x07;	// sub B from A
	_cpu.rom[32] = 0x06;	// store A
	_cpu.rom[33] = 0x03;	// memory location 0x03 'n'

	// check if counter is zero
	_cpu.rom[34] = 0x12;	// jump not zero
	_cpu.rom[35] = 0x0e;	// memory location to jump to	

	// end program
	_cpu.rom[36] = 0xFF;

	execute_program(&mut _cpu);
}