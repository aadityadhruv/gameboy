use super::cpu;
use super::ppu;
use super::mmu;

pub struct Emulator {
    pub cpu: cpu::CPU,
    pub ppu: ppu::PPU,
    pub mmu: mmu::MMU,
}

impl Emulator {
    pub fn new() -> Self {
        let mmu = mmu::MMU::new();
        let cpu = cpu::CPU::new();
        let ppu = ppu::PPU::new();
        Self { cpu, ppu, mmu }
    }
}

