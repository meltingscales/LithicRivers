"""
Body modularity system for the damaged android concept.
Copyright (c) 2024 HenryFBP. All rights reserved.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Tuple

from lithicrivers.model.vector import VectorN


class BodyPartType(Enum):
    """Types of body parts that can be damaged or missing."""
    HEAD = "head"
    TORSO = "torso"
    LEFT_ARM = "left_arm"
    RIGHT_ARM = "right_arm"
    LEFT_LEG = "left_leg"
    RIGHT_LEG = "right_leg"


class BodyPartState(Enum):
    """States a body part can be in."""
    MISSING = "missing"  # Completely gone
    DAMAGED = "damaged"  # Functional but impaired
    FUNCTIONAL = "functional"  # Working normally
    ENHANCED = "enhanced"  # Better than normal


@dataclass
class BodyPart:
    """Represents a single body part with its state and effects."""
    part_type: BodyPartType
    state: BodyPartState
    name: str
    description: str
    
    # Stat modifiers when this part is in different states
    walk_speed_modifier: float = 1.0
    break_speed_modifier: float = 1.0
    health_modifier: int = 0
    stamina_modifier: int = 0
    
    def get_walk_speed_modifier(self) -> float:
        """Get the walk speed modifier based on current state."""
        if self.state == BodyPartState.MISSING:
            return 0.0
        elif self.state == BodyPartState.DAMAGED:
            return self.walk_speed_modifier * 0.5
        elif self.state == BodyPartState.FUNCTIONAL:
            return self.walk_speed_modifier
        elif self.state == BodyPartState.ENHANCED:
            return self.walk_speed_modifier * 1.5
        return 1.0
    
    def get_break_speed_modifier(self) -> float:
        """Get the break speed modifier based on current state."""
        if self.state == BodyPartState.MISSING:
            return 0.0
        elif self.state == BodyPartState.DAMAGED:
            return self.break_speed_modifier * 0.3
        elif self.state == BodyPartState.FUNCTIONAL:
            return self.break_speed_modifier
        elif self.state == BodyPartState.ENHANCED:
            return self.break_speed_modifier * 1.5
        return 1.0
    
    def get_health_modifier(self) -> int:
        """Get the health modifier based on current state."""
        if self.state == BodyPartState.MISSING:
            return self.health_modifier - 20
        elif self.state == BodyPartState.DAMAGED:
            return self.health_modifier - 10
        elif self.state == BodyPartState.FUNCTIONAL:
            return self.health_modifier
        elif self.state == BodyPartState.ENHANCED:
            return self.health_modifier + 10
        return 0
    
    def get_stamina_modifier(self) -> int:
        """Get the stamina modifier based on current state."""
        if self.state == BodyPartState.MISSING:
            return self.stamina_modifier - 20
        elif self.state == BodyPartState.DAMAGED:
            return self.stamina_modifier - 10
        elif self.state == BodyPartState.FUNCTIONAL:
            return self.stamina_modifier
        elif self.state == BodyPartState.ENHANCED:
            return self.stamina_modifier + 10
        return 0
    
    def can_perform_action(self, action_type: str) -> bool:
        """Check if this body part can perform a specific action."""
        if self.state == BodyPartState.MISSING:
            return False
        
        # Define which parts are needed for which actions
        action_requirements = {
            "walk": [BodyPartType.LEFT_LEG, BodyPartType.RIGHT_LEG],
            "mine": [BodyPartType.LEFT_ARM, BodyPartType.RIGHT_ARM],
            "craft": [BodyPartType.LEFT_ARM, BodyPartType.RIGHT_ARM],
            "push": [BodyPartType.LEFT_ARM, BodyPartType.RIGHT_ARM],
            "interact": [BodyPartType.LEFT_ARM, BodyPartType.RIGHT_ARM],
        }
        
        return self.part_type in action_requirements.get(action_type, [])
    
    def get_repair_cost(self) -> Dict[str, int]:
        """Get the materials needed to repair this body part."""
        base_costs = {
            BodyPartType.HEAD: {"scrap_electronics": 2, "iron_scrap": 1},
            BodyPartType.TORSO: {"iron_scrap": 3, "scrap_electronics": 1},
            BodyPartType.LEFT_ARM: {"iron_scrap": 2, "scrap_electronics": 1},
            BodyPartType.RIGHT_ARM: {"iron_scrap": 2, "scrap_electronics": 1},
            BodyPartType.LEFT_LEG: {"iron_scrap": 2, "scrap_electronics": 1},
            BodyPartType.RIGHT_LEG: {"iron_scrap": 2, "scrap_electronics": 1},
        }
        
        if self.state == BodyPartState.MISSING:
            # Missing parts cost more to replace
            cost = base_costs.get(self.part_type, {})
            return {k: v * 2 for k, v in cost.items()}
        elif self.state == BodyPartState.DAMAGED:
            # Damaged parts cost less to repair
            cost = base_costs.get(self.part_type, {})
            return {k: v // 2 for k, v in cost.items()}
        else:
            return {}


@dataclass
class Body:
    """Represents the complete body of the android with all its parts."""
    parts: Dict[BodyPartType, BodyPart] = field(default_factory=dict)
    
    def __post_init__(self):
        """Initialize with default damaged android body."""
        if not self.parts:
            self._create_default_damaged_body()
    
    def _create_default_damaged_body(self):
        """Create the default damaged android body."""
        self.parts = {
            BodyPartType.HEAD: BodyPart(
                part_type=BodyPartType.HEAD,
                state=BodyPartState.FUNCTIONAL,
                name="Head",
                description="Your primary processing unit. Surprisingly intact.",
                walk_speed_modifier=1.0,
                break_speed_modifier=1.0,
                health_modifier=20,
                stamina_modifier=10
            ),
            BodyPartType.TORSO: BodyPart(
                part_type=BodyPartType.TORSO,
                state=BodyPartState.DAMAGED,
                name="Torso",
                description="Your main body. Some internal components are damaged.",
                walk_speed_modifier=1.0,
                break_speed_modifier=1.0,
                health_modifier=30,
                stamina_modifier=20
            ),
            BodyPartType.LEFT_ARM: BodyPart(
                part_type=BodyPartType.LEFT_ARM,
                state=BodyPartState.FUNCTIONAL,
                name="Left Arm",
                description="Your left arm. Working normally.",
                walk_speed_modifier=0.0,
                break_speed_modifier=1.0,
                health_modifier=0,
                stamina_modifier=0
            ),
            BodyPartType.RIGHT_ARM: BodyPart(
                part_type=BodyPartType.RIGHT_ARM,
                state=BodyPartState.MISSING,
                name="Right Arm",
                description="Your right arm. Completely missing.",
                walk_speed_modifier=0.0,
                break_speed_modifier=0.0,
                health_modifier=0,
                stamina_modifier=0
            ),
            BodyPartType.LEFT_LEG: BodyPart(
                part_type=BodyPartType.LEFT_LEG,
                state=BodyPartState.FUNCTIONAL,
                name="Left Leg",
                description="Your left leg. Working normally.",
                walk_speed_modifier=1.0,
                break_speed_modifier=0.0,
                health_modifier=0,
                stamina_modifier=0
            ),
            BodyPartType.RIGHT_LEG: BodyPart(
                part_type=BodyPartType.RIGHT_LEG,
                state=BodyPartState.DAMAGED,
                name="Right Leg",
                description="Your right leg. Damaged and slow.",
                walk_speed_modifier=0.5,
                break_speed_modifier=0.0,
                health_modifier=0,
                stamina_modifier=0
            ),
        }
    
    def get_total_walk_speed_modifier(self) -> float:
        """Calculate total walk speed modifier from all legs."""
        total = 0.0
        leg_parts = [self.parts.get(BodyPartType.LEFT_LEG), self.parts.get(BodyPartType.RIGHT_LEG)]
        
        for leg in leg_parts:
            if leg:
                total += leg.get_walk_speed_modifier()
        
        # Normalize to 1.0 for two functional legs
        return max(0.1, total / 2.0)  # Minimum 10% speed
    
    def get_total_break_speed_modifier(self) -> float:
        """Calculate total break speed modifier from all arms."""
        total = 0.0
        arm_parts = [self.parts.get(BodyPartType.LEFT_ARM), self.parts.get(BodyPartType.RIGHT_ARM)]
        
        for arm in arm_parts:
            if arm:
                total += arm.get_break_speed_modifier()
        
        # Normalize to 1.0 for two functional arms
        return max(0.1, total / 2.0)  # Minimum 10% speed
    
    def get_total_health_modifier(self) -> int:
        """Calculate total health modifier from all body parts."""
        total = 0
        for part in self.parts.values():
            total += part.get_health_modifier()
        return total
    
    def get_total_stamina_modifier(self) -> int:
        """Calculate total stamina modifier from all body parts."""
        total = 0
        for part in self.parts.values():
            total += part.get_stamina_modifier()
        return total
    
    def can_perform_action(self, action_type: str) -> bool:
        """Check if the body can perform a specific action."""
        for part in self.parts.values():
            if part.can_perform_action(action_type):
                return True
        return False
    
    def get_action_speed(self, action_type: str) -> float:
        """Get the speed modifier for a specific action."""
        if action_type == "walk":
            return self.get_total_walk_speed_modifier()
        elif action_type == "mine":
            return self.get_total_break_speed_modifier()
        elif action_type == "craft":
            return self.get_total_break_speed_modifier()
        elif action_type == "push":
            return self.get_total_break_speed_modifier()
        else:
            return 1.0
    
    def get_repair_requirements(self) -> Dict[BodyPartType, Dict[str, int]]:
        """Get repair requirements for all damaged/missing parts."""
        requirements = {}
        for part_type, part in self.parts.items():
            if part.state in [BodyPartState.DAMAGED, BodyPartState.MISSING]:
                requirements[part_type] = part.get_repair_cost()
        return requirements
    
    def get_body_status_summary(self) -> str:
        """Get a summary of the body's current state."""
        status_lines = []
        for part_type, part in self.parts.items():
            state_icon = {
                BodyPartState.MISSING: "[X]",
                BodyPartState.DAMAGED: "[!]",
                BodyPartState.FUNCTIONAL: "[OK]",
                BodyPartState.ENHANCED: "[*]"
            }.get(part.state, "[?]")
            
            status_lines.append(f"{state_icon} {part.name}: {part.state.value}")
        
        return "\n".join(status_lines)
    
    def get_movement_penalty_description(self) -> str:
        """Get a description of current movement penalties."""
        walk_speed = self.get_total_walk_speed_modifier()
        break_speed = self.get_total_break_speed_modifier()
        
        descriptions = []
        
        if walk_speed < 0.5:
            descriptions.append("Severely impaired movement")
        elif walk_speed < 0.8:
            descriptions.append("Impaired movement")
        
        if break_speed < 0.5:
            descriptions.append("Severely impaired actions")
        elif break_speed < 0.8:
            descriptions.append("Impaired actions")
        
        if not descriptions:
            return "Normal operation"
        
        return "; ".join(descriptions) 