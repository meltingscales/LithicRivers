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
            // Stone Axe: 2x Stone + 1x Stick
            Recipe {
                name: "Stone Axe",
                ingredients: &[(ItemKind::Stone, 2), (ItemKind::Stick, 1)],
                result: ItemKind::StoneAxe,
                quantity: 1,
            },
            // Stone Pickaxe: 3x Stone + 2x Stick
            Recipe {
                name: "Stone Pickaxe",
                ingredients: &[(ItemKind::Stone, 3), (ItemKind::Stick, 2)],
                result: ItemKind::StonePickaxe,
                quantity: 1,
            },
            // Torch: 1x Stick + 1x String
            Recipe {
                name: "Torch",
                ingredients: &[(ItemKind::Stick, 1), (ItemKind::String, 1)],
                result: ItemKind::Torch,
                quantity: 4,
            },
            // Rope: 3x String
            Recipe {
                name: "Rope",
                ingredients: &[(ItemKind::String, 3)],
                result: ItemKind::Rope,
                quantity: 1,
            },
            // Iron Bar: 2x Iron Ore
            Recipe {
                name: "Iron Bar",
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

        // Find Stone Axe recipe by name
        let stone_axe_recipe = handler
            .find_recipe_by_name("Stone Axe")
            .expect("Stone Axe recipe should exist");

        // Should be able to craft Stone Axe
        assert!(handler.can_craft(stone_axe_recipe, &inventory));

        // Craft Stone Axe
        assert!(handler.craft(stone_axe_recipe, &mut inventory));
        assert_eq!(inventory[&ItemKind::Stone], 1);
        assert_eq!(inventory[&ItemKind::Stick], 1);
        assert_eq!(inventory[&ItemKind::StoneAxe], 1);

        // Not enough materials for another Stone Axe
        assert!(!handler.can_craft(stone_axe_recipe, &inventory));
    }
}
