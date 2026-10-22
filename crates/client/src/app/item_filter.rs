//! Which of the player's items a machine window lists. Machines show only what they take,
//! and say so in the panel title; chests and corpses list everything. A selected row is an
//! index into the shown list, so stores go through `slot` to find the inventory stack.

use hecs::{Entity, World};
use lithicrivers_core::components::{Inventory, ItemKind};
use lithicrivers_core::factory::{self, FactoryBlock};

pub struct PlayerRows {
    /// Inventory stack indices shown, in inventory order
    pub slots: Vec<usize>,
    /// What the machine takes, when the list is filtered
    pub label: Option<&'static str>,
    /// Stacks in the inventory
    pub total: usize,
}

impl PlayerRows {
    /// The player's stacks that `takes` allows; a `None` label means no filtering.
    pub fn new(
        world: &World,
        player: Entity,
        label: Option<&'static str>,
        takes: impl Fn(ItemKind) -> bool,
    ) -> Self {
        let kinds: Vec<ItemKind> = world
            .get::<&Inventory>(player)
            .map(|inv| inv.slots.iter().map(|s| s.kind).collect())
            .unwrap_or_default();
        let slots = (0..kinds.len())
            .filter(|&i| label.is_none() || takes(kinds[i]))
            .collect();
        Self {
            slots,
            label,
            total: kinds.len(),
        }
    }

    /// The rows for the block or corpse `container`: filtered for machines.
    pub fn for_container(world: &World, player: Entity, container: Entity) -> Self {
        let kind = world.get::<&FactoryBlock>(container).ok().map(|b| b.kind);
        let label = kind.and_then(factory::takes_label);
        Self::new(world, player, label, |item| {
            kind.is_none_or(|k| factory::takes(k, item))
        })
    }

    /// The rows for a furnace window with `slot` selected: what that slot takes.
    pub fn for_furnace_slot(
        world: &World,
        player: Entity,
        slot: lithicrivers_core::furnace::Slot,
    ) -> Self {
        Self::new(world, player, Some(slot.takes_label()), |item| {
            slot.takes(item)
        })
    }

    /// The inventory stack for a shown row.
    pub fn slot(&self, row: usize) -> Option<usize> {
        self.slots.get(row).copied()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// The panel title: "Your Items", or what the machine takes and how many of the
    /// player's stacks are shown, so it's clear the list isn't the whole inventory.
    pub fn title(&self) -> String {
        match self.label {
            None => " Your Items ".to_string(),
            Some(label) => format!(" Showing {label}: {} of {} ", self.slots.len(), self.total),
        }
    }
}
