use std::fs::File;
use std::io::Read;


#[derive(Debug)]
enum REGISTER8 {
    UD = -1,
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HL = 6,
    A = 7,
}

impl From<u8> for REGISTER8 {
    fn from(value: u8) -> Self {
        match value {
            0 => REGISTER8::B,
            1 => REGISTER8::C,
            2 => REGISTER8::D,
            3 => REGISTER8::E,
            4 => REGISTER8::H,
            5 => REGISTER8::L,
            6 => REGISTER8::HL,
            7 => REGISTER8::B,
            _ => REGISTER8::UD
        }
    }
}

#[derive(Debug)]
enum REGISTER16 {
    UD = -1,
    BC = 0b001,
    DE = 0b011,
    HL = 0b101,
    SP = 0b111,
}

impl From<u8> for REGISTER16 {
    fn from(value: u8) -> Self {
        match value {
            0b000 => REGISTER16::BC,
            0b010 => REGISTER16::DE,
            0b011 => REGISTER16::HL,
            0b100 => REGISTER16::SP,
            _ => REGISTER16::UD
        }
    }
}

#[derive(Debug)]
enum REGISTER16MEM {
    UD = -1,
    BC = 0b001,
    DE = 0b011,
    HLI = 0b101, //HL+
    HLD = 0b111, //HL-
}

impl From<u8> for REGISTER16MEM {
    fn from(value: u8) -> Self {
        match value {
            0b000 => REGISTER16MEM::BC,
            0b010 => REGISTER16MEM::DE,
            0b011 => REGISTER16MEM::HLI,
            0b100 => REGISTER16MEM::HLD,
            _ => REGISTER16MEM::UD
        }
    }
}

#[derive(Debug)]
enum REGISTER16STK {
    UD = -1,
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3,
}

