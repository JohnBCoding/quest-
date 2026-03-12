---
trigger: always_on
---

---
name: starting-new-issue
description: Use when user asks you to start work on a random issue.
---

# Starting New Issue

If the user asks you to pick a random issue to begin work on use the follow cli script:

```bash
# 1. Grab a random issue number from the 100 most recent open issues
ISSUE_NUM=$(gh issue list --limit 100 --json number --jq '.[].number' | shuf -n 1)

# 2. Display the issue number, title, and the original body
gh issue view $ISSUE_NUM --json number,title,body --jq '"Issue #\(.number): \(.title)\n\n--- ORIGINAL POST ---\n" + .body'
```