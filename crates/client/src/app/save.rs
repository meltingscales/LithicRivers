//! Saving and loading the whole session: the core game plus the client state that affects
//! play (view position, zoom, tutorial progress, hotbar, build mode, the Crafting filter).
//! Transient UI such as open modals is not saved.

use crate::App;
use lithicrivers_core::components::ItemKind;
use lithicrivers_core::save_load;
use serde::{Deserialize, Serialize};

/// The client section of a save.
#[derive(Serialize, Deserialize)]
struct ClientSave {
    view: [i64; 3],
    tutorial_step: usize,
    completed_tutorials: Vec<String>,
    hotbar: [Option<ItemKind>; 12],
    /// The Crafting tab lists only what can be crafted now
    #[serde(default)]
    craftable_only: bool,
    /// Zoom
    #[serde(default)]
    scale: crate::sprite_loader::Scale,
    /// Build mode and the build tool's choices
    #[serde(default)]
    build_mode: crate::app_state::BuildMode,
    #[serde(default)]
    hotbar_slot: usize,
    #[serde(default)]
    placement_facing: lithicrivers_core::factory::Facing,
    #[serde(default)]
    stairs_down: bool,
    /// Blueprint mode's cursor, corner and turn, so a reload mid-blueprint carries on
    #[serde(default)]
    blueprint: crate::app_state::BlueprintTool,
}

impl App {
    fn client_save(&self) -> ClientSave {
        ClientSave {
            view: [self.ui.view_x, self.ui.view_y, self.ui.view_z],
            tutorial_step: self.ui.tutorial_system.current_step_index,
            completed_tutorials: self.ui.tutorial_system.completed_tutorials.clone(),
            hotbar: self.panels.build.hotbar_assignments,
            craftable_only: self.panels.crafting.craftable_only,
            scale: self.ui.scale,
            build_mode: self.panels.build.mode,
            hotbar_slot: self.panels.build.selected_hotbar_slot,
            placement_facing: self.panels.build.placement_facing,
            stairs_down: self.panels.build.stairs_down,
            blueprint: self.panels.build.blueprint.clone(),
        }
    }

    fn set_view(&mut self, [x, y, z]: [i64; 3]) {
        (self.ui.view_x, self.ui.view_y, self.ui.view_z) = (x, y, z);
    }

    /// The current session as a save file's contents.
    pub(crate) fn save_to_string(&self) -> anyhow::Result<String> {
        let client = serde_json::to_value(self.client_save())?;
        save_load::save_to_string(&self.core.game, &client)
    }

    pub(crate) fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let client = serde_json::to_value(self.client_save())?;
        save_load::save_to_file(&self.core.game, path, &client)
    }

    /// Replace the current session with a save produced by [`App::save_to_string`].
    pub(crate) fn load_from_str(&mut self, json: &str) -> anyhow::Result<()> {
        let loaded = save_load::load_from_str(&mut self.core.game, json)?;
        self.apply_loaded(loaded)
    }

    pub(crate) fn load_from_file(&mut self, path: &str) -> anyhow::Result<()> {
        let loaded = save_load::load_from_file(&mut self.core.game, path)?;
        self.apply_loaded(loaded)
    }

    fn apply_loaded(&mut self, loaded: save_load::LoadedSave) -> anyhow::Result<()> {
        if let Some(viewport) = loaded.viewport {
            // Version 4 save: only the view position was kept
            self.set_view([viewport.view_x, viewport.view_y, viewport.view_z]);
        } else if !loaded.client.is_null() {
            let client: ClientSave = serde_json::from_value(loaded.client)?;
            self.set_view(client.view);
            self.ui.tutorial_system.current_step_index = client.tutorial_step;
            self.ui.tutorial_system.completed_tutorials = client.completed_tutorials;
            self.panels.build.hotbar_assignments = client.hotbar;
            self.panels.crafting.craftable_only = client.craftable_only;
            self.ui.scale = client.scale;
            self.panels.build.mode = client.build_mode;
            self.panels.build.selected_hotbar_slot = client.hotbar_slot.min(11);
            self.panels.build.placement_facing = client.placement_facing;
            self.panels.build.stairs_down = client.stairs_down;
            self.panels.build.blueprint = client.blueprint;
        }
        // The combat panel follows the loaded game
        let in_combat = self.core.game.res.player_state.combat_active;
        if in_combat && !self.combat.is_active() {
            self.open_combat_ui();
        } else if !in_combat {
            self.combat = crate::app_state::CombatUiState::None;
        }
        Ok(())
    }
}
