use super::glowing_trims_texture::ShineTexture;
use ::horizon_client::{font::Patch, hud::HudAtlas, trims::TrimTexture};
use std::{cell::RefCell, collections::HashMap, time::Instant};

// The high byte of the entity overlay carries the renderer's light-source tier.
// The low bits select the plain textured material.
pub(crate) const TRIM_EMISSION: u32 = (::horizon_client::mesher::BRIGHT as u32) << 16;
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
    active_rects: Vec<[f32; 4]>,
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
            cache.active_rects.clear();
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
        cache.active_rects.clear();
        for name in &cache.names {
            let Some(rect) = atlas.sprites.get(name) else { continue };
            let x = (rect.u0 * atlas.width as f32).round() as u32;
            let y = (rect.v0 * atlas.height as f32).round() as u32;
            let Some(slot) = cache.slots.get_mut(&(x, y)) else { continue };
            if rect.w != slot.width || rect.h != slot.height {
                continue;
            }
            cache.active_rects.push([rect.u0, rect.v0, rect.u1, rect.v1]);
            if slot.frame == tick {
                continue;
            }
            patches.push(Patch { x, y, width: slot.width, height: slot.height, rgba: slot.texture.frame(phase) });
            slot.frame = tick;
        }
    });
}


// Models carry cardinal-light normals even when both lightmap channels are full.
// Match Horizon's eye-layer treatment, restricted to our own atlas tiles. The
// BRIGHT tier is also used by unrelated entities such as spiders and beacons.
pub(crate) fn flatten_trim_faces(
    vertices: &mut [::horizon_client::mesher::Vertex],
    jobs: &mut [::horizon_client::gfx::entity_skin::SkinJob],
) {
    use ::horizon_client::gfx::entity_skin::SkinJob;
    use ::horizon_client::mesher::{CULL_BACK, SHADED_CULL_W};

    EFFECT.with_borrow(|cache| {
        let rects = &cache.active_rects;
        if rects.is_empty() {
            return;
        }
        // The upper byte can carry OIT/ray classification added by the game.
        let is_trim_candidate = |layer: u32, light: u32| {
            (layer & 0x00ff_ffff) == TRIM_EMISSION && light == FULL_BRIGHT
        };
        for quad in vertices.chunks_exact_mut(4) {
            if !is_trim_candidate(quad[0].layer, quad[0].light) {
                continue;
            }
            let u = (quad[0].uv[0] + quad[2].uv[0]) * 0.5;
            let v = (quad[0].uv[1] + quad[2].uv[1]) * 0.5;
            if rects.iter().any(|r| u >= r[0] && u < r[2] && v >= r[1] && v < r[3]) {
                for corner in quad {
                    corner.normal = if corner.normal[3] == SHADED_CULL_W || corner.normal == CULL_BACK { CULL_BACK } else { [0; 4] };
                }
            }
        }
        for job in jobs {
            if job.mode == SkinJob::MODEL && is_trim_candidate(job.overlay, job.light)
                && rects.contains(&job.rect)
            {
                job.mode = SkinJob::FLAT;
            }
        }
    });
}
