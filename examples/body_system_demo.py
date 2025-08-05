#!/usr/bin/env python3
"""
Body System Demo for LithicRivers
Copyright (c) 2024 HenryFBP. All rights reserved.

This demo showcases the body modularity system and how it affects gameplay.
"""

from lithicrivers.model.body import Body, BodyPartState, BodyPartType
from lithicrivers.game import Player


def demo_body_system():
    """Demonstrate the body modularity system."""
    print("LITHICRIVERS BODY MODULARITY SYSTEM DEMO")
    print("=" * 50)
    
    # Create a player with the default damaged android body
    player = Player("Damaged Android")
    
    print(f"\nPlayer: {player.name}")
    print(f"[*] Health: {player.health}")
    print(f"[*] Stamina: {player.stamina}")
    
    # Show body status
    print(f"\nBODY STATUS:")
    print(player.get_body_status_summary())
    
    # Show movement penalties
    print(f"\nMOVEMENT PENALTIES:")
    print(f"   {player.get_movement_penalty_description()}")
    
    # Show speed modifiers
    print(f"\nSPEED MODIFIERS:")
    print(f"   Walk Speed: {player.get_walk_speed_modifier():.2f}")
    print(f"   Break Speed: {player.get_break_speed_modifier():.2f}")
    
    # Show action capabilities
    print(f"\nACTION CAPABILITIES:")
    actions = ["walk", "mine", "craft", "push", "interact"]
    for action in actions:
        can_do = player.can_perform_action(action)
        speed = player.get_action_speed(action)
        status = "[OK]" if can_do else "[X]"
        print(f"   {action.upper():8} {status} (speed: {speed:.2f})")
    
    # Show repair requirements
    print(f"\nREPAIR REQUIREMENTS:")
    requirements = player.body.get_repair_requirements()
    if requirements:
        for part_type, costs in requirements.items():
            part_name = part_type.value.replace("_", " ").title()
            cost_str = ", ".join([f"{amt} {item}" for item, amt in costs.items()])
            print(f"   {part_name}: {cost_str}")
    else:
        print("   No repairs needed!")
    
    # Demonstrate body part effects
    print(f"\nDEMONSTRATING BODY PART EFFECTS:")
    
    # Test with different body configurations
    test_configs = [
        ("Fully Functional", {
            BodyPartType.RIGHT_ARM: BodyPartState.FUNCTIONAL,
            BodyPartType.RIGHT_LEG: BodyPartState.FUNCTIONAL,
        }),
        ("Severely Damaged", {
            BodyPartType.LEFT_ARM: BodyPartState.MISSING,
            BodyPartType.LEFT_LEG: BodyPartState.MISSING,
        }),
        ("Enhanced", {
            BodyPartType.RIGHT_ARM: BodyPartState.ENHANCED,
            BodyPartType.RIGHT_LEG: BodyPartState.ENHANCED,
        }),
    ]
    
    for config_name, changes in test_configs:
        print(f"\n   {config_name}:")
        
        # Apply changes to a copy
        test_body = Body()
        for part_type, new_state in changes.items():
            test_body.parts[part_type].state = new_state
        
        walk_speed = test_body.get_total_walk_speed_modifier()
        break_speed = test_body.get_total_break_speed_modifier()
        
        print(f"      Walk Speed: {walk_speed:.2f}")
        print(f"      Break Speed: {break_speed:.2f}")
        
        # Show what actions are possible
        possible_actions = []
        for action in actions:
            if test_body.can_perform_action(action):
                possible_actions.append(action)
        
        print(f"      Possible Actions: {', '.join(possible_actions)}")
    
    print(f"\nGAMEPLAY IMPLICATIONS:")
    print("   • Damaged body parts reduce movement and action speeds")
    print("   • Missing body parts prevent certain actions entirely")
    print("   • Players must repair their body to improve performance")
    print("   • Higher tick rates (200+) allow for more granular speed control")
    print("   • Body condition affects mining, crafting, and movement")
    
    print(f"\nNEXT STEPS FOR MVP:")
    print("   • Implement block placement with body-based restrictions")
    print("   • Add push boxes that require functional arms")
    print("   • Create crafting system for body repairs")
    print("   • Add procedural dungeons with body-based challenges")
    print("   • Implement fluid mechanics affected by body condition")


if __name__ == "__main__":
    demo_body_system() 