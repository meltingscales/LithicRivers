use crate::components::ItemKind;
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
