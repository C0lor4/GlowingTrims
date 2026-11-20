// The engine supplies the already palette-mapped texture, including resource-pack trims.
pub(crate) struct ShineTexture {
    base: Vec<u8>,
    bands: [Vec<u32>; 32],
    halo_bands: [Vec<u32>; 32],
}

impl ShineTexture {
    pub(crate) fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        let mut result = Self {
            base: pixels,
            bands: Default::default(),
            halo_bands: Default::default(),
        };
        let (w, h) = (width as usize, height as usize);
        if w == 0 || h == 0 || w > 512 || h > 512 || result.base.len() != w * h * 4 {
            return result;
        }

        // Find the nearest original opaque texel in each row. For each transparent
        // texel only rows inside the glow radius need to be searched.
        let mut nearest = vec![u16::MAX; w * h];
        for y in 0..h {
            let row = &mut nearest[y * w..(y + 1) * w];
            let mut left = u16::MAX;
            for (x, seed) in row.iter_mut().enumerate() {
                if result.base[4 * (y * w + x) + 3] >= 64 {
                    left = x as u16;
                }
                *seed = left;
            }
            let mut right = u16::MAX;
            for (x, seed) in row.iter_mut().enumerate().rev() {
                if *seed == x as u16 {
                    right = x as u16;
                } else if right != u16::MAX && (*seed == u16::MAX || right - (x as u16) < (x as u16) - *seed) {
                    *seed = right;
                }
            }
        }

        // Express the radius and band slopes in vanilla's 64 x 32 model UV units,
        // so a high-resolution pack gets the same apparent halo width.
        let dx = 64.0 / width as f32;
        let dy = 32.0 / height as f32;
        let reach = (2.65 / dy).ceil() as usize;
        for y in 0..h {
            for x in 0..w {
                let offset = (y * w + x) * 4;
                let band = ((x as f32 * dx * 0.85 + y as f32 * dy * 1.15) * (32.0 / 43.0)) as usize % 32;
                if result.base[offset + 3] != 0 {
                    result.bands[band].push(offset as u32);
                    continue;
                }
                let mut nearest_dist = 2.65_f32.powi(2);
                let mut seed = None;
                for row in y.saturating_sub(reach)..=(y + reach).min(h - 1) {
                    let col = nearest[row * w + x];
                    if col == u16::MAX {
                        continue;
                    }
                    let dist = ((col as f32 - x as f32) * dx).powi(2)
                        + ((row as f32 - y as f32) * dy).powi(2);
                    if dist < nearest_dist {
                        nearest_dist = dist;
                        seed = Some(4 * (row * w + col as usize));
                    }
                }
                let Some(seed) = seed else { continue };
                let rgba = &result.base[seed..seed + 4];
                let peak = f32::from(rgba[0].max(rgba[1]).max(rgba[2])).max(1.0);
                let alpha = (f32::from(rgba[3]) * 0.75 * (-0.47 * nearest_dist).exp()).round() as u8;
                // The entity renderer discards fragments below 0.1 alpha.
                if alpha < 26 {
                    continue;
                }
                let halo = std::array::from_fn::<_, 3, _>(|c| {
                    (48.0 + 188.0 * f32::from(rgba[c]) / peak).round() as u8
                });
                result.base[offset..offset + 3].copy_from_slice(&halo);
                result.base[offset + 3] = alpha;
                result.halo_bands[band].push(offset as u32);
            }
        }
        result
    }

    pub(crate) fn frame(&self, phase: u32) -> Vec<u8> {
        let mut pixels = self.base.clone();
        // A narrow white-hot center and softer shoulders travel together. Raise
        // only texels reached by the band; each new frame starts from the base.
        for (index, strength) in [17u16, 54, 114, 206, 114, 54, 17].into_iter().enumerate() {
            let band = (phase as usize + 32 + index - 3) % 32;
            for &offset in &self.bands[band] {
                for channel in &mut pixels[offset as usize..offset as usize + 3] {
                    *channel += (((255 - u16::from(*channel)) * strength + 127) / 255) as u8;
                }
            }
            for &offset in &self.halo_bands[band] {
                let a = &mut pixels[offset as usize + 3];
                *a += (((255 - u16::from(*a)) * strength) / 850) as u8;
            }
        }
        pixels
    }
}
