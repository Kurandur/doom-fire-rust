use getrandom::fill;
pub const FIRE_WIDTH: usize = 320;
pub const FIRE_HEIGHT: usize = 240;

const FIRE_PALETTE_RGB_VALUES: [[u8; 3]; 37] = [
    [0x07, 0x07, 0x07],
    [0x1F, 0x07, 0x07],
    [0x2F, 0x0F, 0x07],
    [0x47, 0x0F, 0x07],
    [0x57, 0x17, 0x07],
    [0x67, 0x1F, 0x07],
    [0x77, 0x1F, 0x07],
    [0x8F, 0x27, 0x07],
    [0x9F, 0x2F, 0x07],
    [0xAF, 0x3F, 0x07],
    [0xBF, 0x47, 0x07],
    [0xC7, 0x47, 0x07],
    [0xDF, 0x4F, 0x07],
    [0xDF, 0x57, 0x07],
    [0xDF, 0x57, 0x07],
    [0xD7, 0x5F, 0x07],
    [0xD7, 0x5F, 0x07],
    [0xD7, 0x67, 0x0F],
    [0xCF, 0x6F, 0x0F],
    [0xCF, 0x77, 0x0F],
    [0xCF, 0x7F, 0x0F],
    [0xCF, 0x87, 0x17],
    [0xC7, 0x87, 0x17],
    [0xC7, 0x8F, 0x17],
    [0xC7, 0x97, 0x1F],
    [0xBF, 0x9F, 0x1F],
    [0xBF, 0x9F, 0x1F],
    [0xBF, 0xA7, 0x27],
    [0xBF, 0xA7, 0x27],
    [0xBF, 0xAF, 0x2F],
    [0xB7, 0xAF, 0x2F],
    [0xB7, 0xB7, 0x2F],
    [0xB7, 0xB7, 0x37],
    [0xCF, 0xCF, 0x6F],
    [0xDF, 0xDF, 0x9F],
    [0xEF, 0xEF, 0xC7],
    [0xFF, 0xFF, 0xFF],
];

pub fn random_0_to_3() -> u8 {
    let mut buf = [0u8; 1];
    fill(&mut buf).expect("random generation failed");

    buf[0] % 4
}

pub struct DoomFire {
    pub fire_pixels: Vec<u8>,
}

impl DoomFire {
    pub fn new() -> Self {
        // set whole buffer to black
        let mut fire_pixels = vec![0; FIRE_WIDTH * FIRE_HEIGHT];
        // set bottom row to white (index 36)
        for i in 0..FIRE_WIDTH {
            fire_pixels[(FIRE_HEIGHT - 1) * FIRE_WIDTH + i] = 36;
        }
        DoomFire { fire_pixels }
    }

    pub fn spread_fire(&mut self, i: usize) {
        let pixel = self.fire_pixels[i];
        if pixel == 0 {
            self.fire_pixels[i - FIRE_WIDTH] = 0;
        } else {
            let rand_idx = (random_0_to_3() as f64 + 0.5) as usize & 3;
            let dst = i - rand_idx + 1;
            self.fire_pixels[dst - FIRE_WIDTH] = pixel - ((rand_idx & 1) as u8);
        }
    }

    pub fn do_fire(&mut self) {
        for i in 0..FIRE_WIDTH {
            for j in 1..FIRE_HEIGHT {
                self.spread_fire(j * FIRE_WIDTH + i);
            }
        }
    }

    pub fn get_color_from_palette(&self, index: usize) -> [u8; 4] {
        if index >= FIRE_PALETTE_RGB_VALUES.len() {
            [0, 0, 0, 255]
        } else {
            let [r, g, b] = FIRE_PALETTE_RGB_VALUES[index];
            [r, g, b, 255]
        }
    }
}
