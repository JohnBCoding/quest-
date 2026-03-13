# AGENTS.md

## Scope
These rules apply to the entire repository tree rooted at this file.

## Absolute Requirements (No Exceptions)
1. Read the FULL global skill at:
    `/home/johnbcoding/.codex/skills/global/SKILL.md`
    Do not skim. Do not partially apply.
2. Pass every gate and protocol defined in that global skill, in its required order.
3. Follow all mandatory steps exactly as written ("MUST", "MANDATORY", "CRITICAL", "STOP", "NEVER").

## Required Execution Order
1. Load and fully read global skill.
2. Execute required environment/worktree setup from global skill.
3. Run the required Socratic gate/questions from global skill and wait for user answers when required.
4. Classify request type per global skill.
5. Perform implementation only after all required gates are satisfied.
6. Complete required finish flow from global skill (including PR flow if required).

## Hard Stops
- If current branch is `main`, STOP. Create/switch to the required worktree branch first.
- If Socratic gate is required and not completed, STOP. Ask required questions first.
- If any required step is unclear, STOP and ask the user before proceeding.
- Never bypass gates because the task seems “simple”.

## Compliance Rule
If any instruction in this file conflicts with behavior, enforce this file + FULL global skill literally.
Default behavior is to halt and request clarification rather than proceed non-compliantly.

## Rust escalation 
Due to how the Rust environment is setup, you will have to use network access via escalation to do
certain things like running tests and pushing PRs. You have permission to do this.