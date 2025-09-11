# Combat Code Issues - Priority List

Generated: 2025-01-27

## 🎯 **Recently Fixed Issues**
- ✅ Move queuing after killing all enemies (combat state validation)
- ✅ Duplicate queue cleanup logic (centralized cleanup function)
- ✅ Queue cleanup not removing actions targeting dead entities
- ✅ **Race Condition in Action Queue Processing** - Refactored to two-phase processing
- ✅ **No Validation of Action Targets During Execution** - Added target validation checks

## 🔍 **Outstanding Issues by Severity**

### 🚨 **HIGH SEVERITY**

*All high severity issues have been resolved! 🎉*

### 🟡 **MEDIUM SEVERITY**

#### 3. **Inconsistent Combat State Management** ⚡ *Medium Effort*
- **Location**: Multiple files - combat state checked in various places
- **Issue**: Combat state is checked/set in multiple places with different conditions
- **Impact**: Possible states where combat UI shows but combat logic isn't active
- **Fix**: Centralize combat state management in a single system
- **Code Changes**: Create `CombatStateManager` system, consolidate all state transitions

#### 4. **Performance: O(n²) Queue Cleanup** ⚡ *Medium Effort*
- **Location**: `cleanup_actions_targeting_dead_entity()` in `/home/melty/Git/LithicRivers/crates/core/src/systems.rs:1030`
- **Issue**: Every entity death iterates through ALL entities with queues 
- **Impact**: Performance degrades with many entities in large battles
- **Fix**: Maintain reverse lookup: `HashMap<Entity, Vec<Entity>>` of who targets whom
- **Code Changes**: Add targeting relationship tracking component/system

#### 5. **Memory Leak: Dead Entities with Components** ⚡ *Low Effort*
- **Location**: Entity cleanup throughout codebase
- **Issue**: Dead entities keep their ActionQueue and other components until manually cleaned
- **Impact**: Memory usage grows during long combat sessions  
- **Fix**: Add periodic cleanup system or immediate component removal on death
- **Code Changes**: Extend cleanup functions to remove all unnecessary components

#### 6. **No Action Priority System** ⚡ *High Effort*
- **Location**: `ActionQueue` system in `/home/melty/Git/LithicRivers/crates/core/src/moves.rs:278`
- **Issue**: All actions execute in timing order only, no tactical priority
- **Impact**: Unrealistic combat where healing can't interrupt attacks
- **Fix**: Add priority levels to actions and priority-based queue sorting
- **Code Changes**: Major refactor of action queue to support priority levels

### 🟢 **LOW SEVERITY**

#### 7. **UI Shows Entity IDs Instead of Names** ⚡ *Low Effort*
- **Location**: Combat UI in `/home/melty/Git/LithicRivers/crates/client/src/ui/panels/combat.rs:222`
- **Issue**: Target display shows "Enemy 42" instead of "Goblin" or similar
- **Impact**: Poor user experience, harder to track specific enemies
- **Fix**: Add name component lookup in UI target display logic
- **Code Changes**: Add Name component and lookup in UI rendering

#### 8. **No Action Animation/Feedback Delay** ⚡ *Medium Effort*  
- **Location**: Action execution system
- **Issue**: Actions complete instantly with no visual feedback period
- **Impact**: Combat feels disconnected, hard to follow action flow
- **Fix**: Add animation/feedback phase to action execution
- **Code Changes**: Add animation states and timing to action system

#### 9. **Hard-coded Move Validation** ⚡ *Medium Effort*
- **Location**: Input handler `/home/melty/Git/LithicRivers/crates/client/src/app/input_handler.rs:677`
- **Issue**: Move #3 is hard-coded as "Escape" in input handler
- **Impact**: Fragile code, breaks if move list changes
- **Fix**: Add move metadata for target requirements
- **Code Changes**: Add `requires_target` field to Move struct

#### 10. **Limited Error Recovery** ⚡ *Low Effort*
- **Location**: Various action execution paths
- **Issue**: Failed actions don't provide recovery mechanisms
- **Impact**: Players can get stuck in invalid states
- **Fix**: Add action validation and error state recovery
- **Code Changes**: Add validation checks and fallback behaviors

## 📊 **Summary Statistics**

### By Severity:
- **High**: 0 issues ✅ (All resolved!)
- **Medium**: 4 issues (performance/UX problems) 
- **Low**: 4 issues (polish/maintainability)

### By Effort:
- **Low Effort**: 4 issues (quick fixes)
- **Medium Effort**: 5 issues (moderate refactoring)
- **High Effort**: 1 issue (major feature addition)

## 🎯 **Recommended Fix Order**

### Phase 1 - Critical Fixes (High Impact, Lower Effort)
1. **Action target validation** (High Severity, Low Effort) - Prevents crashes
2. **Memory leak cleanup** (Medium Severity, Low Effort) - Performance improvement
3. **UI entity names** (Low Severity, Low Effort) - Quick UX win

### Phase 2 - Performance & Stability (Medium Effort)
4. **Race condition fix** (High Severity, Medium Effort) - System stability
5. **Queue cleanup performance** (Medium Severity, Medium Effort) - Scalability
6. **Combat state centralization** (Medium Severity, Medium Effort) - Maintainability

### Phase 3 - Polish & Features (Higher Effort)
7. **Hard-coded move validation** (Low Severity, Medium Effort) - Code quality
8. **Animation system** (Low Severity, Medium Effort) - UX improvement
9. **Error recovery** (Low Severity, Low Effort) - Robustness
10. **Priority system** (Medium Severity, High Effort) - Major feature

## 🔧 **Technical Notes**

- All high severity issues should be addressed before major feature additions
- The race condition fix (#1) is the most critical as it affects system stability
- Performance issues (#4) will become more apparent with larger battles
- Priority system (#6) would be a good candidate for a major refactor milestone