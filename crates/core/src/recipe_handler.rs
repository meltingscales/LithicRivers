use crate::components::ItemKind;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Recipe {
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
            // Stone Axe: 2x Stone + 1x Stick
            Recipe {
                ingredients: &[(ItemKind::Stone, 2), (ItemKind::Stick, 1)],
                result: ItemKind::StoneAxe,
                quantity: 1,
            },
            // Stone Pickaxe: 3x Stone + 2x Stick
            Recipe {
                ingredients: &[(ItemKind::Stone, 3), (ItemKind::Stick, 2)],
                result: ItemKind::StonePickaxe,
                quantity: 1,
            },
            // Torch: 1x Stick + 1x String
            Recipe {
                ingredients: &[(ItemKind::Stick, 1), (ItemKind::String, 1)],
                result: ItemKind::Torch,
                quantity: 4,
            },
            // Rope: 3x String
            Recipe {
                ingredients: &[(ItemKind::String, 3)],
                result: ItemKind::Rope,
                quantity: 1,
            },
            // Iron Bar: 2x Iron Ore
            Recipe {
                ingredients: &[(ItemKind::IronOre, 2)],
                result: ItemKind::IronBar,
                quantity: 1,
            },
        ];

        Self { recipes }
    }

    pub fn get_recipes(&self) -> &[Recipe] {
        &self.recipes
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crafting() {
        let handler = RecipeHandler::new();
        let mut inventory = HashMap::new();

        // Add items to inventory
        inventory.insert(ItemKind::Stone, 3);
        inventory.insert(ItemKind::Stick, 2);
        inventory.insert(ItemKind::String, 1);

        // Should be able to craft Stone Axe
        assert!(handler.can_craft(0, &inventory));

        // Craft Stone Axe
        assert!(handler.craft(0, &mut inventory));
        assert_eq!(inventory[&ItemKind::Stone], 1);
        assert_eq!(inventory[&ItemKind::Stick], 1);
        assert_eq!(inventory[&ItemKind::StoneAxe], 1);

        // Not enough materials for another Stone Axe
        assert!(!handler.can_craft(0, &inventory));
    }
}
