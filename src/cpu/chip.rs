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
    }
    pub fn read_memory(&self, pc: usize) -> u8 {
        let address = match pc {
            0x0000..=0x3fff => { self.rom_bank_0[self.pc as usize]  }, //Fixed bank, rom_bank_0
            0x4000..=0x7fff => { self.rom_bank_n[self.pc as usize - 0x4000] },
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
        self.pc += 1;
    }
    pub fn execute(&self) {

        match self.instr {
            _ => { panic!("Error: 0x{:02X} not implemented!", self.instr) }
        }
    }
}
