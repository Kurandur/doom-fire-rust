use getrandom::fill;

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
    pub width: usize,
    pub height: usize,
    pub fire_pixels: Vec<u8>,
    pub rgba_palette: Vec<u8>,
    pub rgba_buffer: Vec<u8>,
}

impl DoomFire {
    pub fn new(width: usize, height: usize) -> Self {
        let mut fire_pixels = vec![0; width * height];

        for i in 0..width {
            fire_pixels[(height - 1) * width + i] = 36;
        }

        let rgba_palette = Self::build_rgba_palette();
        let rgba_buffer = vec![0; width * height * 4];

        Self {
            width,
            height,
            fire_pixels,
            rgba_palette,
            rgba_buffer,
        }
    }

    fn build_rgba_palette() -> Vec<u8> {
        let mut palette = Vec::with_capacity(FIRE_PALETTE_RGB_VALUES.len() * 4);
        for [r, g, b] in FIRE_PALETTE_RGB_VALUES {
            palette.extend_from_slice(&[r, g, b, 255]);
        }
        palette
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let new_width = new_width.max(1);
        let new_height = new_height.max(1);

        self.width = new_width;
        self.height = new_height;

        let new_pixel_count = new_width * new_height;
        self.fire_pixels.resize(new_pixel_count, 0);
        self.rgba_buffer.resize(new_pixel_count * 4, 0);

        let bottom = new_height - 1;
        for x in 0..new_width {
            self.fire_pixels[bottom * new_width + x] = 36;
        }
    }

    pub fn spread_fire(&mut self, src: usize) {
        let pixel = self.fire_pixels[src];

        if pixel == 0 {
            if src >= self.width {
                self.fire_pixels[src - self.width] = 0;
            }
        } else {
            let rand_idx = (random_0_to_3() as f64 + 0.5) as usize & 3;
            let decay = (rand_idx & 1) as u8;
            let offset = (rand_idx as isize) - 1;

            let src_x = src % self.width;
            let dst_y = src / self.width - 1;

            if dst_y < self.height {
                let dst_x = (src_x as isize + offset) as usize;
                if dst_x < self.width {
                    let dst = dst_y * self.width + dst_x;
                    self.fire_pixels[dst] = pixel.saturating_sub(decay);
                }
            }
        }
    }

    pub fn do_fire(&mut self) {
        for i in 0..self.width {
            for j in 1..self.height {
                self.spread_fire(j * self.width + i);
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

    pub fn get_rgba_buffer(&mut self) -> &[u8] {
        let palette = &self.rgba_palette;
        let out = &mut self.rgba_buffer;

        for (pixel, chunk) in self.fire_pixels.iter().zip(out.chunks_exact_mut(4)) {
            let idx = *pixel as usize * 4;
            chunk.copy_from_slice(&palette[idx..idx + 4]);
        }

        out
    }
}
