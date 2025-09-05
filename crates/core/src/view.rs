use crate::components::{Player, Position};
use crate::resources::Resources;
use hecs::World;

#[derive(Debug, Clone)]
pub struct RenderView {
    pub gametick: u64,
    pub player_pos: Position,
    pub map_lines: Vec<String>,
}

pub fn build_render_view(world: &World, res: &Resources) -> RenderView {
    // Provide minimal data to the client; sizing and rendering are handled there
    let player_pos = world
        .query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(_, (_, pos))| *pos)
        .unwrap_or(Position { x: 0, y: 0, z: 0 });

    RenderView {
        gametick: res.gametick,
        player_pos,
        map_lines: Vec::new(),
    }
}
