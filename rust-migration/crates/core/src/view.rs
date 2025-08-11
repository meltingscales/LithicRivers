use crate::components::Position;
use crate::resources::Resources;
use hecs::{World};

#[derive(Debug, Clone)]
pub struct RenderView {
    pub gametick: u64,
    pub player_pos: Position,
}

pub fn build_render_view(world: &World, res: &Resources) -> RenderView {
    let mut player_pos = Position { x: 0, y: 0, z: 0 };
    if let Some(e) = res.player_entity {
        if let Ok(p) = world.get::<&Position>(e) {
            player_pos = *p;
        }
    }
    RenderView { gametick: res.gametick, player_pos }
}
