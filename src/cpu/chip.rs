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
    pub fn read_memory(&self, pc: usize) -> u8 {
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
        self.instr = self.read_memory(self.pc as usize);
        let arg1 = self.read_memory(self.pc as usize + 1);
        let arg2 = self.read_memory(self.pc as usize + 2);
        self.byte2 = arg1;
        self.byte3 = arg2;
        println!("Instr: 0x{:02X}", self.instr);
        self.pc += 1;
        println!("0x{:02X?}", &self.rom_bank_0[self.pc as usize..(self.pc + 10) as usize]);
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
            (0b00, 0b001, 0b000) => { self.ld_xx_sp() }, //LD (u16), SP
            (0b00, 0b010, 0b000) => { println!("STOP") }, //STOP
            (0b00, 0b011, 0b000) => { self.jr() }, //JR
            (0b00, 0b100..=0b111, 0b000) => { self.jr_cond(oct2) }, //JR conditonal
            (0b00,0b000|0b010|0b100|0b110, 0b001) => { self.ld_xx_rr(oct2) }, //LD r16, u16
            (0b00,0b001|0b011|0b101|0b111, 0b001) => { self.add_hl_rr(oct2) }, //ADD HL, r16
            (0b00,0b000|0b010|0b100|0b110, 0b010) => {  }, //LD (r16), A
            (0b00,0b001|0b011|0b101|0b111, 0b010) => {  }, //LD A, (r16)
            (0b00,0b000|0b010|0b100|0b110, 0b011) => {  }, //INC r16
            (0b00,0b001|0b011|0b101|0b111, 0b011) => {  }, //DEC r16
            (0b00, r8, 0b100) => {  }, //INC r8
            (0b00, r8, 0b101) => {  }, //DEC r8
            (0b00, r8, 0b110) => {  }, //LD r8, u8
            (0b00, opcode, 0b111) => {  }, //Opcode grp 1
            (0b01, 0b110, 0b110) => {  }, //HALT
            (0b01, src_r8, dst_r8) => {  }, //LD r8, r8
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

//        match self.instr {
//            0x00 => { println!("NOOP!") }
//            0x32 => { self.ldd_hl_a() }
//            0x0e | 0x06 => { self.ld_rr(); next = 1; }
//            0xa8..=0xaf => { self.xor_r() }
//            0x01 | 0x11 | 0x21 | 0x31 => { self.ld_rr_nn(); next = 2; },
//            _ => { println!("Error: 0x{:02X} not implemented!", self.instr); std::process::exit(1); }
//        }
        self.pc += next;
    }


    //Add register r16 value to HL
    fn add_hl_rr(&mut self, register: u8) {
        let r16 = match register {
            0b001 => { (self.b as u16) << 8 | self.c as u16 },
            0b011 => {  (self.d as u16) << 8 | self.e as u16 },
            0b101 => {(self.h as u16) << 8 | self.l as u16 },
            0b111 => { self.sp },
            _ => { panic!("Unknown condition {}", register) }
        };

        self.h += (r16 >> 8) as u8;
        self.l += (r16 & 0x00ff) as u8;

        //Additions reset the n flag
        self.flags.n = 0;
        
    }

    //Load value u16 into register r16
    fn ld_xx_rr(&mut self, register: u8) {
        match register {
            0b000 => {self.b = self.byte3; self.c = self.byte2; },
            0b010 => { self.d = self.byte3; self.e = self.byte2; },
            0b100 => { self.h = self.byte3; self.l = self.byte2; },
            0b110 => { self.sp = (self.byte3 as u16) << 8 | self.byte2 as u16; },
            _c => { panic!("Unknown condition {}", _c) }
        }
    }

    //Conditional Jump
    fn jr_cond(&mut self, condition: u8) {

        let mut should_execute = false;
        match condition {
            0b100 => { should_execute = self.flags.zero != 0 },
            0b101 => { should_execute = self.flags.zero == 0},
            0b110 => { should_execute = self.flags.carry != 0 },
            0b111 => { should_execute = self.flags.carry == 0 },
            _c => { panic!("Unknown condition {}", _c) }
        }
        if should_execute {
            let offset: i8 = self.byte2 as i8;
            self.pc.wrapping_add_signed(offset.into());
        }
    }

    //Unconditional Jump
    fn jr(&mut self) {
        let offset: i8 = self.byte2 as i8;
        self.pc.wrapping_add_signed(offset.into());
    }

    //Store SP lower at address u16, and SP upper at address u16 + 1
    fn ld_xx_sp(&mut self) {
        let address: u16 = (self.byte2 << 8) as u16 | self.byte3 as u16;
        self.write_memory(address, (self.sp & 0xff) as u8);
        self.write_memory(address + 1, (self.sp & 0xff00 >> 8) as u8);
    }

    fn alu_a_r8(&mut self, opcode: u8, r8: u8) {
        match opcode {
            1 => { self.adc_r(r8) },
            5 => { self.xor_r(r8) },
            _ => {}
        }
    }
    
    fn ld_rr(&mut self) {
        match self.instr {
            0x0e => { self.c = self.byte2 },
            0x06 => { self.b = self.byte2 },
            _ => {},
        }
    }
    
    fn ldd_hl_a(&mut self) {
        let mut pc = ((self.h as u16) << 8) as u16 | self.l as u16;
        self.write_memory(pc, self.a);
        pc -= 1;
        self.h = (pc >> 8) as u8;
        self.l = (pc << 8 >> 8) as u8;
    }

    fn adc_r(&mut self, r8: u8) {
        self.a += self.flags.carry + r8;
    }
    fn xor_r(&mut self, r8: u8) {
        let value = match r8 {
            0 => { self.b },
            1 => { self.c },
            2 => { self.d },
            3 => { self.e },
            4 => { self.h },
            5 => { self.l },
            6 => { 
                let ptr =  ((self.h as u16) << 8) as u16 | self.l as u16;
                self.read_memory(ptr as usize)
            },
            7 => { self.a },
            _ => { panic!("Wrong register read for XOR_R") }
        };
        self.a = self.a ^ value;
        self.flags.zero = (self.a == 0) as u8
    }

    fn jmp_nn(&mut self) {
        println!("{:02X}, {:02X}", self.byte2, self.byte3);
        self.pc = ((self.byte3 as u16) << 8) as u16 | self.byte2 as u16;
        println!("Jumping to 0x{:02X}", self.pc);
    }

    fn ld_rr_nn(&mut self) {
        match self.instr {
            0x01 => { self.b = self.byte3; self.c = self.byte2;  }, //BC = nn (nn = MSB << 8 | LSB)
            0x11 => { self.d = self.byte3; self.e = self.byte2;  }, //DE = nn (nn = MSB << 8 | LSB)
            0x21 => { self.h = self.byte3; self.l = self.byte2;  }, //HL = nn (nn = MSB << 8 | LSB)
            0x31 => { self.sp = ((self.byte3 as u16) << 8)  as u16 | self.byte2 as u16  }, //HL = nn (nn = MSB << 8 | LSB)
            _ => {  },
        }
    }
    
}
