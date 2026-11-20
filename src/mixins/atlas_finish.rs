fn finish_trim_uploads(_unit: (), game: &mut Game<'_>) {
    crate::game::api_values::block_state::glowing_trims_light::finish(
        &game.gui.atlas,
        &mut game.map_patches,
    );
}
