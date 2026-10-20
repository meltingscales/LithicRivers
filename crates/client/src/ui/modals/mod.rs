//! Modal windows drawn over the main UI.

mod action_selection;
mod cheat_console;
mod corpse_looting;
mod hotbar_assignment;
mod npc_interaction;

pub(crate) use action_selection::render_multi_action_selection_modal;
pub(crate) use cheat_console::render_cheat_console_modal;
pub(crate) use corpse_looting::render_corpse_looting_modals;
pub(crate) use hotbar_assignment::render_hotbar_assignment_modal;
pub(crate) use npc_interaction::render_npc_interaction_modals;
