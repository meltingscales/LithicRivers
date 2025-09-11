#[cfg(test)]
mod tests {
    use crate::components::Position;
    use crate::moves::*;

    #[test]
    fn test_tackle_push_calculation() {
        let player_pos = Position { x: 0, y: 0, z: 0 };

        // Test tackle push east
        let enemy_east = Position { x: 1, y: 0, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_east, 2);
        assert_eq!(pushed, Position { x: 3, y: 0, z: 0 });

        // Test tackle push west
        let enemy_west = Position { x: -1, y: 0, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_west, 2);
        assert_eq!(pushed, Position { x: -3, y: 0, z: 0 });

        // Test tackle push north
        let enemy_north = Position { x: 0, y: 1, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_north, 2);
        assert_eq!(pushed, Position { x: 0, y: 3, z: 0 });

        // Test tackle push south
        let enemy_south = Position { x: 0, y: -1, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_south, 2);
        assert_eq!(pushed, Position { x: 0, y: -3, z: 0 });

        // Test diagonal - should prefer horizontal movement
        let enemy_ne = Position { x: 2, y: 1, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_ne, 2);
        assert_eq!(pushed, Position { x: 4, y: 1, z: 0 }); // Pushed east, not north

        // Test diagonal - should prefer vertical movement when dy > dx
        let enemy_steep_north = Position { x: 1, y: 3, z: 0 };
        let pushed = calculate_push_position(player_pos, enemy_steep_north, 2);
        assert_eq!(pushed, Position { x: 1, y: 5, z: 0 }); // Pushed north, not east
    }

    #[test]
    fn test_tackle_specifications() {
        let tackle = Move::tackle();

        // Verify tackle meets all specifications from MVP document
        assert_eq!(tackle.damage, 15, "Tackle should do 15 damage");
        assert_eq!(tackle.energy_cost, 20, "Tackle should cost 20 energy");
        assert_eq!(
            tackle.execution_time_ticks, 10,
            "Tackle should take 10 ticks to execute"
        );
        assert_eq!(tackle.move_type, MoveType::Tackle);

        // Verify description mentions key mechanics
        assert!(
            tackle.description.contains("Pushes enemy back 2 spaces"),
            "Description should mention 2-space push"
        );
        assert!(
            tackle.description.contains("50% chance to stun"),
            "Description should mention 50% stun chance"
        );
        assert!(
            tackle.description.contains("600 ticks"),
            "Description should mention 600 tick stun duration"
        );
    }

    #[test]
    fn test_all_combat_moves_have_valid_properties() {
        let moves = get_available_moves();

        for mv in moves {
            // All moves should have non-empty names and descriptions
            assert!(!mv.name.is_empty(), "Move {} has empty name", mv.name);
            assert!(
                !mv.description.is_empty(),
                "Move {} has empty description",
                mv.name
            );

            // Execution time should be reasonable (1-20 ticks)
            assert!(
                mv.execution_time_ticks > 0,
                "Move {} has zero execution time",
                mv.name
            );
            assert!(
                mv.execution_time_ticks <= 20,
                "Move {} has excessive execution time",
                mv.name
            );

            // Energy cost should be reasonable (0-100)
            assert!(
                mv.energy_cost <= 100,
                "Move {} has excessive energy cost",
                mv.name
            );

            // Damage should be reasonable (0-10000, allowing for debug moves)
            assert!(mv.damage <= 10000, "Move {} has excessive damage", mv.name);

            // Splash radius, if present, should be reasonable
            if let Some(radius) = mv.splash_radius {
                assert!(
                    radius > 0,
                    "Move {} has non-positive splash radius",
                    mv.name
                );
                assert!(radius <= 5, "Move {} has excessive splash radius", mv.name);
            }
        }
    }

    #[test]
    fn test_move_type_human_names_are_unique() {
        let moves = get_available_moves();
        let mut names = std::collections::HashSet::new();

        for mv in moves {
            let human_name = mv.move_type.human_name();
            assert!(
                names.insert(human_name),
                "Duplicate human name found: {}",
                human_name
            );
        }
    }

    #[test]
    fn test_escape_move_properties() {
        let escape = Move::escape();

        // Escape should do no damage
        assert_eq!(escape.damage, 0, "Escape should do no damage");

        // Escape should have reasonable cost and timing
        assert_eq!(escape.energy_cost, 20, "Escape should cost 20 energy");
        assert_eq!(
            escape.execution_time_ticks, 15,
            "Escape should take 15 ticks"
        );

        // Should mention key mechanics
        assert!(
            escape.description.contains("combat"),
            "Escape description should mention combat"
        );
        assert!(
            escape.description.contains("BattleDelay") || escape.description.contains("delay"),
            "Escape description should mention delay mechanic"
        );
    }

    #[test]
    fn test_fireball_aoe_properties() {
        let fireball = Move::fireball();

        // Fireball should have splash damage
        assert!(
            fireball.splash_radius.is_some(),
            "Fireball should have splash radius"
        );
        assert_eq!(
            fireball.splash_radius.unwrap(),
            1,
            "Fireball splash radius should be 1"
        );

        // Should be appropriately powerful but costly
        assert_eq!(fireball.damage, 30, "Fireball should do 30 damage");
        assert_eq!(fireball.energy_cost, 40, "Fireball should cost 40 energy");

        // Should mention AoE in name/description
        assert!(
            fireball.name.contains("AoE") || fireball.description.contains("area"),
            "Fireball should mention area of effect"
        );
    }

    #[test]
    fn test_action_queue_clears_correctly() {
        let mut queue = ActionQueue::new();

        // Add some actions
        queue.queue_action(QueuedAction {
            entity: hecs::Entity::DANGLING,
            action: CombatAction::PlayerMove {
                move_data: Move::melee(),
                target_entity: None,
                target_position: None,
            },
            execution_time_ticks: 5,
            remaining_time_ticks: 5,
        });

        queue.queue_action(QueuedAction {
            entity: hecs::Entity::DANGLING,
            action: CombatAction::PlayerMove {
                move_data: Move::tackle(),
                target_entity: None,
                target_position: None,
            },
            execution_time_ticks: 10,
            remaining_time_ticks: 10,
        });

        assert_eq!(queue.actions.len(), 2);

        // Clear the queue
        queue.clear();

        // Should be empty
        assert_eq!(queue.actions.len(), 0);
        assert!(queue.current_action.is_none());
    }

    #[test]
    fn test_damage_random_body_part_deterministic() {
        use crate::model::body::Body;

        let mut body1 = Body::default();
        let body1_clone = body1.clone(); // Create identical copy
        let mut body2 = body1_clone;

        // Same seed and tick should produce same results
        let seed = 12345;
        let tick = 100;
        let damage = 25;

        let part1 = damage_random_body_part(&mut body1, seed, tick, damage);
        let part2 = damage_random_body_part(&mut body2, seed, tick, damage);

        assert_eq!(part1, part2, "Same seed/tick should damage same body part");

        // Body states should match after applying the same damage
        for (part_type, part) in body1.parts.iter() {
            let other_part = &body2.parts[part_type];
            assert_eq!(
                part.state, other_part.state,
                "Body part {:?} states should match",
                part_type
            );
            assert_eq!(
                part.integrity, other_part.integrity,
                "Body part {:?} integrity should match",
                part_type
            );
        }
    }
}
