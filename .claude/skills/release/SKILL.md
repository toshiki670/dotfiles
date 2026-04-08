---
name: release
description: This skill should be used when the user asks to "/release", "run release", "create a release", "do a release", or "リリースする". Determines the appropriate version bump (minor or patch) based on unreleased PRs and triggers the GitHub Actions release workflow.
version: 0.1.0
---

# Release Skill

Automate releases for the dotfiles repository by analyzing unreleased PRs and triggering the appropriate GitHub Actions workflow.

## Workflow

### Step 1: Get Unreleased PRs

```bash
LAST_TAG=$(git describe --tags --abbrev=0)
git log ${LAST_TAG}..HEAD --oneline --merges | grep -oE '#[0-9]+' | tr -d '#' | while read pr; do
  gh pr view $pr --json number,title,body,labels --jq '{number: .number, title: .title, body: .body, labels: [.labels[].name]}'
done
```

### Step 2: Determine Version Bump

Read `VERSIONING.md` to understand the versioning rules, then apply them to the unreleased PRs.

If multiple PRs are present, the highest bump wins (minor > patch).

### Step 3: Confirm with User

Present the analysis to the user:
- List of unreleased PRs with titles
- Determined bump type (minor or patch) with reasoning
- Ask for confirmation before triggering the workflow

### Step 4: Trigger Release Workflow

After user confirmation, run the appropriate command:

**For MINOR:**
```bash
gh workflow run release.yml -f release_type=minor
```

**For PATCH:**
```bash
gh workflow run release.yml -f release_type=patch
```

Then verify the workflow was triggered:
```bash
gh run list --workflow=release.yml --limit=3
```

## Notes

- Always show the PR list and reasoning before triggering the workflow.
- Never trigger the workflow without explicit user confirmation.
- If there are no unreleased PRs, inform the user and do not trigger anything.
- The release workflow handles version calculation, tagging, and GitHub Release creation automatically.
