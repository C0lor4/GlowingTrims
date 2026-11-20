fn worn_trim(trim: (&Part, [f32; 4], u32, u32, bool)) -> (&Part, [f32; 4], u32, u32, bool) {
    let (model, uv, color, _, _) = trim;
    // The colored trim animates in its atlas slot. Vanilla armor glint would
    // override the transparent halo's alpha; enchanted base armor keeps its glint.
    (model, uv, color, 0, true)
}
