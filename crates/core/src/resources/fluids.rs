use std::collections::HashMap;

use crate::components::Position;
use crate::resources::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FluidType {
    Water,
    // Add more fluid types as needed
}

#[derive(Debug, Clone)]
pub struct Fluid {
    pub fluid_type: FluidType,
    pub position: Position,
    pub amount: u32,
    pub max_amount: u32,
    pub settled: bool,
    pub spread_threshold: u32,
    pub stability_counter: u32,
}

impl Fluid {
    pub fn new(fluid_type: FluidType, position: Position, amount: u32) -> Self {
        Self {
            fluid_type,
            position,
            amount,
            max_amount: 100,
            settled: false,
            spread_threshold: 10,
            stability_counter: 0,
        }
    }
}

#[derive(Default)]
pub struct FluidManager {
    pub fluids: HashMap<Position, Fluid>,
}

impl FluidManager {
    pub fn add_fluid(&mut self, fluid: Fluid) {
        let pos = fluid.position;
        self.fluids
            .entry(pos)
            .and_modify(|existing| {
                if existing.fluid_type == fluid.fluid_type {
                    let total = existing.amount + fluid.amount;
                    if total <= existing.max_amount {
                        existing.amount = total;
                    } else {
                        existing.amount = existing.max_amount;
                        // Overflow: could implement spreading here
                    }
                    existing.settled = false;
                    existing.stability_counter = 0;
                }
            })
            .or_insert(fluid);
    }

    pub fn remove_fluid(&mut self, position: Position) {
        self.fluids.remove(&position);
    }

    pub fn get_fluid(&self, position: Position) -> Option<&Fluid> {
        self.fluids.get(&position)
    }

    pub fn process_fluids(&mut self, world: &World, gametick: u64) {
        // For now, just a stub: in future, implement spreading logic
        // Iterate and update fluids
        let mut to_remove = vec![];
        for (pos, fluid) in self.fluids.iter_mut() {
            if fluid.amount == 0 {
                to_remove.push(*pos);
            }
            // TODO: Add spreading logic here
        }
        for pos in to_remove {
            self.fluids.remove(&pos);
        }
    }
}

// Debug utility to spawn exactly 4 pools in a cross around the center
pub fn spawn_debug_pools(fm: &mut FluidManager, center: Position) {
    let offsets = [
        (1, 0), (-1, 0), (0, 1), (0, -1)
    ];
    for (dx, dy) in offsets.iter() {
        let pos = Position { x: center.x + dx, y: center.y + dy, z: center.z };
        fm.add_fluid(Fluid::new(FluidType::Water, pos, 60));
    }
}
