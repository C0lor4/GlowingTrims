fn worn_trim(trim: (&Part, [f32; 4], u32, u32, bool)) -> (&Part, [f32; 4], u32, u32, bool) {
    let (model, uv, color, _, _) = trim;
    let emission = crate::game::api_values::block_state::glowing_trims_light::TRIM_EMISSION;
    (model, uv, color, emission, true)
}
