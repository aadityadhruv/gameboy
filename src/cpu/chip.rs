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
        }
    }
    pub fn load_rom(&mut self, rom_path: &str) {
        let mut rom_buf = Vec::new();
        let mut rom = File::open(rom_path).unwrap_or_else(|_err| panic!("Valid ROM needed!"));
        rom.read_to_end(&mut rom_buf).unwrap_or_else(|_err| panic!("Error reading ROM"));

	for i in 0..self.rom_bank_0.len() {
            self.rom_bank_0[i] = rom_buf.get(i).unwrap().to_owned();
        }
        self.pc = 0x0100;
    }
    pub fn write_memory(&mut self, pc: usize, value: u8) {
        match pc {
            0x0000..=0x3fff => { println!("ROM Bank 0"); self.rom_bank_0[pc as usize] = value  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { println!("ROM Bank 1"); self.rom_bank_n[pc as usize - 0x4000] = value },
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

        match self.instr {
            0x00 => { println!("NOOP!") }
            0x32 => { self.ldd_hl_a() }
            0x0e | 0x06 => { self.ld_rr(); next = 1; }
            0xc3 => { self.jmp_nn() }
            0xa8..=0xaf => { self.xor_r() }
            0x01 | 0x11 | 0x21 | 0x31 => { self.ld_rr_nn(); next = 2; },
            _ => { println!("Error: 0x{:02X} not implemented!", self.instr); std::process::exit(1); }
        }
        self.pc += next;
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
        self.write_memory(pc as usize, self.a);
        pc -= 1;
        self.h = (pc >> 8) as u8;
        self.l = (pc << 8 >> 8) as u8;
    }

    fn xor_r(&mut self) {
        let value = match self.instr & 0x0f {
            0x8 => { self.b },
            0x9 => { self.c },
            0xa => { self.d },
            0xb => { self.e },
            0xc => { self.h },
            0xd => { self.l },
            0xe => { 
                let ptr =  ((self.h as u16) << 8) as u16 | self.l as u16;
                self.read_memory(ptr as usize)
            },
            0xf => { self.a },
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
