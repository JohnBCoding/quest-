---
name: finishing-a-development-branch
description: Use when implementation is complete, all tests pass, and you need to decide how to integrate the work
---

# Finishing a Development Branch

## Overview

Guide completion of development work by presenting clear options and handling chosen workflow.

**Core principle:** Verify tests → Execute PR → Clean up.

**Announce at start:** "I'm using the finishing-a-development-branch skill to complete this work."

## The Process

### Step 1: Verify Tests

**Before presenting options, verify tests pass:**

```bash
# Run project's test suite
npm test / cargo test / pytest / go test ./...
```

**If tests fail:**
```
Tests failing (<N> failures). Must fix before completing:

[Show failures]

Cannot proceed with merge/PR until tests pass.
```

Stop. Don't proceed to Step 2.

**If tests pass:** Continue to Step 2.

### Step 2: Determine Base Branch

```bash
# Try common base branches
git merge-base HEAD main 2>/dev/null || git merge-base HEAD master 2>/dev/null
```

### Step 3: Execute PR

#### Push and Create PR

```bash
# Push branch
git push -u origin <feature-branch>

# Create PR
gh pr create --title "<title>" --body "$(cat <<'EOF'
## Summary
<2-3 bullets of what changed>

## Test Plan
- [ ] <verification steps>
EOF
)"
```

### Step 4: Wait for PR Acceptance -> Cleanup

#### Check Every 5 minutes For PR Status

```bash
# Check PR status
sleep 300 && echo "--- PR Status ---" && gh pr status **PR_NUM** && echo -e "\n--- Latest Comments ---" && gh pr view **PR_NUM** --comments
```

#### Cleanup

```bash
# Delete branch
git branch -d <feature-branch>

# Delete remote branch
git push origin --delete <feature-branch>

# Delete worktree
git worktree remove <path>
```

## Common Mistakes

**Merging PR Yourself**
- **Problem:** You want to merge the PR yourself
- **Fix:** Wait for the PR to be merged by the user, then cleanup

**Skipping test verification**
- **Problem:** Merge broken code, create failing PR
- **Fix:** Always verify tests before offering options

**Open-ended questions**
- **Problem:** "What should I do next?" → ambiguous
- **Fix:** Present exactly 4 structured options

**Automatic worktree cleanup**
- **Problem:** Remove worktree when might need it (Option 2, 3)
- **Fix:** Only cleanup when told to.

**No confirmation for discard**
- **Problem:** Accidentally delete work
- **Fix:** Require typed "discard" confirmation

## Red Flags

**Never:**
- Proceed with failing tests
- Merge without verifying tests on result
- Delete work without confirmation

**Always:**
- Verify tests before doing PR

## Integration

**Pairs with:**
- **using-git-worktrees** - Cleans up worktree created by that skill
