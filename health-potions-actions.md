# Health Potions + Action Priority

## Goal
Implement health potions unlocked with the first Rat Lord fruit reward, add action priority configuration, and ensure player action animation priority over mob damage on same tick.

## Confirmed Decisions
1. Health potions start with 5 uses and refill automatically at town.
2. If potion cannot trigger, action system falls through to next priority.
3. Action config persists in save data.
4. Same-tick ordering: player action visual resolves before mob damage visual.
5. Resolve local cargo cross-device error to run verification.
