struct Pixel {
    color: u8,
}
struct Tile {
    pixels: [Pixel; 8*8],
}
impl Tile {
    fn from_raw(data: [u8; 16]) -> Self {
        for i in 0..15 {
            let byte1 = data[i];
            let byte2 = data[i + 1];
            for j in 0..8 {
                let pixel = 
            }
        }

        Self { }
    }
}
pub struct PPU {
}

impl PPU {
    pub fn new() -> Self {
        Self {  }
    }

    // fn mode1(&mut self, scanline: u8) {
    // }
    // fn mode2(&mut self, scanline: u8) {
    // }
    // fn mode3(&mut self, scanline: u8) {
    // }
    // fn mode0(&mut self, scanline: u8) {
    // }
    // fn render(&mut self) {
    //     // For each scan line
    //     for line in 0..144 {
    //         self.mode2(line);
    //         self.mode3(line);
    //         self.mode0(line);
    //     }
    //     for line in 144..154 {
    //         self.mode1(line);
    //     }
    // }
}
