//! Sound cues: the core notes when something audible happens, and the client plays the
//! queued cues each frame. Cues are never saved and never feed back into the game, so a
//! run is the same with or without audio.

/// Something the player should hear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    /// The player fells a tree
    Chop,
    /// The player mines a tile or breaks a block
    Mine,
    /// The player places a block or machine
    Place,
    /// Items go into the player's inventory from the ground
    Pickup,
    /// The player damages an enemy
    Hit,
    /// The player takes damage
    Hurt,
    /// A teleport
    Teleport,
    /// A recipe is crafted
    Craft,
}

impl SoundCue {
    pub const ALL: [SoundCue; 8] = [
        SoundCue::Chop,
        SoundCue::Mine,
        SoundCue::Place,
        SoundCue::Pickup,
        SoundCue::Hit,
        SoundCue::Hurt,
        SoundCue::Teleport,
        SoundCue::Craft,
    ];

    /// The name the client registers the cue's sound under, and its file's name.
    pub fn name(self) -> &'static str {
        match self {
            SoundCue::Chop => "wood_crack",
            SoundCue::Mine => "mine",
            SoundCue::Place => "place",
            SoundCue::Pickup => "pickup",
            SoundCue::Hit => "hit",
            SoundCue::Hurt => "hurt",
            SoundCue::Teleport => "teleport",
            SoundCue::Craft => "craft",
        }
    }
}

/// At most this many cues wait to be played; more are dropped.
pub const MAX_QUEUED_SOUNDS: usize = 32;

impl crate::resources::Resources {
    /// Queue a sound for the client to play.
    pub fn play(&mut self, cue: SoundCue) {
        if self.sounds.len() < MAX_QUEUED_SOUNDS {
            self.sounds.push(cue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SoundCue;
    use crate::components::{ItemKind, Position};
    use crate::intent::PlayerAction;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    fn mined(tile: TileKind) -> Vec<SoundCue> {
        let mut test = TestWorld::new(3)
            .fill((-2, -2, 0), (2, 2, 0), TileKind::Grass)
            .tile(1, 0, 0, tile)
            .inventory(&[])
            .auto_pickup(false)
            .build();
        test.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        test.tick();
        test.game.res.sounds.clone()
    }

    #[test]
    fn mining_and_chopping_make_their_sounds() {
        assert_eq!(mined(TileKind::Rock), vec![SoundCue::Mine]);
        assert_eq!(mined(TileKind::Tree), vec![SoundCue::Chop]);
    }

    #[test]
    fn picking_up_makes_a_sound() {
        let mut world = TestWorld::new(3)
            .fill((-2, -2, 0), (2, 2, 0), TileKind::Grass)
            .inventory(&[])
            .auto_pickup(true);
        world.item(1, 0, 0, ItemKind::Log, 2);
        let mut test = world.build();
        test.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        test.tick();
        assert!(test.game.res.sounds.contains(&SoundCue::Pickup));
    }

    #[test]
    fn an_enemy_hitting_the_player_makes_the_hurt_sound() {
        let mut world = TestWorld::new(3).fill((-3, -3, 0), (3, 3, 0), TileKind::Grass);
        world.feral_dog(1, 0, 0);
        let mut test = world.build();
        for _ in 0..300 {
            test.game.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, 0, 10);
            test.tick();
            if test.game.res.sounds.contains(&SoundCue::Hurt) {
                return;
            }
        }
        panic!("no hurt sound: {:?}", test.game.res.sounds);
    }

    #[test]
    fn a_harvester_fells_trees_silently() {
        let mut game = TestWorld::new(3)
            .fill((-2, -2, 0), (2, 2, 0), TileKind::Grass)
            .tile(1, 0, 0, TileKind::Tree)
            .build()
            .game;
        crate::crafting::mine_tile(&mut game.res, Position { x: 1, y: 0, z: 0 });
        assert!(game.res.sounds.is_empty());
    }

    #[test]
    fn the_queue_is_capped_and_never_saved() {
        let mut game = TestWorld::new(3).build().game;
        for _ in 0..100 {
            game.res.play(SoundCue::Hit);
        }
        assert_eq!(game.res.sounds.len(), super::MAX_QUEUED_SOUNDS);
        let saved = serde_json::to_string(&game.res).unwrap();
        assert!(!saved.contains("sounds"));
    }
}
