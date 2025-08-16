use std::collections::HashMap;

use crate::components::Position;
use crate::palette::PaletteKey;
use crate::resources::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FluidType {
    Water,
    Blood,
    Oil,
    Acid,
    Lava,
    // Add more fluid types as needed
}

impl FluidType {
    pub fn sprite_key(&self) -> &'static str {
        match self {
            FluidType::Water => "water",
            FluidType::Blood => "blood",
            FluidType::Oil => "oil",
            FluidType::Acid => "acid",
            FluidType::Lava => "lava",
        }
    }
    pub fn palette_key(&self) -> PaletteKey {
        match self {
            FluidType::Water => PaletteKey::Water,
            FluidType::Blood => PaletteKey::Blood,
            FluidType::Oil => PaletteKey::Oil,
            FluidType::Acid => PaletteKey::Acid,
            FluidType::Lava => PaletteKey::Lava,
        }
    }
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
    pub viscosity: u32, // Lower = faster flow
    pub last_spread_tick: u64,
    pub settlement_threshold: u32,
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
            viscosity: 10, // Default viscosity for water
            last_spread_tick: 0,
            settlement_threshold: 5,
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
        let mut to_remove = vec![];
        let mut to_add = vec![];
        let directions = [
            (0, 1, 0),  // South
            (0, -1, 0), // North
            (1, 0, 0),  // East
            (-1, 0, 0), // West
            (0, 0, 1),  // Down (z+1)
        ];
        // Work on a snapshot to avoid borrow issues
        let fluids_snapshot: Vec<_> = self.fluids.iter().map(|(p, f)| (*p, f.clone())).collect();
        for (pos, mut fluid) in fluids_snapshot {
            if fluid.amount == 0 {
                to_remove.push(pos);
                continue;
            }
            if fluid.settled || fluid.amount < fluid.spread_threshold {
                fluid.stability_counter += 1;
                if fluid.stability_counter >= fluid.settlement_threshold {
                    if let Some(f) = self.fluids.get_mut(&pos) {
                        f.settled = true;
                    }
                }
                continue;
            }
            let spread_amount =
                std::cmp::min(fluid.amount - fluid.spread_threshold, fluid.viscosity);
            if spread_amount == 0 {
                continue;
            }
            let mut valid_targets = vec![];
            for (dx, dy, dz) in directions.iter() {
                let target = Position {
                    x: pos.x + dx,
                    y: pos.y + dy,
                    z: pos.z + dz,
                };
                if self.can_hold_fluid(world, &target, fluid.fluid_type) {
                    valid_targets.push(target);
                }
            }
            if valid_targets.is_empty() {
                // No spread possible, increase stability
                if let Some(f) = self.fluids.get_mut(&pos) {
                    f.stability_counter += 1;
                    if f.stability_counter >= f.settlement_threshold {
                        f.settled = true;
                    }
                }
                continue;
            }
            // Distribute spread_amount among valid targets
            let amount_per = spread_amount / valid_targets.len() as u32;
            let remainder = spread_amount % valid_targets.len() as u32;
            let mut distributed = 0;
            for (i, target) in valid_targets.iter().enumerate() {
                let mut amt = amount_per;
                if (i as u32) < remainder {
                    amt += 1;
                }
                if amt > 0 {
                    to_add.push((
                        target.clone(),
                        Fluid {
                            fluid_type: fluid.fluid_type,
                            position: *target,
                            amount: amt,
                            max_amount: fluid.max_amount,
                            settled: false,
                            spread_threshold: fluid.spread_threshold,
                            stability_counter: 0,
                            viscosity: fluid.viscosity,
                            last_spread_tick: gametick,
                            settlement_threshold: fluid.settlement_threshold,
                        },
                    ));
                    distributed += amt;
                }
            }
            // Subtract what was spread from this fluid
            if let Some(f) = self.fluids.get_mut(&pos) {
                if f.amount >= distributed {
                    f.amount -= distributed;
                    f.settled = false;
                    f.stability_counter = 0;
                    f.last_spread_tick = gametick;
                }
            }
        }
        // Add new/merged fluids
        for (pos, fluid) in to_add {
            self.add_fluid(fluid);
        }
        for pos in to_remove {
            self.fluids.remove(&pos);
        }
    }

    fn can_hold_fluid(&self, world: &World, pos: &Position, fluid_type: FluidType) -> bool {
        use crate::resources::world::CHUNK_SIZE;
        // Only allow fluid in-bounds and on passable tiles
        let t = world.get_tile(pos.x, pos.y);
        // Only allow on non-solid tiles
        match t {
            crate::tiles::TileKind::Rock | crate::tiles::TileKind::Tree => false, //TODO make this configurable, i.e. "TileKind.is_solid"
            _ => {
                // Don't overfill
                match self.fluids.get(pos) {
                    Some(existing)
                        if existing.fluid_type == fluid_type
                            && existing.amount >= existing.max_amount =>
                    {
                        false
                    }
                    _ => true,
                }
            }
        }
    }
}

// Debug utility to spawn some pools in a cross around the center
pub fn spawn_debug_pools(fm: &mut FluidManager, center: Position) {
    let offsets = [(2, 2), (5, 5)];
    for (dx, dy) in offsets.iter() {
        let pos = Position {
            x: center.x + dx,
            y: center.y + dy,
            z: center.z,
        };
        fm.add_fluid(Fluid::new(FluidType::Water, pos, 500));
    }

    // spawn some lava
    let pos = Position {
        x: center.x + 7,
        y: center.y + 7,
        z: center.z,
    };
    fm.add_fluid(Fluid::new(FluidType::Lava, pos, 500));
}
