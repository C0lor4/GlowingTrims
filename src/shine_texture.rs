// Horizon supplies one palette-mapped trim texture. Its original coverage mask
// is also the emission mask, including any translucent edges in a resource pack.
pub(crate) struct ShineTexture {
    base: Vec<u8>,
    bands: [Vec<u32>; 32],
}

impl ShineTexture {
    pub(crate) fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        let mut result = Self { base: pixels, bands: Default::default() };
        if width == 0 || height == 0 || width > 512 || height > 512
            || result.base.len() != (width * height * 4) as usize
        {
            return result;
        }

        let dx = 64.0 / width as f32;
        let dy = 32.0 / height as f32;
        for (i, rgba) in result.base.chunks_exact_mut(4).enumerate() {
            if rgba[3] == 0 {
                continue;
            }
            // Keep the hue chosen by the material palette. Raising the emitting
            // core before animation keeps the line lit between passes of the glint.
            let peak = rgba[0].max(rgba[1]).max(rgba[2]);
            if peak == 0 {
                continue; // Authored black ink can separate parts of the motif.
            }
            let value = f32::from(peak) / 255.0;
            // Ease out of black so tiny palette rounding differences in ink
            // do not turn into bright colored specks.
            let toe = (f32::from(peak) / 24.0).min(1.0);
            let radiant_peak = (145.0 + 110.0 * value.sqrt()) * toe;
            let hot_core = 0.08 + 0.14 * value;
            for channel in &mut rgba[..3] {
                let hue = f32::from(*channel) / f32::from(peak);
                *channel = (radiant_peak * (hue + (1.0 - hue) * hot_core)).round() as u8;
            }

            let x = (i % width as usize) as f32 * dx;
            let y = (i / width as usize) as f32 * dy;
            let band = ((x * 0.85 + y * 1.15) * (32.0 / 43.0)) as usize % 32;
            result.bands[band].push((4 * i) as u32);
        }
        result
    }

    pub(crate) fn frame(&self, phase: u32) -> Vec<u8> {
        let mut pixels = self.base.clone();
        let phase = (phase % 32) as usize;
        for (i, strength) in [18u16, 56, 124, 206, 124, 56, 18].into_iter().enumerate() {
            let band = (phase + 32 + i - 3) % 32;
            for &offset in &self.bands[band] {
                let rgb = &mut pixels[offset as usize..offset as usize + 3];
                // Near-black outlines also cap the moving reflection. A tiny
                // nonzero palette value should never produce a white spark.
                let peak = u16::from(rgb[0].max(rgb[1]).max(rgb[2]));
                let highlight = (2 * peak).min(255);
                for channel in rgb {
                    *channel += (((highlight - u16::from(*channel)) * strength + 127) / 255) as u8;
                }
            }
        }
        pixels
    }
}
