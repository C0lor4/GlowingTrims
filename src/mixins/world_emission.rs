fn world_trim_emission(
    mut geometry: (Vec<Vertex>, Arc<[StaticSectionSnapshot]>, Vec<[i32; 3]>, StaticCacheFrameStats,),
    game: &mut Game<'_>,
) -> (Vec<Vertex>, Arc<[StaticSectionSnapshot]>, Vec<[i32; 3]>, StaticCacheFrameStats,) {
    crate::game::api_values::block_state::glowing_trims_light::flatten_trim_faces(
        &mut geometry.0,
        &mut game.skin_frame.get_mut().jobs,
    );
    geometry
}
