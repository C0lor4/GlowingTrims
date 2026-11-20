use super::glowing_trims_texture::ShineTexture;
use ::horizon_client::{font::Patch, hud::HudAtlas, trims::TrimTexture};
use std::{cell::RefCell, collections::HashMap, time::Instant};

pub(crate) const FULL_BRIGHT: u32 = ::horizon_client::mobs::render::FULL_BRIGHT;

struct Slot {
    texture: ShineTexture,
    width: u32,
    height: u32,
    frame: u64,
}

#[derive(Default)]
struct AtlasEffect {
    revision: Option<u64>,
    first_trim_patch: usize,
    requested: Vec<TrimTexture>,
    names: Vec<String>,
    request_changed: bool,
    clock: Option<Instant>,
    last_tick: Option<u64>,
    // Keys are the top-left coordinates of Horizon's 32 reusable trim slots.
    slots: HashMap<(u32, u32), Slot>,
}

thread_local! {
    static EFFECT: RefCell<AtlasEffect> = RefCell::new(AtlasEffect::default());
}

pub(crate) fn begin(revision: u64, existing_uploads: usize) {
    EFFECT.with_borrow_mut(|cache| {
        if cache.revision != Some(revision) {
            *cache = AtlasEffect { revision: Some(revision), ..Default::default() };
        }
        cache.first_trim_patch = existing_uploads;
    });
}

pub(crate) fn visible(wanted: &[TrimTexture]) {
    EFFECT.with_borrow_mut(|cache| {
        cache.request_changed = cache.requested != wanted;
        if cache.request_changed {
            cache.requested = wanted.to_vec();
            cache.names = wanted.iter().map(TrimTexture::sprite).collect();
        }
    });
}

pub(crate) fn finish(atlas: &HudAtlas, patches: &mut Vec<Patch>) {
    EFFECT.with_borrow_mut(|cache| {
        if cache.names.is_empty() {
            return;
        }
        // 12 frames per second; ordinary render frames allocate/upload nothing.
        let tick = (cache.clock.get_or_insert_with(Instant::now).elapsed().as_millis() * 12 / 1000) as u64;
        let start = cache.first_trim_patch.min(patches.len());
        if cache.last_tick == Some(tick) && !cache.request_changed && start == patches.len() {
            return;
        }
        cache.last_tick = Some(tick);
        let phase = (tick % 32) as u32;
        // Only prepare_trims' own new uploads contain palette-mapped source pixels.
        // Map images, font uploads and animation frames already in the queue stay untouched.
        for patch in &mut patches[start..] {
            let texture = ShineTexture::new(patch.width, patch.height, std::mem::take(&mut patch.rgba));
            patch.rgba = texture.frame(phase);
            cache.slots.insert((patch.x, patch.y), Slot {
                texture, width: patch.width, height: patch.height, frame: tick,
            });
        }
        for name in &cache.names {
            let Some(rect) = atlas.sprites.get(name) else { continue };
            let x = (rect.u0 * atlas.width as f32).round() as u32;
            let y = (rect.v0 * atlas.height as f32).round() as u32;
            let Some(slot) = cache.slots.get_mut(&(x, y)) else { continue };
            if slot.frame == tick || rect.w != slot.width || rect.h != slot.height {
                continue;
            }
            patches.push(Patch { x, y, width: slot.width, height: slot.height, rgba: slot.texture.frame(phase) });
            slot.frame = tick;
        }
    });
}
