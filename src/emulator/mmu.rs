use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct MMU {
    pub rom: Vec<u8>,
    pub rom_bank_0: [u8; 16384],
    pub rom_bank_n: [u8; 16384],
    pub external_ram_bank_n: [u8; 8192],
    pub wram_bank_0: [u8; 4096],
    pub wram_bank_n: [u8; 4096],
    pub vram: [u8; 8192],
    pub object_attribute_memory: [u8; 160],
    pub io_registers: [u8; 128],
}

impl MMU {
    pub fn new() -> Self {
        Self {
            rom_bank_0: [0; 16384],
            rom_bank_n: [0; 16384],
            external_ram_bank_n: [00; 8192],
            wram_bank_0: [0; 4096],
            wram_bank_n: [0; 4096],
            vram: [0; 8192],
            object_attribute_memory: [0; 160],
            io_registers: [0; 128],
            rom: Vec::new(),
        }
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
}
