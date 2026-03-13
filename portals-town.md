# Portals To Town

## Goal
Implement unlockable portals after first town arrival, convert out-of-town HUD exit button into a portal action with loading behavior, add a subtle old-school glowy transition for area-to-town travel once unlocked, and rename the town exit button.

## Confirmed Decisions
1. Portal unlock is a save-state flag and persists across reloads.
2. `Portal To Town` appears only when outside town, and clicking it resets action timer and turns the next action into portal travel (no normal action execution first).
3. Pending portal travel is canceled if an interrupting state happens before completion (death/boss/fruit scene).
4. Once unlocked, any area-to-town travel uses the new subtle portal transition; boss portal visuals remain separate.
