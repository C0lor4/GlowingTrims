fn start_trim_uploads(game: &mut Game<'_>) {
    crate::game::api_values::block_state::glowing_trims_light::begin(
        game.gui.atlas.revision,
        game.map_patches.len(),
    );
}
