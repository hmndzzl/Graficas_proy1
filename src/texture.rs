use image::{GenericImageView};

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Texture {
    pub fn new(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let img_rgba = img.to_rgba8();
        let (width, height) = img_rgba.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = img_rgba.get_pixel(x, y);
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                let a = pixel[3] as u32;
                
                // Pack to ARGB (Minifb default layout is XRGB or ARGB)
                let color = (a << 24) | (r << 16) | (g << 8) | b;
                pixels.push(color);
            }
        }

        Ok(Texture { width, height, pixels })
    }

    pub fn get_pixel_color(&self, tx: u32, ty: u32) -> u32 {
        if tx >= self.width || ty >= self.height {
            return 0;
        }
        self.pixels[(ty * self.width + tx) as usize]
    }
}
