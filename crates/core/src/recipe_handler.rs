use crate::components::ItemKind;
use crate::model::body::{BodyPartState, BodyPartType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Recipe {
    pub name: &'static str,
    pub ingredients: &'static [(ItemKind, u32)],
    pub result: ItemKind,
    pub quantity: u32,
}

pub struct RecipeHandler {
    recipes: Vec<Recipe>,
}

impl RecipeHandler {
    pub fn new() -> Self {
        let recipes = vec![
            // 3 Wooden Planks: 1x Log
            Recipe {
                name: "Wooden Planks",
                ingredients: &[(ItemKind::Log, 1)],
                result: ItemKind::WoodenPlank,
                quantity: 3,
            },
            // 4 Plank Blocks: 1x Wooden Plank
            Recipe {
                name: "Plank Blocks",
                ingredients: &[(ItemKind::WoodenPlank, 1)],
                result: ItemKind::PlankBlock,
                quantity: 4,
            },
            // 2 sticks: 1x plank
            Recipe {
                name: "Sticks",
                ingredients: &[(ItemKind::WoodenPlank, 1)],
                result: ItemKind::Stick,
                quantity: 2,
            },
            // 1 wood shavings: 1x acorn
            Recipe {
                name: "Wooden Shavings from Acorn",
                ingredients: &[(ItemKind::Acorn, 1)],
                result: ItemKind::WoodenShavings,
                quantity: 1,
            },
            // 1 wood shavings: 1x stick
            Recipe {
                name: "Wooden Shavings from Stick",
                ingredients: &[(ItemKind::Stick, 1)],
                result: ItemKind::WoodenShavings,
                quantity: 1,
            },
            // Torch: 1x Stick + 1x String
            Recipe {
                name: "Torch",
                ingredients: &[(ItemKind::Stick, 1), (ItemKind::String, 1)],
                result: ItemKind::Torch,
                quantity: 4,
            },
        ];

        Self { recipes }
    }

    pub fn get_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    pub fn find_recipe_by_name(&self, name: &str) -> Option<usize> {
        self.recipes.iter().position(|r| r.name == name)
    }

    pub fn can_craft(&self, recipe_index: usize, inventory: &HashMap<ItemKind, u32>) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            recipe
                .ingredients
                .iter()
                .all(|&(item, required)| inventory.get(&item).copied().unwrap_or(0) >= required)
        } else {
            false
        }
    }

    pub fn craft(&self, recipe_index: usize, inventory: &mut HashMap<ItemKind, u32>) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // First check if we can craft
            if !self.can_craft(recipe_index, inventory) {
                return false;
            }

            // Consume ingredients
            for &(item, required) in recipe.ingredients {
                *inventory.get_mut(&item).unwrap() -= required;
            }

            // Add result
            *inventory.entry(recipe.result).or_insert(0) += recipe.quantity;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RepairRecipe {
    pub name: &'static str,
    pub ingredients: &'static [(ItemKind, u32)],
    pub durability_restored: i64,
    pub compatible_parts: &'static [BodyPartType], // Empty slice means all parts
}

pub struct RepairRecipeHandler {
    recipes: Vec<RepairRecipe>,
}

impl RepairRecipeHandler {
    pub fn new() -> Self {
        let recipes = vec![RepairRecipe {
            name: "Basic Electronics Repair",
            ingredients: &[(ItemKind::ScrapElectronics, 1)],
            durability_restored: 25,
            compatible_parts: &[], // Works on all parts
        }];

        Self { recipes }
    }

    pub fn get_recipes(&self) -> &[RepairRecipe] {
        &self.recipes
    }

    pub fn can_repair(&self, recipe_index: usize, inventory: &HashMap<ItemKind, u32>) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            recipe
                .ingredients
                .iter()
                .all(|&(item, required)| inventory.get(&item).copied().unwrap_or(0) >= required)
        } else {
            false
        }
    }

    pub fn can_repair_part(
        &self,
        recipe_index: usize,
        part_type: BodyPartType,
        part_state: BodyPartState,
    ) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // Can't repair missing parts
            if part_state == BodyPartState::Missing {
                return false;
            }

            // Check if recipe is compatible with this part type
            recipe.compatible_parts.is_empty() || recipe.compatible_parts.contains(&part_type)
        } else {
            false
        }
    }

    pub fn apply_repair(
        &self,
        recipe_index: usize,
        inventory: &mut HashMap<ItemKind, u32>,
        current_integrity: i64,
    ) -> Option<i64> {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // Check if we can use this recipe
            if !self.can_repair(recipe_index, inventory) {
                return None;
            }

            // Consume ingredients
            for &(item, required) in recipe.ingredients {
                if let Some(count) = inventory.get_mut(&item) {
                    *count = count.saturating_sub(required);
                    if *count == 0 {
                        inventory.remove(&item);
                    }
                }
            }

            // Apply repair (cap at 100)
            Some((current_integrity + recipe.durability_restored).min(100))
        } else {
            None
        }
    }
}
