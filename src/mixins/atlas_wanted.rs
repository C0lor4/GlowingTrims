fn wanted_trims(wanted: Vec<TrimTexture>) -> Vec<TrimTexture> {
    crate::game::api_values::block_state::glowing_trims_light::visible(&wanted);
    wanted
}
