| Class | Field | Likely Cycle? | Fix Suggestion | 
|---------------|--------------------|---------------|-----------------------------------------| 
| Game | world | YES | OK, but check if world → game exists | 
| World | game | YES | Remove from struct fields, set manually | 
| World | entities | YES | OK, but check if entity → world exists | 
| Entity | world | YES | Remove from struct fields, set manually | 
| ChunkCache | world | YES | Remove from struct fields, set manually | 
| StructureMgr | world/game | YES | Remove from struct fields, set manually |


1. Game.world exists and it's a World instance.
2. World.game does not exist.
3. World.entities_by_position exists and it's a dict[tuple[int, int, int], list[Entity]]
4. Entity.world does not exist.
5. ChunkCache.world does not exist.
6. StructureManager.world does not exist.


(see find-cycles makefile target...)