impl From<u8> for REGISTER16STK {
    fn from(value: u8) -> Self {
        match value {
            0 => REGISTER16STK::BC,
            1 => REGISTER16STK::DE,
            2 => REGISTER16STK::HL,
            3 => REGISTER16STK::AF,
            _ => REGISTER16STK::UD
        }
    }
}

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

    fn get_cond(&self, cond: u8) -> u16 {
        match cond {
            0 => { (self.n as u16) << 8 | self.zero as u16  }
            1 => { self.zero as u16 }
            2 => { (self.n as u16) << 8 | self.carry as u16 }
            3 => { self.carry as u16 }
            _ => { panic!("Invalid Condition for Flags!") }
            
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
    pub ime: u8, // IME (Interrupt) flag
    pub rom_bank_0: [u8; 16384],
    pub rom_bank_n: [u8; 16384],
    pub vram: [u8; 8192],
    pub external_ram_bank_n: [u8; 8192],
    pub wram_bank_0: [u8; 4096],
    pub wram_bank_n: [u8; 4096],
    pub object_attribute_memory: [u8; 160],
    pub io_registers: [u8; 128],
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
            ime: 0,
            rom_bank_0: [0; 16384],
            rom_bank_n: [0; 16384],
            vram: [0; 8192],
            external_ram_bank_n: [00; 8192],
            wram_bank_0: [0; 4096],
            wram_bank_n: [0; 4096],
            object_attribute_memory: [0; 160],
            io_registers: [0; 128],
            rom: Vec::new(),
        }
    }

    fn dump(&self) {
        println!("Instruction: 0x{:02X}, PC: 0x{:04X}, SP: 0x{:04X}", self.instr, self.pc, self.sp);
        println!("[Optional] Byte 2 0x{:02X}, Byte 3: 0x{:02X}", self.byte2, self.byte3);
    }
    pub fn load_rom(&mut self, rom_path: &str) {
        let mut rom = File::open(rom_path).unwrap_or_else(|_err| panic!("Valid ROM needed!"));
        rom.read_to_end(&mut self.rom).unwrap_or_else(|_err| panic!("Error reading ROM"));

	for i in 0..self.rom_bank_0.len() {
            let item = self.rom.get(i);
            match item {
                Some(byte) => { self.rom_bank_0[i] = byte.to_owned() },
                None => { return; }
            }
        }
	for i in 0..self.rom_bank_n.len() {
            let item = self.rom.get(i + self.rom_bank_0.len());
            match item {
                Some(byte) => { self.rom_bank_n[i] = byte.to_owned() },
                None => { return; }
            }
        }
        self.pc = 0x0100;
    }
    /// Write to memory
    /// Write a 8 bit value to memory addressed in 16 bits
    /// The function decides which bank to write to based on the address value
    /// # Arguments
    ///
    /// * `address` - Address to write to
    /// * `value` - Value to write to address
    pub fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3fff => { println!("Writing: ROM Bank 0: 0x{:04X}", address); self.rom_bank_0[address as usize] = value  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { println!("Writing: ROM Bank 1: 0x{:04X}", address); self.rom_bank_n[address as usize - 0x4000] = value },
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
    /// Read from memory
    /// Same as write_memory, but read from the correct rom bank instead
    /// # Arguments
    ///
    /// * `address` - 16 bit address to access
    ///
    pub fn read_memory(&self, address: u16) -> u8 {
        let address = match address {
            0x0000..=0x3fff => { println!("Reading ROM Bank 0: 0x{:04X}", address); self.rom_bank_0[address as usize]  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { println!("Reading ROM Bank 1: 0x{:04X}", address); self.rom_bank_n[address as usize - 0x4000] },
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
        println!("==========================");
        self.instr = self.read_memory(self.pc);
        let arg1 = self.read_memory(self.pc + 1);
        let arg2 = self.read_memory(self.pc + 2);
        self.byte2 = arg1;
        self.byte3 = arg2;
        self.dump();
        //println!("0x{:02X?}", &self.rom_bank_0[self.pc as usize..(self.pc + 10) as usize]);
    }


    /// Return the value of a register
    ///
    /// # Arguments
    ///
    /// * `register_code` - The code of the register to access
    ///
    fn get_r8_register(&mut self, register_code: REGISTER8) -> u8 {
        match register_code {
            REGISTER8::B => { self.b },
            REGISTER8::C => { self.c },
            REGISTER8::D => { self.d },
            REGISTER8::E => { self.e },
            REGISTER8::H => { self.h },
            REGISTER8::L => { self.l },
            REGISTER8::HL=> { let memory_address = self.get_r16_register(REGISTER16::HL); self.read_memory(memory_address) }, //[HL]
            REGISTER8::A => { self.a },
            _ => { panic!("Cannot get register code: Unknown register code {:?}", register_code) }

        }
    }

    /// Set a register's value
    ///
    /// # Arguments
    ///
    /// * `register_code` - The register to change
    /// * `value` - New value of register
    ///
    fn set_r8_register(&mut self, register_code: REGISTER8, value: u8) {
        match register_code {
            REGISTER8::B => { self.b = value },
            REGISTER8::C => { self.c = value },
            REGISTER8::D => { self.d = value },
            REGISTER8::E => { self.e = value },
            REGISTER8::H => { self.h = value },
            REGISTER8::L => { self.l = value },
            REGISTER8::HL => { let memory_address = self.get_r16_register(REGISTER16::HL); self.write_memory(memory_address, value) }, //[HL]
            REGISTER8::A => { self.a = value },
            _ => { panic!("Cannot get register code: Unknown register code {:?}", register_code) }

        }
    }

    //Lookup the u8 register table to get the u16 address we want to look at. We are constructing the address
    //to read/write from/to based on this table
    fn get_r16_register(&self, register_code: REGISTER16) -> u16 {
        //Example
        // reg b = 0x00ca
        // reg c = 0x00fe
        // reg b << 8 = 0xca00
        // bitwise OR with reg c results in 0xcafe
        match register_code {
            REGISTER16::BC => { (self.b as u16) << 8 | self.c as u16 },
            REGISTER16::DE => {  (self.d as u16) << 8 | self.e as u16 },
            REGISTER16::HL => {(self.h as u16) << 8 | self.l as u16 },
            REGISTER16::SP => { self.sp },
            _ => { panic!("Cannot get register address: Unknown register code {:?}", register_code) }
        }
    }


    // Write a 16 bit value to a pair of registers based on the usual pairing
    fn set_r16_register(&mut self, register_code: REGISTER16, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;
        match register_code {
            REGISTER16::BC => { self.b = high; self.c = low; },
            REGISTER16::DE => {  self.d = high; self.e = low; },
            REGISTER16::HL => { self.h = high; self.l = low; },
            REGISTER16::SP => { self.sp = value; },
            _ => { panic!("Cannot set register address: Unknown register code {:?}", register_code) }
        };
    }
    // Get the memory address pointed to by a register pair. This is basically the r16mem table
    // This returns a 16 bit memory address
    fn get_r16mem_register(&mut self, register_code: REGISTER16MEM) -> u16 {

        match register_code {
            REGISTER16MEM::BC => { (self.b as u16) << 8 | self.c as u16 },
            REGISTER16MEM::DE => {  (self.d as u16) << 8 | self.e as u16 },
            REGISTER16MEM::HLI => {
                let hl = (self.h as u16) << 8 | self.l as u16; //Get current HL value
                self.set_r16_register(REGISTER16::HL, hl+1); //Increment HL value and set it back to HL (ptr++)
                hl //Return HL
            },
            REGISTER16MEM::HLD => {
                let hl = (self.h as u16) << 8 | self.l as u16; //Get current HL value
                self.set_r16_register(REGISTER16::HL, hl-1); //Decrement HL value and set it back to HL (ptr--)
                hl //Return HL
            },
            _ => { panic!("Cannot get register address: Unknown register code {:?}", register_code) }
        }
    }


    // Get the memory address pointed to by a register pair. This is basically the r16stk table
    // This returns a 16 bit memory address
    fn get_r16stk_register(&mut self, register_code: REGISTER16STK) -> u16 {

        match register_code {
            REGISTER16STK::BC => { (self.b as u16) << 8 | self.c as u16 },
            REGISTER16STK::DE => {  (self.d as u16) << 8 | self.e as u16 },
            REGISTER16STK::HL => { (self.h as u16) << 8 | self.l as u16 },
            REGISTER16STK::AF => {
                // We do this because in reality, the flags are just a single register F, and the bit shift
                // corresponds to the flag position in the register byte
                let f = self.flags.zero << 7 | self.flags.n << 6 | self.flags.h << 5 | self.flags.carry << 4;
                (self.a as u16) << 8 | f as u16
            },
            _ => { panic!("Cannot get register address: Unknown register code {:?}", register_code) }
        }
    }

    // Set the memory address pointed to by a register pair. This is basically the r16stk table
    // This returns a 16 bit memory address
    fn set_r16stk_register(&mut self, register_code: REGISTER16STK, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;
        match register_code {
            REGISTER16STK::BC => { self.b = high; self.c = low; },
            REGISTER16STK::DE => {  self.d = high; self.e = low; },
            REGISTER16STK::HL => { self.h = high; self.l = low; },
            REGISTER16STK::AF => {
                // We do this because in reality, the flags are just a single register F, and the bit shift
                // corresponds to the flag position in the register byte
                self.a = high;
                self.flags.zero = low & 0b10000000;
                self.flags.n = low & 0b01000000;
                self.flags.h = low & 0b00100000;
                self.flags.carry = low & 0b00010000;
            },
            _ => { panic!("Cannot get register address: Unknown register code {:?}", register_code) }
        }
    }
   
    // Return the half-carry and carry of adding two u8s
    fn check_carry_add_u8(&self, lhs: u8, rhs: u8) -> (bool, bool) {
        let (_, carry) = lhs.overflowing_add(rhs);
        let half_carry = ((lhs & 0xf).wrapping_add(rhs & 0xf)) & 0x10 == 0x10;
        return (half_carry, carry);
    }
    // Return the half-carry and carry of adding two i8s
    fn check_carry_add_i8(&self, lhs: i8, rhs: i8) -> (bool, bool) {
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

    pub fn execute_cb(&mut self) {
        println!("CB-Prefix: 0x{:02X}",  self.instr);
        let oct1 = (self.instr & 0b11000000) >> 6;
        let oct2 = (self.instr & 0b00111000) >> 3;
        let oct3 = self.instr & 0b00000111;
        println!("Octets: 0b{:03b} 0b{:03b} 0b{:03b}", oct1, oct2, oct3);

        match (oct1, oct2, oct3) {
            (0b00, opcode, r8) => { self.shift_rotate(opcode, r8)  },
            (0b01, bit, r8) => { self.bit(bit, r8) },
            (0b10, bit, r8) => { self.res(bit, r8) },
            (0b11, bit, r8) => { self.set(bit, r8) },
            _ => { println!("CB-Prefix Error: 0x{:02X} not implemented!", self.instr); std::process::exit(1); },
        }
    }

    pub fn execute(&mut self) {
        // println!("{:b}",  self.instr);
        // Handle CB-prefixed instructions
        if self.instr == 0xcb {
            // Move ahead from 0xcb
            self.pc += 1;
            self.execute_cb();
            return;
        }
        let oct1 = (self.instr & 0b11000000) >> 6;
        let oct2 = (self.instr & 0b00111000) >> 3;
        let oct3 = self.instr & 0b00000111;
         println!("Octets: 0b{:03b} 0b{:03b} 0b{:03b}", oct1, oct2, oct3);

        match (oct1, oct2, oct3) {
            (0b00, 0b000, 0b000) => { println!("NOOP"); self.noop() }, //noop
            (0b00, 0b001, 0b000) => { println!("ld_u16_sp"); self.ld_u16_sp() }, //LD (u16), SP
            (0b00, 0b010, 0b000) => { println!("STOP"); panic!("STOPPING!") }, //STOP
            (0b00, 0b011, 0b000) => { println!("jr"); self.jr() }, //JR
            (0b00, 0b100..=0b111, 0b000) => { println!("jr_cond"); self.jr_cond(oct2) }, //JR conditonal
            (0b00,0b000|0b010|0b100|0b110, 0b001) => { println!("ld_r16_u16"); self.ld_r16_u16(oct2) }, //LD r16, u16
            (0b00,0b001|0b011|0b101|0b111, 0b001) => { println!("add_hl_r16"); self.add_hl_r16(oct2) }, //ADD HL, r16
            (0b00,0b000|0b010|0b100|0b110, 0b010) => { println!("ld_r16_addr_a"); self.ld_r16_addr_a(oct2) }, //LD (r16), A
            (0b00,0b001|0b011|0b101|0b111, 0b010) => { println!("ld_a_r16_addr"); self.ld_a_r16_addr(oct2) }, //LD A, (r16)
            (0b00,0b000|0b010|0b100|0b110, 0b011) => { println!("inc_r16"); self.inc_r16(oct2) }, //INC r16
            (0b00,0b001|0b011|0b101|0b111, 0b011) => { println!("dec_r16"); self.dec_r16(oct2) }, //DEC r16
            (0b00, r8, 0b100) => { println!("inc_r8"); self.inc_r8(r8) }, //INC r8
            (0b00, r8, 0b101) => { println!("dec_r8"); self.dec_r8(r8) }, //DEC r8
            (0b00, r8, 0b110) => { println!("ld_r8_n8"); self.ld_r8_n8(r8) }, //LD r8, u8
            (0b00, opcode, 0b111) => { println!("special_opcodes"); self.special_opcodes(opcode) }, //Opcode grp 1
            (0b01, 0b110, 0b110) => { println!("halt"); self.halt() }, //HALT
            (0b01, dst_r8, src_r8) => { println!("ld_r8_r8"); self.ld_r8_r8(src_r8, dst_r8) }, //LD r8, r8
            (0b10, op, r8) => { println!("alu_a_r8"); self.alu_a_r8(op, r8);  }, //ALU A, r8
            (0b11, 0b000..=0b011, 0b000) => { println!("ret_cond"); self.ret_cond(oct2) }, //RET condition
            (0b11, 0b100, 0b000) => { println!("ldh_i16_a"); self.ldh_i16_a() }, //LD (FF00 + u8), A
            (0b11, 0b101, 0b000) => { println!("add_sp_i8"); self.add_sp_i8() }, //ADD SP, i8
            (0b11, 0b110, 0b000) => { println!("ldh_a_i16"); self.ldh_a_i16() }, //LD A, (FF00 + u8)
            (0b11, 0b111, 0b000) => { println!("ld_hl_sp_imm8"); self.ld_hl_sp_imm8() }, //LD HL, SP + i8
            (0b11, 0b000|0b010|0b100|0b110, 0b001) => { println!("pop_r16"); self.pop_r16(oct2) }, //POP r16
            (0b11, 0b001, 0b001) => { println!("ret"); self.ret() }, // RET
            (0b11, 0b011, 0b001) => { println!("reti"); self.reti() }, // RETI
            (0b11, 0b101, 0b001) => { println!("jp_hl"); self.jp_hl() }, // JP HL
            (0b11, 0b111, 0b001) => { println!("ld_sp_hl"); self.ld_sp_hl() }, // LD SP, HL
            (0b11, 0b000..=0b011, 0b010) => { println!("jp_cond"); self.jp_cond(oct2) }, //JP
            (0b11, 0b100, 0b010) => { println!("ldh_c_a"); self.ldh_c_a() }, //LD (FF00 + C), A
            (0b11, 0b101, 0b010) => { println!("ld_n16_a"); self.ld_n16_a() }, //LD (u16), A
            (0b11, 0b110, 0b010) => { println!("ldh_a_c"); self.ldh_a_c() }, //LD A, (FF00 + C)
            (0b11, 0b111, 0b010) => { println!("ld_a_n16"); self.ld_a_n16() }, //LD A, (u16)
            (0b11, 0b000, 0b011) => { println!("jp_u16"); self.jp_u16() }, //JP u16
            (0b11, 0b001, 0b011) => {  }, //CB prefix
            (0b11, 0b110, 0b011) => { println!("di"); self.di() }, //DI
            (0b11, 0b111, 0b011) => { println!("ei"); self.ei() }, //EI
            (0b11, 0b000..=0b011, 0b100) => { println!("call_cond"); self.call_cond(oct2) }, //CALL condition
            (0b11, 0b000|0b010|0b100|0b110, 0b101) => { println!("push_r16"); self.push_r16(oct2) }, //PUSH r16
            (0b11, 0b001, 0b101) => { println!("call"); self.call() }, //CALL u16
            (0b11, opcode, 0b110) => { println!("alu_a_u8"); self.alu_a_u8(opcode) }, //ALU a, u8
            (0b11, tgt, 0b111) => { println!("rst"); self.rst(tgt) }, //RST
            _ => { println!("Error: 0x{:02X} not implemented!", self.instr); std::process::exit(1); },
        }
    }

    // Move PC by inc, usually 1
    fn next(&mut self, inc: u16) {
        self.pc += inc;
    }

    fn noop(&mut self) {
        self.next(1);
    }

    // Set nth bit to zero in r8
    fn res(&mut self, bit: u8, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        reg = !(1 << bit) & reg;
        self.set_r8_register(r8.into(), reg);
    }

    // Set nth bit in r8
    fn set(&mut self, bit: u8, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        reg = (1 << bit) | reg;
        self.set_r8_register(r8.into(), reg);
    }

    // Test for nth bit value in r8
    fn bit(&mut self, bit: u8, r8: u8) {
        let reg = self.get_r8_register(r8.into());
        // Check nth bit of reg, and if it is zero, set zero flag
        if ((1 << bit) & reg) >> bit == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 1;
    }

    fn shift_rotate(&mut self, opcode: u8, r8: u8) {
        match opcode {
            0b000 => { self.rlc_r8(r8) },
            0b001 => { self.rrc_r8(r8) },
            0b010 => { self.rl_r8(r8) },
            0b011 => { self.rr_r8(r8) },
            0b100 => { self.sla_r8(r8) },
            0b101 => { self.sra_r8(r8) },
            0b110 => { self.swap_r8(r8) },
            0b111 => { self.srl_r8(r8) },
            _ => { panic!("Invalid opcode for shift_rotate: {}", opcode) }
        }
    }


    // Swap upper 4 bits of r8 with lower 4 bits
    fn swap_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        let high = (reg >> 4) as u8;
        let low = (reg & 0x0f) as u8;
        reg = (low << 4) | high;
        if reg == 0 {
            self.flags.zero = reg;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.flags.carry = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Shift r8 right logically
    fn srl_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        self.flags.carry = reg & 0b1;
        reg  = reg >> 1;
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Shift r8 right 
    fn sra_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        self.flags.carry = reg & 0b1;
        // Think of this as signed division by 2
        let bit = reg >> 7;
        reg  = (reg >> 1) | (bit << 7);
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Rotate register r8 right through carry flag (wrapping)
    fn rr_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        let carry = self.flags.carry;
        self.flags.carry = reg & 0b1;
        reg  = (reg >> 1) | (carry << 7);
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Rotate register r8 right
    fn rrc_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        self.flags.carry = reg & 0b1;
        // Move bit 0 to bit 7, since bit carry is bit 0
        reg  = (reg >> 1) | (self.flags.carry << 7);
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Shift r8 left
    fn sla_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        self.flags.carry = reg >> 7;
        reg = reg << 1;
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Rotate register r8 left through carry flag (wrapping)
    fn rl_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        let carry = self.flags.carry;
        self.flags.carry = reg >> 7;
        reg = (reg << 1) | carry;
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    // Rotate register r8 left
    fn rlc_r8(&mut self, r8: u8) {
        let mut reg = self.get_r8_register(r8.into());
        self.flags.carry = reg >> 7;
        // Move bit 7 to bit 0, since carry is now bit7
        reg = (reg << 1) | self.flags.carry;
        if reg == 0 {
            self.flags.zero = 1;
        }
        self.flags.n = 0;
        self.flags.h = 0;
        self.set_r8_register(r8.into(), reg);
    }

    fn rst(&mut self, tgt: u8) {
        // Target address 0x00exp000
        let address = (tgt * 8) as u16;
        // PC has already moved onto the next address
        let high = (self.pc >> 8) as u8;
        let low = (self.pc & 0xff) as u8;
        self.sp -= 1;
        self.write_memory(self.sp, high);
        self.sp -= 1;
        self.write_memory(self.sp, low);
        // JP u16
        self.pc = address;
    }

    //ALU A u8 -> Similar to ALU A r8, but instead of register, use next byte
    fn alu_a_u8(&mut self, opcode: u8) {
        let r8 = self.byte2;
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
        self.pc += 1;
        self.next(1);
    }

    // CALL if cond met
    fn call_cond(&mut self, cond: u8) {
        if self.flags.get_cond(cond) != 0 {
            self.call()
        }
    }

    // Save next address onto stack so that RET can pop it later
    fn call(&mut self) {
        let address = (self.byte3 as u16) << 8 | self.byte2 as u16; 
        self.pc += 2;
        // PC has already moved onto the next address
        let high = (self.pc >> 8) as u8;
        let low = (self.pc & 0xff) as u8;
        self.sp -= 1;
        self.write_memory(self.sp, high);
        self.sp -= 1;
        self.write_memory(self.sp, low);
        // JP u16
        self.pc = address;
    }

    fn ei(&mut self) {
        self.ime = 1;
        self.next(1);
    }
    fn di(&mut self) {
        self.ime = 0;
        self.next(1);
    }

    // Jump to address u16
    fn jp_u16(&mut self) {
        let address = (self.byte3 as u16) << 8 | self.byte2 as u16; 
        self.pc = address;
    }

    // Jump based on condition
    fn jp_cond(&mut self, cond: u8) {
        // If condition true, JUMP
        if self.flags.get_cond(cond) != 0 {
            self.jp_u16();
        }
    }

    // Load value of HL into SP
    fn ld_sp_hl(&mut self) {
        self.sp = self.get_r16_register(REGISTER16::HL);
        self.next(1);
    }
    // Jump to address to HL
    fn jp_hl(&mut self) {
        self.pc = self.get_r16_register(REGISTER16::HL);
    }

    // Enable interrupts and RETURN
    fn reti(&mut self) {
        // Enable interrupt
        self.ei();
        self.ret();
    }

    // RETURN
    fn ret(&mut self) {
        let low = self.read_memory(self.sp);
        self.sp += 1;
        let high = self.read_memory(self.sp);
        self.sp += 1;
        let value = (((high as u16) << 8) as u16) | (low as u16);
        self.pc = value;
    }
    // RETURN based on condition
    fn ret_cond(&mut self, ret_code: u8) {
        // If condition true, RET
        if self.flags.get_cond(ret_code) != 0 {
            self.ret();
        }
        self.next(1);
    }

    // Push to stack
    fn push_r16(&mut self, r16: u8) {
        let value = self.get_r16stk_register(r16.into());
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;
        self.sp -= 1;
        self.write_memory(self.sp, high);
        self.sp -= 1;
        self.write_memory(self.sp, low);
        self.next(1);
    }

    // Load value in reg A from [n16]
    fn ld_a_n16(&mut self) {
        let address = (self.byte3 as u16) << 8 | self.byte2 as u16; 
        let value = self.read_memory(address);
        self.set_r8_register(REGISTER8::A, value);
        self.pc += 2;
        self.next(1);
    }
    
    // Load [0xff00 + c] into reg A
    fn ldh_a_c(&mut self) {
        let address = 0xff00 + self.get_r8_register(REGISTER8::C) as u16;
        let value = self.read_memory(address);
        self.set_r8_register(REGISTER8::A, value);
        self.next(1);
    }

    // Load reg A value into memory address byte2 << 8|byte3
    fn ld_n16_a(&mut self) {
        let value = self.get_r8_register(REGISTER8::A);
        let address = (self.byte3 as u16) << 8 | self.byte2 as u16; 
        self.write_memory(address, value);
        self.pc += 2;
    }
    // Load value in register A into $ff00 + C
    fn ldh_c_a(&mut self) {
        let value = self.get_r8_register(REGISTER8::A);
        let address = self.get_r8_register(REGISTER8::C);
        self.write_memory(0xff00 + address as u16, value);
        self.next(1);
    }

    // POP address from stack and save to register
    fn pop_r16(&mut self, r16: u8) {
        let low = self.read_memory(self.sp);
        self.sp += 1;
        let high = self.read_memory(self.sp);
        self.sp += 1;
        let value = (((high as u16) << 8) as u16) | (low as u16);
        self.set_r16stk_register(r16.into(), value);
        self.next(1);

    }

    fn ld_hl_sp_imm8(&mut self) {
        let imm8 = self.byte2;
        //NOTE: Might be bugged
        let (half_carry, carry) = self.check_carry_add_i8(self.sp as i8, imm8 as i8);
        let value = (self.sp as i16).wrapping_add(imm8 as i16);
        self.set_r16_register(REGISTER16::HL, value as u16);
        self.flags.zero = 0;
        self.flags.n = 0;
        self.flags.h = half_carry as u8;
        self.flags.carry = carry as u8;
        self.pc += 1;
        self.next(1);

    }
    // Load value in address ff00 + n8 if in range, then save value in register A
    fn ldh_a_i16(&mut self) {
        let address = self.byte2 as u16 + 0xFF00;
        if address < 0xffff && address > 0xff00 {
            let value = self.read_memory(address);
            self.set_r8_register(REGISTER8::A, value);
        }
        self.pc += 1;
        self.next(1);
    }

    // Add immediate i8 value to stack pointer, and set hc/carry flags 
    // appropriately
    // ADD SP i8
    fn add_sp_i8(&mut self) {
        let imm8 = self.byte2;
        let (half_carry, carry) = self.check_carry_add_i8(self.sp as i8, imm8 as i8);
        let value = (self.sp as i16).wrapping_add(imm8 as i16);
        self.set_r16_register(REGISTER16::SP, value as u16);
        self.flags.zero = 0;
        self.flags.n = 0;
        self.flags.h = half_carry as u8;
        self.flags.carry = carry as u8;
        self.pc += 1;
        self.next(1);
    }

    // Load value from dst_r8 into src_r8. When called as LD r1 r2, this method
    // is called as ld_r8_r8(r2, r1) (Notice the inversion)
    fn ld_r8_r8(&mut self, src_r8: u8, dst_r8: u8) {
        let value = self.get_r8_register(src_r8.into());
        self.set_r8_register(dst_r8.into(), value);
        self.next(1);

    }

    // If n8 + ff00 is in range, store value of register A into memory location n8 + ff00
    // LDH [n16], A OR LDH [$FF00 + n8], A
    fn ldh_i16_a(&mut self) {
        let address: u16 = 0xFF00 + self.byte2 as u16;
        if address < 0xffff && address > 0xff00 {
            let value = self.get_r8_register(REGISTER8::A);
            self.write_memory(address, value);
        }
        self.pc += 1;
        self.next(1);
    }

    //TODO - Respond to interrupt
    fn halt(&mut self) {
        panic!("HALT")

    }

    //TODO
    fn special_opcodes(&mut self, opcode: u8) {
        match opcode {
            0 => { self.rlca() },
            1 => { self.rrca() },
            2 => { self.rla() },
            3 => { self.rra() },
            4 => { self.daa() },
            5 => { self.cpl() },
            6 => { self.scf() },
            7 => { self.ccf() },
            _ => { panic!("Invalid opcode for special group: {:02X}", opcode);
            }
        }
        self.next(1);
    }

    fn rlca(&mut self) {
        let reg_code_a = 7;
        self.rlc_r8(reg_code_a.into());
    }

    fn rrca(&mut self) {
        let reg_code_a = 7;
        self.rrc_r8(reg_code_a.into());
    }

    fn rla(&mut self) {
        let reg_code_a = 7;
        self.rl_r8(reg_code_a.into());
    }

    fn rra(&mut self) {
        let reg_code_a = 7;
        self.rr_r8(reg_code_a.into());
    }

    fn daa(&mut self) {
        panic!("TODO")
    }
    fn cpl(&mut self) {
        let reg_code_a = 7;
        let mut reg = self.get_r8_register(reg_code_a.into());
        reg = !reg;
        self.set_r8_register(reg_code_a.into(), reg);
    }
    fn scf(&mut self) {
        panic!("TODO")
    }
    fn ccf(&mut self) {
        panic!("TODO")
    }




    //Increment value in register r8 by 1
    fn inc_r8(&mut self, register_lookup: u8) {
        let register = self.get_r8_register(register_lookup.into());
        let sum = register.wrapping_add(1);
        if sum == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        let (half_carry, _) = self.check_carry_add_u8(register, 1);
        if half_carry { self.flags.h = 1 }
        self.set_r8_register(register_lookup.into(), sum);
        self.next(1);
    }

    //Decrement value in register r8 by 1
    fn dec_r8(&mut self, register_lookup: u8) {
        let register = self.get_r8_register(register_lookup.into());
        let sum = register.wrapping_sub(1);
        if sum == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
        let (half_carry, _) = self.check_carry_sub_u8(register, 1);
        if half_carry{ self.flags.h = 1 }
        self.set_r8_register(register_lookup.into(), sum);
        self.next(1);
    }

    //Increment value in register r16 by 1
    fn inc_r16(&mut self, register_lookup: u8) {
        let mut register = self.get_r16_register(register_lookup.into());
        register = register.wrapping_add(1);
        self.set_r16_register(register_lookup.into(), register);
        self.next(1);
    }

    //Decrement value in register r16 by 1
    fn dec_r16(&mut self, register_lookup: u8) {
        let mut register = self.get_r16_register(register_lookup.into());
        register = register.wrapping_sub(1);
        self.set_r16_register(register_lookup.into(), register);
        self.next(1);
    }

    //Load value  n8 into register r8
    fn ld_r8_n8(&mut self, r8: u8) {
        self.set_r8_register(r8.into(), self.byte2);
        self.pc += 1;
        self.next(1);
    }

    //Load value pointed in memory by r16 register pair into register A
    fn ld_a_r16_addr(&mut self, register_lookup: u8) {
        let memory_address = self.get_r16mem_register(register_lookup.into());
        self.a = self.read_memory(memory_address);
        self.next(1);
    }
    // Load the 8 bit value in register A to the memory address pointed by the register from the
    // table
    fn ld_r16_addr_a(&mut self, register_lookup: u8) {
        let memory_address = self.get_r16mem_register(register_lookup.into());
        self.write_memory(memory_address, self.a);
        self.next(1);
    }

    //Add register r16 value to HL
    fn add_hl_r16(&mut self, register_lookup: u8) {
        let r16 = self.get_r16_register(register_lookup.into());
        let hl = self.get_r16_register(REGISTER16::HL); //0b101 == HL register pair
        let (sum, overflow_high) = r16.overflowing_add(hl);

        //Additions reset the n flag
        self.flags.n = 0;
        //Check 11th to 12th bit overflow
        if (r16 & 0xfff).overflowing_add(hl & 0xfff).1 as u8 == 1 { self.flags.h = 1 }
        //Check 15th to 16th bit overflow
        if overflow_high { self.flags.carry = 1 }

        self.set_r16_register(REGISTER16::HL, sum);
        self.next(1);
    }

    //Load value u16 into register r16
    fn ld_r16_u16(&mut self, register_lookup: u8) {
        let address = (self.byte3 as u16) << 8 | self.byte2 as u16; 
        self.set_r16_register(register_lookup.into(), address);
        self.pc += 2;
        self.next(1);
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
        } else {
            self.pc += 1;
        }
    }

    //Unconditional relative jump
    fn jr(&mut self) {
        let offset: i8 = self.byte2 as i8;
        self.pc = self.pc.wrapping_add_signed(offset.into());
    }

    //Store SP lower at address u16, and SP upper at address u16 + 1
    fn ld_u16_sp(&mut self) {
        let address: u16 = (self.byte3 as u16) << 8 | self.byte2 as u16;
        self.write_memory(address, (self.sp & 0xff) as u8);
        self.write_memory(address + 1, (self.sp >> 8) as u8);
        self.pc += 2;
        self.next(1);
    }

    //All math based operations are processed here
    fn alu_a_r8(&mut self, opcode: u8, r8: u8) {
        match opcode {
            0b000 => { println!("add_a_r8"); self.add_a_r8(r8) },
            0b001 => { println!("adc_a_r8"); self.adc_a_r8(r8) },
            0b010 => { println!("sub_a_r8"); self.sub_a_r8(r8) },
            0b011 => { println!("sbc_a_r8"); self.sbc_a_r8(r8) },
            0b100 => { println!("and_a_r8"); self.and_a_r8(r8) },
            0b101 => { println!("xor_a_r8"); self.xor_a_r8(r8) },
            0b110 => { println!("or_a_r8"); self.or_a_r8(r8) },
            0b111 => { println!("cp_a_r8"); self.cp_a_r8(r8) },
            _ => { panic!("Invalid ALU A R8 Operation: opcode: {}, register: {}", opcode, r8)}
        }
        self.next(1);
    }

    //Add the value in r8 to the a register
    fn add_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8.into());
        let (half_carry, carry) = self.check_carry_add_u8(self.a, reg_val);
        self.a = self.a.wrapping_add(reg_val);
        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
    }
    //Add the value in r8 to the a register, along with the value of the carry flag
    fn adc_a_r8(&mut self, r8: u8) {
        let sum = self.get_r8_register(r8.into()).wrapping_add(self.flags.carry);
        let (half_carry, carry) = self.check_carry_add_u8(self.a, sum);
        self.a = self.a.wrapping_add(sum);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;

    }
    //Sub the value in r8 from the a register
    fn sub_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8.into());
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, reg_val);
        self.a = self.a.wrapping_sub(reg_val);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }
    //Sub the value in r8 from the a register along with the value of the carry flag
    fn sbc_a_r8(&mut self, r8: u8) {
        let sum = self.get_r8_register(r8.into()).wrapping_add(self.flags.carry);
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, sum);
        self.a = self.a.wrapping_sub(sum);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }

    //Bitwise AND between r8 value and A
    fn and_a_r8(&mut self, r8: u8) {
        self.a = self.a & self.get_r8_register(r8.into());
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 1;
        self.flags.carry = 0;
    }

    //Bitwise XOR between r8 value and A
    fn xor_a_r8(&mut self, r8: u8) {
        self.a = self.a ^ self.get_r8_register(r8.into());
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 0;
        self.flags.carry = 0;
    }

    //Bitwise OR between r8 value and A
    fn or_a_r8(&mut self, r8: u8) {
        self.a = self.a | self.get_r8_register(r8.into());
        if self.a == 0 { self.flags.zero = 0 }
        self.flags.n = 0;
        self.flags.h = 0;
        self.flags.carry = 0;
    }

    //Subtract value in r8 from A, but don't store the result, only set flags
    fn cp_a_r8(&mut self, r8: u8) {
        let reg_val = self.get_r8_register(r8.into());
        let (half_carry, carry) = self.check_carry_sub_u8(self.a, reg_val);
        let tmp = self.a.wrapping_sub(reg_val);

        if half_carry { self.flags.h = 1 }
        if carry { self.flags.carry = 1 }
        if tmp == 0 { self.flags.zero = 0 }
        self.flags.n = 1;
    }
}
