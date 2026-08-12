use super::mmu;

#[derive(Copy, Clone)]
struct Pixel {
    pub color: u8,
}
impl Pixel {
    fn new(color: u8) -> Self {
        Self { color }
    }
}
impl Default for Pixel {
    fn default() -> Pixel {
        Self { color: 0 }
    }
}
#[derive(Copy, Clone)]
struct Tile {
    pixels: [Pixel; 8*8],
}

struct TileMap {
    // 32 x 32 map of indexes of 1 byte
    indices: [[u8; 32]; 32],
}

impl Tile {
    fn new() -> Self {
        Self { pixels: [Pixel::default(); 8*8] }
    }
    fn from_raw(data: [u8; 16]) -> Self {
        let mut pixels: [Pixel; 8 * 8] = [Pixel::new(0); 8 * 8];
        for i in 0..16 {
            // LSB
            let byte1 = data[i];
            // MSB
            let byte2 = data[i + 1];
            let row = i  / 2;
            for j in 0..8 {
                //mask = 0b1000000 shifted by position of current bit we are inspecting, going left to right
                let mask = (0xb1 << 7) >> (7 - j);
                // Pretty ugly stuff.
                // By doing an & of byte and mask, we obtain the jth bit of the byte. But it needs
                // to be shifted all the way to the right by (7 - j) to make it the first bit of a
                // u8. We need to shift 1 by right for byte2 because it's the MSB vs byte1 being
                // LSB, so LSB gets no left shift
                // pixel will look like 0b01 (as a u8 of course)
                pixels[row*8 + j].color = (((byte2 & mask) >> (7 - j)) << 1) & ((byte1 & mask) >> (7 - j));
            }
        }

        Self { pixels }
    }
}
pub struct PPU {
}

impl PPU {
    pub fn new() -> Self {
        Self {  }
    }

    // Convert slice of vram from 0x8000 to 0x97ff of tile data
    // to struct Tile array
    fn get_tiles(mmu: &mut mmu::MMU) -> [Tile; 384] {
        let mut tiles = [Tile::new(); 384]; 

        // 384 tiles in vram, 16 bytes each
        for i in (0..384).step_by(16) {
            let mut tile_data: [u8; 16] = [0; 16];
            // This is so garbage because I can't for some reason
            // get a fixed size slice, idk how to do this better
            for j in 0..16 {
                //starting from 0x8000
                tile_data[j] = mmu.vram[0x8000 + i + j];
            }
            let tile = Tile::from_raw(tile_data);
            tiles[i] = tile;
        }

        return tiles;
    }
    fn get_tilemap(mmu: &mut mmu::MMU) -> TileMap {
        return TileMap { indices: [[0; 32]; 32] }
    }

    // fn mode1(&mut self, scanline: u8) {
    // }
    fn mode2(&mut self, scanline: u8) {
    }
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
