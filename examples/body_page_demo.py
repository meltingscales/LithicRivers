#!/usr/bin/env python3
"""
BodyPage Demo for LithicRivers
Copyright (c) 2024 HenryFBP. All rights reserved.

This demo shows what the BodyPage displays.
"""

from lithicrivers.game import Game
from lithicrivers.settings import DEFAULT_SEED


def demo_body_page_content():
    """Show what the BodyPage would display."""
    print("🤖 BODYPAGE CONTENT DEMO")
    print("=" * 50)
    
    # Create a game with player
    game = Game(seed=DEFAULT_SEED)
    player = game.player
    
    print(f"\n👤 Player: {player.name}")
    print(f"❤️  Health: {player.health}")
    print(f"⚡ Stamina: {player.stamina}")
    
    print(f"\n🔧 BODY PARTS:")
    body_summary = player.get_body_status_summary()
    for line in body_summary.split('\n'):
        print(f"   {line}")
    
    print(f"\n🚶 MOVEMENT STATUS:")
    penalty_desc = player.get_movement_penalty_description()
    print(f"   {penalty_desc}")
    
    print(f"\n⚡ SPEED MODIFIERS:")
    walk_speed = player.get_walk_speed_modifier()
    break_speed = player.get_break_speed_modifier()
    print(f"   Walk Speed: {walk_speed:.2f}")
    print(f"   Break Speed: {break_speed:.2f}")
    
    print(f"\n🎯 ACTION CAPABILITIES:")
    actions = ["walk", "mine", "craft", "push", "interact"]
    for action in actions:
        can_do = player.can_perform_action(action)
        speed = player.get_action_speed(action)
        status = "✅" if can_do else "❌"
        print(f"   {action.upper():8} {status} (speed: {speed:.2f})")
    
    print(f"\n🔧 REPAIR REQUIREMENTS:")
    requirements = player.body.get_repair_requirements()
    if requirements:
        for part_type, costs in requirements.items():
            part_name = part_type.value.replace("_", " ").title()
            cost_str = ", ".join([f"{amt} {item}" for item, amt in costs.items()])
            print(f"   {part_name}: {cost_str}")
    else:
        print("   No repairs needed!")
    
    print(f"\n📝 BODY DESCRIPTIONS:")
    for part_type, part in player.body.parts.items():
        part_name = part.name
        description = part.description
        print(f"   {part_name}: {description}")
    
    print(f"\n🎮 IN-GAME ACCESS:")
    print("   • Press 'B' to open the Body page")
    print("   • View your android's current condition")
    print("   • See what actions you can perform")
    print("   • Check repair requirements")
    print("   • Monitor your movement penalties")


if __name__ == "__main__":
    demo_body_page_content() 