use std::fs::File;
use std::io::Read;


#[derive(Debug)]
pub struct Flags {
    pub zero: u8, //Zero flag
    pub n: u8, //Subtraction flag (BCD)
    pub h: u8, //Half carry flag (BCD)
    pub carry: u8, //Carry flag
}

impl Flags {
    fn new() -> Self {
        Flags {
            zero: 0,
            n: 0,
            h: 0,
            carry: 0,
        }
    }
}

#[derive(Debug)]
pub struct Chip {
    pub a: u8, //Upper bits of AF, the Accumulator
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub instr: u8, //Current instruction
    pub sp: u16, //Stack pointer
    pub pc: u16, //Program counter
    pub byte2: u8,
    pub byte3: u8,
    pub flags: Flags, //Lower bits of AF, Flags register
    pub rom_bank_0: [u8; 16384],
    pub rom_bank_n: [u8; 16384],
    pub external_ram_bank_n: [u8; 8192],
    pub wram_bank_0: [u8; 4096],
    pub wram_bank_n: [u8; 4096],
    pub rom: Vec<u8>,

}

impl Chip {
    pub fn new() -> Self {
        Chip {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            instr: 0,
            sp: 0,
            pc: 0,
            byte2: 0,
            byte3: 0,
            flags: Flags::new(),
            rom_bank_0: [0; 16384],
            rom_bank_n: [0; 16384],
            external_ram_bank_n: [00; 8192],
            wram_bank_0: [0; 4096],
            wram_bank_n: [0; 4096],
            rom: Vec::new(),
        }
    }
    pub fn load_rom(&mut self, rom_path: &str) {
        let mut rom = File::open(rom_path).unwrap_or_else(|_err| panic!("Valid ROM needed!"));
        rom.read_to_end(&mut self.rom).unwrap_or_else(|_err| panic!("Error reading ROM"));

	for i in 0..self.rom_bank_0.len() {
            self.rom_bank_0[i] = self.rom.get(i).unwrap().to_owned();
        }
        self.pc = 0x0100;
    }
    pub fn write_memory(&mut self, pc: u16, value: u8) {
        match pc {
            0x0000..=0x3fff => { println!("Writing: ROM Bank 0"); self.rom_bank_0[pc as usize] = value  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { println!("Writing: ROM Bank 1"); self.rom_bank_n[pc as usize - 0x4000] = value },
            0x8000..=0x9fff => {  },
            0xa000..=0xbfff => {  },
            0xc000..=0xcfff => {  },
            0xd000..=0xdfff => {  },
            0xfe00..=0xfe9f => {  },
            0xff00..=0xff7f => {  },
            0xff80..=0xfffe => {  },
            0xffff..=0xffff => {  },
            _ => {  }
        };
    }
    pub fn read_memory(&self, pc: u16) -> u8 {
        let address = match pc {
            0x0000..=0x3fff => { println!("ROM Bank 0"); self.rom_bank_0[pc as usize]  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { println!("ROM Bank 1"); self.rom_bank_n[pc as usize - 0x4000] },
            0x8000..=0x9fff => { 0x0 },
            0xa000..=0xbfff => { 0x0 },
            0xc000..=0xcfff => { 0x0 },
            0xd000..=0xdfff => { 0x0 },
            0xfe00..=0xfe9f => { 0x0 },
            0xff00..=0xff7f => { 0x0 },
            0xff80..=0xfffe => { 0x0 },
            0xffff..=0xffff => { 0x0 },
            _ => { 0xff }
        };
        address
    }
    pub fn fetch(&mut self) {
        self.instr = self.read_memory(self.pc);
        let arg1 = self.read_memory(self.pc + 1);
        let arg2 = self.read_memory(self.pc + 2);
        self.byte2 = arg1;
        self.byte3 = arg2;
        println!("Instr: 0x{:02X}", self.instr);
        self.pc += 1;
        println!("0x{:02X?}", &self.rom_bank_0[self.pc as usize..(self.pc + 10) as usize]);
    }

    //Lookup the u8 register table to get the u16 address we want to look at. We are constructing the address
    //to read/write from/to based on this table
    fn get_r16_register_address(&self, register_code: u8) -> u16 {
        //Example
        // reg b = 0x00ca
        // reg c = 0x00fe
        // reg b << 8 = 0xca00
        // bitwise OR with reg c results in 0xcafe
        match register_code {
            0b001 => { (self.b as u16) << 8 | self.c as u16 },
            0b011 => {  (self.d as u16) << 8 | self.e as u16 },
            0b101 => {(self.h as u16) << 8 | self.l as u16 },
            0b111 => { self.sp },
            _ => { panic!("Cannot get register address: Unknown register code {}", register_code) }
        }
    }


    fn get_r8_register(&mut self, register_code: u8) -> u8 {
        match register_code {
            0 => { self.b },
            1 => { self.c },
            2 => { self.d },
            3 => { self.e },
            4 => { self.h },
            5 => { self.l },
            6 => { let memory_address = self.get_r16_register_memory_address(0b101); self.read_memory(memory_address) }, //[HL]
            7 => { self.a },
            _ => { panic!("Cannot get register code: Unknown register code {}", register_code) }

        }
    }

    fn set_r8_register(&mut self, register_code: u8, value: u8) {
        match register_code {
            0 => { self.b = value },
            1 => { self.c = value },
            2 => { self.d = value },
            3 => { self.e = value },
            4 => { self.h = value },
            5 => { self.l = value },
            6 => { let memory_address = self.get_r16_register_memory_address(0b101); self.write_memory(memory_address, value) }, //[HL]
            7 => { self.a = value },
            _ => { panic!("Cannot get register code: Unknown register code {}", register_code) }

        }
    }

    // Write a 16 bit value to a pair of registers based on the usual pairing
    fn set_r16_register_address(&mut self, register_code: u8, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;
        match register_code {
            0b001 => { self.b = high; self.c = low; },
            0b011 => {  self.d = high; self.e = low; },
            0b101 => { self.h = high; self.l = low; },
            0b111 => { self.sp = value; },
            _ => { panic!("Cannot set register address: Unknown register code {}", register_code) }
        };
    }
    // Get the memory address pointed to by a register pair. This is basically the r16mem table
    // This returns a 16 bit memory address
    fn get_r16_register_memory_address(&mut self, register_code: u8) -> u16 {

        match register_code {
            0b001 => { (self.b as u16) << 8 | self.c as u16 },
            0b011 => {  (self.d as u16) << 8 | self.e as u16 },
            0b101 => {
                let hl = (self.h as u16) << 8 | self.l as u16; //Get current HL value
                self.set_r16_register_address(0b101, hl+1); //Increment HL value and set it back to HL (ptr++)
                hl //Return HL
            },
            0b111 => {
                let hl = (self.h as u16) << 8 | self.l as u16; //Get current HL value
                self.set_r16_register_address(0b101, hl-1); //Decrement HL value and set it back to HL (ptr--)
                hl //Return HL
            },
            _ => { panic!("Cannot get register address: Unknown register code {}", register_code) }
        }
    }
   
    // Return the half-carry and carry of adding two u8s
    fn check_carry_add_u8(&self, lhs: u8, rhs: u8) -> (bool, bool) {
        let (_, carry) = lhs.overflowing_add(rhs);
        let half_carry = ((lhs & 0xf).wrapping_add(rhs & 0xf)) & 0x10 == 0x10;
        return (half_carry, carry);
    }
    // Return the half-carry and carry of subtracting two u8s
    fn check_carry_sub_u8(&self, lhs: u8, rhs: u8) -> (bool, bool) {
        let (_, carry) = lhs.overflowing_sub(rhs);
        let half_carry = ((lhs & 0xf).wrapping_sub(rhs & 0xf)) & 0x10 == 0x10; //NOTE: This might need a rework
        return (half_carry, carry);
    }

    pub fn execute(&mut self) {
        let mut next = 0;

        println!("{:b}",  self.instr);
        let oct1 = (self.instr & 0b11000000) >> 6;
        let oct2 = (self.instr & 0b00111000) >> 3;
        let oct3 = self.instr & 0b00000111;
        println!("{:b} {:b} {:b}", oct1, oct2, oct3);

        match (oct1, oct2, oct3) {
            (0b11, 0b001, 0b001) => {  }, //CB-prefixed instructions
            (0b00, 0b000, 0b000) => { println!("NOOP") }, //noop
            (0b00, 0b001, 0b000) => { self.ld_u16_sp() }, //LD (u16), SP
            (0b00, 0b010, 0b000) => { println!("STOP") }, //STOP
            (0b00, 0b011, 0b000) => { self.jr() }, //JR
            (0b00, 0b100..=0b111, 0b000) => { self.jr_cond(oct2) }, //JR conditonal
            (0b00,0b000|0b010|0b100|0b110, 0b001) => { self.ld_r16_u16(oct2) }, //LD r16, u16
            (0b00,0b001|0b011|0b101|0b111, 0b001) => { self.add_hl_r16(oct2) }, //ADD HL, r16
            (0b00,0b000|0b010|0b100|0b110, 0b010) => { self.ld_r16_addr_a(oct2) }, //LD (r16), A
            (0b00,0b001|0b011|0b101|0b111, 0b010) => { self.ld_a_r16_addr(oct2) }, //LD A, (r16)
            (0b00,0b000|0b010|0b100|0b110, 0b011) => { self.inc_r16(oct2) }, //INC r16
            (0b00,0b001|0b011|0b101|0b111, 0b011) => { self.dec_r16(oct2) }, //DEC r16
            (0b00, r8, 0b100) => { self.inc_r8(r8) }, //INC r8
            (0b00, r8, 0b101) => { self.dec_r8(r8) }, //DEC r8
            (0b00, r8, 0b110) => { self.ld_r8_n8(r8) }, //LD r8, u8
            (0b00, opcode, 0b111) => { self.special_opcodes(opcode) }, //Opcode grp 1
            (0b01, 0b110, 0b110) => { self.halt() }, //HALT
            (0b01, dst_r8, src_r8) => { self.ld_r8_r8(src_r8, dst_r8) }, //LD r8, r8
            (0b10, op, r8) => { self.alu_a_r8(op, r8);  }, //ALU A, r8
            (0b11, 0b000..=0b011, 0b000) => {  }, //RET condition
            (0b11, 0b100, 0b000) => {  }, //LD (FF00 + u8), A
            (0b11, 0b101, 0b000) => {  }, //ADD SP, i8
            (0b11, 0b110, 0b000) => {  }, //LD A, (FF00 + u8)
            (0b11, 0b111, 0b000) => {  }, //LD HL, SP + i8
            (0b11, 0b000|0b010|0b100|0b110, 0b001) => {  }, //POP r16
            (0b11, 0b001|0b011|0b101|0b111, 0b001) => {  }, //0: RET, 1: RETI, 2: JP HL, 3: LD SP, HL
            (0b11, 0b000..=0b011, 0b010) => {  }, //JP
            (0b11, 0b100, 0b010) => {  }, //LD (FF00 + C), A
            (0b11, 0b101, 0b010) => {  }, //LD (u16), A
            (0b11, 0b110, 0b010) => {  }, //LD A, (FF00 + C)
            (0b11, 0b111, 0b010) => {  }, //LD A, (u16)
            (0b11, opcode, 0b011) => {  }, //0: JP u16, 1: CB prefix, 6: DI, 7: EI
            (0b11, 0b000..=0b011, 0b100) => {  }, //CALL condition
            (0b11, 0b000|0b010|0b100|0b110, 0b101) => {  }, //PUSH r16
            (0b11, 0b001, 0b101) => {  }, //CALL u16
            (0b11, opcode, 0b110) => {  }, //ALU a, u8
            (0b11, exp, 0b111) => {  }, //RST
            _ => { println!("Error: 0x{:02X} not implemented!", self.instr); std::process::exit(1); },
        }

        self.pc += next;
    }


    // Load value from dst_r8 into src_r8. When called as LD r1 r2, this method
    // is called as ld_r8_r8(r2, r1) (Notice the inversion)
    fn ld_r8_r8(&mut self, src_r8: u8, dst_r8: u8) {
        let value = self.get_r8_register(src_r8);
        self.set_r8_register(dst_r8, value);

    }

    //TODO - Respond to interrupt
    fn halt(&mut self) {

    }

    //TODO
    fn special_opcodes(&mut self, opcode: u8) {
    }


    //Increment value in register r8 by 1
    fn inc_r8(&mut self, register_lookup: u8) {
        let register = self.get_r8_register(register_lookup);
        let sum = register.wrapping_add(1);
        if sum == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        let (half_carry, _) = self.check_carry_add_u8(register, 1);
        if half_carry { self.flags.h = 1 }
        self.set_r8_register(register_lookup, sum);
    }

    //Decrement value in register r8 by 1
    fn dec_r8(&mut self, register_lookup: u8) {
        let register = self.get_r8_register(register_lookup);
        let sum = register.wrapping_sub(1);
        if sum == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
        let (half_carry, _) = self.check_carry_sub_u8(register, 1);
        if half_carry{ self.flags.h = 1 }
        self.set_r8_register(register_lookup, sum);
    }

    //Increment value in register r16 by 1
    fn inc_r16(&mut self, register_lookup: u8) {
        let mut register = self.get_r16_register_address(register_lookup);
        register = register.wrapping_add(1);
        self.set_r16_register_address(register_lookup, register);
    }

    //Decrement value in register r16 by 1
    fn dec_r16(&mut self, register_lookup: u8) {
        let mut register = self.get_r16_register_address(register_lookup);
        register = register.wrapping_sub(1);
        self.set_r16_register_address(register_lookup, register);
    }

    //Load value  n8 into register r8
    fn ld_r8_n8(&mut self, r8: u8) {
        self.set_r8_register(r8, self.byte2);
    }

    //Load value pointed in memory by r16 register pair into register A
    fn ld_a_r16_addr(&mut self, register_lookup: u8) {
        let memory_address = self.get_r16_register_memory_address(register_lookup);
        self.a = self.read_memory(memory_address);
    }
    // Load the 8 bit value in register A to the memory address pointed by the register from the
    // table
    fn ld_r16_addr_a(&mut self, register_lookup: u8) {
        let memory_address = self.get_r16_register_memory_address(register_lookup);
        self.write_memory(memory_address, self.a);
    }

    //Add register r16 value to HL
    fn add_hl_r16(&mut self, register_lookup: u8) {
        let r16 = self.get_r16_register_address(register_lookup);
        let hl = self.get_r16_register_address(0b101); //0b101 == HL register pair
        let (sum, overflow_high) = r16.overflowing_add(hl);

        //Additions reset the n flag
        self.flags.n = 0;
        //Check 11th to 12th bit overflow
        if (r16 & 0xfff).overflowing_add(hl & 0xfff).1 as u8 == 1 { self.flags.h = 1 }
        //Check 15th to 16th bit overflow
        if overflow_high { self.flags.carry = 1 }

        self.set_r16_register_address(0b101, sum);
    }

    //Load value u16 into register r16
    fn ld_r16_u16(&mut self, register_lookup: u8) {
        let address = (self.byte2 as u16) << 8 | self.byte3 as u16; 
        self.set_r16_register_address(register_lookup, address);
    }

    //Conditional Jump
    fn jr_cond(&mut self, condition: u8) {

        let should_execute = match condition {
            0b100 => { self.flags.zero != 0 },
            0b101 => { self.flags.zero == 0},
            0b110 => { self.flags.carry != 0 },
            0b111 => { self.flags.carry == 0 },
            _c => { panic!("Unknown condition {}", _c) }
        };
        if should_execute {
            let offset: i8 = self.byte2 as i8;
            self.pc = self.pc.wrapping_add_signed(offset.into());
        }
    }

    //Unconditional relative jump
    fn jr(&mut self) {
        let offset: i8 = self.byte2 as i8;
        self.pc = self.pc.wrapping_add_signed(offset.into());
    }

    //Store SP lower at address u16, and SP upper at address u16 + 1
    fn ld_u16_sp(&mut self) {
        let address: u16 = (self.byte2 as u16) << 8 | self.byte3 as u16;
        self.write_memory(address, (self.sp & 0xff) as u8);
        self.write_memory(address + 1, (self.sp >> 8) as u8);
    }

    //All math based operations are processed here
    fn alu_a_r8(&mut self, opcode: u8, r8: u8) {
        match opcode {
            0b000 => { self.add_a_r8(r8) },
            0b001 => { self.adc_a_r8(r8) },
            0b010 => { self.sub_a_r8(r8) },
            0b011 => { self.sbc_a_r8(r8) },
            0b100 => { self.and_a_r8(r8) },
            0b101 => { self.xor_a_r8(r8) },
            0b110 => { self.or_a_r8(r8) },
            0b111 => { self.cp_a_r8(r8) },
            _ => { panic!("Invalid ALU A R8 Operation: opcode: {}, register: {}", opcode, r8)}
        }
    }

    //Add the value in r8 to the a register
    fn add_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8);
        let (half_carry, carry) = self.check_carry_add_u8(self.a, reg_val);
        self.a = self.a.wrapping_add(reg_val);
        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
    }
    //Add the value in r8 to the a register, along with the value of the carry flag
    fn adc_a_r8(&mut self, r8: u8) {
        let sum = self.get_r8_register(r8).wrapping_add(self.flags.carry);
        let (half_carry, carry) = self.check_carry_add_u8(self.a, sum);
        self.a = self.a.wrapping_add(sum);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;

    }
    //Sub the value in r8 from the a register
    fn sub_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8);
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, reg_val);
        self.a = self.a.wrapping_sub(reg_val);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }
    //Sub the value in r8 from the a register along with the value of the carry flag
    fn sbc_a_r8(&mut self, r8: u8) {
        let sum = self.get_r8_register(r8).wrapping_add(self.flags.carry);
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, sum);
        self.a = self.a.wrapping_sub(sum);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }

    //Bitwise AND between r8 value and A
    fn and_a_r8(&mut self, r8: u8) {
        self.a = self.a & self.get_r8_register(r8);
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 1;
        self.flags.carry = 0;
    }

    //Bitwise XOR between r8 value and A
    fn xor_a_r8(&mut self, r8: u8) {
        self.a = self.a ^ self.get_r8_register(r8);
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 0;
        self.flags.carry = 0;
    }

    //Bitwise OR between r8 value and A
    fn or_a_r8(&mut self, r8: u8) {
        self.a = self.a | self.get_r8_register(r8);
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 0;
        self.flags.carry = 0;
    }

    //Subtract value in r8 from A, but don't store the result, only set flags
    fn cp_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8);
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, reg_val);
        let tmp = self.a.wrapping_sub(reg_val);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if tmp == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }
}
