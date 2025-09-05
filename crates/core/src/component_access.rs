use crate::components::*;
use hecs::{Entity, World};

/// Safe component access patterns that ensure data consistency
pub struct ComponentAccess<'w> {
    world: &'w mut World,
}

impl<'w> ComponentAccess<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self { world }
    }

    /// Safely update entity position with validation
    pub fn set_position(
        &mut self,
        entity: Entity,
        new_pos: Position,
    ) -> Result<Position, ComponentError> {
        // Validate entity still exists and has Position component
        if !self.world.contains(entity) {
            return Err(ComponentError::EntityNotFound(entity));
        }

        let old_pos = match self.world.get::<&Position>(entity) {
            Ok(pos) => *pos,
            Err(_) => return Err(ComponentError::ComponentMissing(entity, "Position")),
        };

        // Apply the position change
        if let Ok(mut pos) = self.world.get::<&mut Position>(entity) {
            *pos = new_pos;
        } else {
            return Err(ComponentError::ComponentAccessFailed(entity, "Position"));
        }

        Ok(old_pos)
    }

    /// Safely apply damage with health validation
    pub fn apply_damage(
        &mut self,
        entity: Entity,
        damage: u32,
    ) -> Result<DamageResult, ComponentError> {
        // Validate entity exists and is not already dead
        if !self.world.contains(entity) {
            return Err(ComponentError::EntityNotFound(entity));
        }

        if self.world.get::<&Dead>(entity).is_ok() {
            return Ok(DamageResult::AlreadyDead);
        }

        // Try Health component first
        let health_result = if let Ok(mut health) = self.world.get::<&mut Health>(entity) {
            let old_health = health.current;
            health.damage(damage);
            let new_health = health.current;
            let is_alive = health.is_alive();
            Some((old_health, new_health, is_alive))
        } else {
            None
        };

        if let Some((old_health, new_health, is_alive)) = health_result {
            if is_alive {
                return Ok(DamageResult::Damaged {
                    old_health,
                    new_health,
                    damage_applied: damage,
                });
            } else {
                // Mark as dead consistently
                self.world.insert_one(entity, Dead).ok();
                // Remove combat capability
                self.world.remove_one::<Combat>(entity).ok();
                return Ok(DamageResult::Killed {
                    old_health,
                    damage_applied: damage,
                });
            }
        }

        // Try Body component for robots
        if let Ok(_body) = self.world.get::<&crate::model::body::Body>(entity) {
            // Body damage logic would go here
            // For now, return that we couldn't apply damage
            return Ok(DamageResult::NoDamageSystem);
        }

        Err(ComponentError::ComponentMissing(entity, "Health or Body"))
    }

    /// Safely modify inventory with validation
    pub fn modify_inventory<F>(
        &mut self,
        entity: Entity,
        mut modifier: F,
    ) -> Result<(), ComponentError>
    where
        F: FnMut(&mut Inventory),
    {
        if !self.world.contains(entity) {
            return Err(ComponentError::EntityNotFound(entity));
        }

        if let Ok(mut inventory) = self.world.get::<&mut Inventory>(entity) {
            modifier(&mut inventory);
            Ok(())
        } else {
            Err(ComponentError::ComponentMissing(entity, "Inventory"))
        }
    }

    /// Check if entity has component (for validation)
    pub fn has_component<T: hecs::Component>(&self, entity: Entity) -> bool {
        if !self.world.contains(entity) {
            return false;
        }
        self.world.get::<&T>(entity).is_ok()
    }

    /// Batch update multiple related components atomically
    pub fn atomic_update(
        &mut self,
        entity: Entity,
        update: ComponentUpdate,
    ) -> Result<(), ComponentError> {
        if !self.world.contains(entity) {
            return Err(ComponentError::EntityNotFound(entity));
        }

        // Apply all updates in the batch
        match update {
            ComponentUpdate::Move {
                new_position,
                remove_combat_delay,
            } => {
                // Update position
                if let Ok(mut pos) = self.world.get::<&mut Position>(entity) {
                    *pos = new_position;
                }

                // Remove battle delay if requested (for movement after combat)
                if remove_combat_delay {
                    self.world.remove_one::<BattleDelay>(entity).ok();
                }
            }
            ComponentUpdate::StartCombat { trigger_combat } => {
                if trigger_combat {
                    if let Ok(mut combat) = self.world.get::<&mut Combat>(entity) {
                        combat.triggered = true;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ComponentError {
    EntityNotFound(Entity),
    ComponentMissing(Entity, &'static str),
    ComponentAccessFailed(Entity, &'static str),
    ValidationFailed(String),
}

#[derive(Debug)]
pub enum DamageResult {
    Damaged {
        old_health: u32,
        new_health: u32,
        damage_applied: u32,
    },
    Killed {
        old_health: u32,
        damage_applied: u32,
    },
    AlreadyDead,
    NoDamageSystem,
}

#[derive(Debug)]
pub enum ComponentUpdate {
    Move {
        new_position: Position,
        remove_combat_delay: bool,
    },
    StartCombat {
        trigger_combat: bool,
    },
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::EntityNotFound(e) => write!(f, "Entity {:?} not found", e),
            ComponentError::ComponentMissing(e, comp) => {
                write!(f, "Entity {:?} missing component {}", e, comp)
            }
            ComponentError::ComponentAccessFailed(e, comp) => {
                write!(f, "Failed to access component {} on entity {:?}", comp, e)
            }
            ComponentError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {}
