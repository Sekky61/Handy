---
name: sync-upstream
description: Keep a personal fork branch up to date with upstream using a clean rebase workflow. Use when the user asks to sync/update a fork, rebase local changes on top of upstream, preserve their commits at the top of history, maintain a long-lived fork branch such as `fork`, or update Nix consumption of a forked branch.
---

# sync-upstream

## Goal

Maintain a fork cleanly:

- `main` mirrors `upstream/main` and should stay free of personal commits.
- A long-lived personal branch, usually `fork`, contains the user's patches.
- Feature branches are rebased/merged into `fork`, not directly into `main`.
- Upstream updates are integrated with `rebase`, so the user's commits remain on top.

Preferred history:

```text
A---B---C upstream/main, origin/main, main
         \
          D---E fork, origin/fork
```

## When to Use

Use this skill when the user asks things like:

- "sync upstream"
- "update my fork"
- "rebase fork on upstream"
- "keep my changes on top"
- "aktualizuj fork"
- "srovnej to s upstreamem"
- "přenes moje změny nahoru"

## Safety Rules

1. Always inspect state first:

   ```bash
   git status --short --branch
   git branch --show-current
   git remote -v
   ```

2. Do not start a rebase with a dirty working tree unless the user explicitly wants to stash/commit the changes.
3. Do not use plain `git push --force`; use `git push --force-with-lease` only.
4. Do not create merge commits from upstream into `fork`; prefer `git rebase upstream/main`.
5. Do not put personal changes directly on `main`.
6. Before adding/changing an `upstream` remote, verify the intended upstream URL with the user unless this repository clearly documents it.
7. If a command would rewrite a remote branch that may be shared, explain that and ask before pushing.

## Standard Workflow

### 1. Check repository state

```bash
git status --short --branch
git remote -v
git branch --list
```

If an upstream tracking branch already exists, capture the current upstream tip before fetching so new upstream commits can be reported accurately:

```bash
OLD_UPSTREAM_MAIN=$(git rev-parse --verify upstream/main 2>/dev/null || true)
```

Expected remotes:

```text
origin    <user fork>
upstream  <canonical project repository>
```

For Handy, the canonical upstream is usually:

```bash
git remote add upstream https://github.com/cjpais/Handy.git
```

Only add it if it is missing and appropriate.

### 2. Fetch everything

```bash
git fetch upstream
git fetch origin
```

After fetching, report the new upstream changes that arrived from `cjpais/Handy`. The user specifically wants the number of new commits and all new features added upstream, not remotes, ranges, or commit hashes.

Prefer comparing the old upstream tip captured before fetch to the new one:

```bash
NEW_UPSTREAM_MAIN=$(git rev-parse upstream/main)
if [ -n "$OLD_UPSTREAM_MAIN" ] && [ "$OLD_UPSTREAM_MAIN" != "$NEW_UPSTREAM_MAIN" ]; then
  git rev-list --count "$OLD_UPSTREAM_MAIN..$NEW_UPSTREAM_MAIN"
  git log --format='%s' --reverse "$OLD_UPSTREAM_MAIN..$NEW_UPSTREAM_MAIN"
  git log --format='%s' --reverse --grep='^feat\|feature\|add ' "$OLD_UPSTREAM_MAIN..$NEW_UPSTREAM_MAIN"
fi
```

If this is the first time configuring `upstream`, compare the current local `main` (or the fork branch's merge-base with `upstream/main`) against `upstream/main` instead:

```bash
git rev-list --count main..upstream/main
git log --format='%s' --reverse main..upstream/main
```

Reporting rules:

- Include a "New upstream features" section in the final response.
- Start with the total number of new upstream commits.
- List every upstream commit that appears to add a feature, using commit subjects/PR titles as the source of truth.
- Do not include commit hashes, raw revision ranges, or remote URL details in the user-facing report unless the user asks.
- If a commit subject includes a PR number like `(#123)`, turn it into a Markdown link to `https://github.com/cjpais/Handy/pull/123`.
- If there are no feature-like commits, say so explicitly.
- Do not invent product behavior beyond what commit messages or changed files clearly show; phrase uncertain items as "appears to".
- If conflicts occurred, include a very small summary of their nature (for example: additive settings fields, generated bindings, overlapping UI copy, or behavioral logic overlap), not just the file list.

### 3. Keep `main` aligned with upstream

```bash
git switch main
git merge --ff-only upstream/main
```

If the user wants the fork's `main` updated too:

```bash
git push origin main
```

If `merge --ff-only` fails, `main` has local commits. Stop and explain; do not auto-merge.

### 4. Rebase the personal fork branch

Use `fork` unless the user names another branch.

```bash
git switch fork
git rebase upstream/main
```

If conflicts occur:

```bash
# resolve files
git add <resolved-files>
git rebase --continue
```

Abort if needed:

```bash
git rebase --abort
```

### 5. Verify history

```bash
git status --short --branch
git log --oneline --graph --decorate --boundary upstream/main..HEAD
```

The output should show only the user's commits on top of upstream.

### 6. Push safely

For an existing remote `fork` branch after a successful rebase:

```bash
git push --force-with-lease origin fork
```

For first-time publishing:

```bash
git push -u origin fork
```

## Feature Branch Workflow

Create features from the personal branch:

```bash
git switch fork
git switch -c feature/<name>
```

Before integrating feature work:

```bash
git fetch upstream
git rebase upstream/main
```

Then integrate into `fork` with a fast-forward when possible:

```bash
git switch fork
git rebase upstream/main
git merge --ff-only feature/<name>
```

Push the updated personal branch safely:

```bash
git push --force-with-lease origin fork
```

## Checks After Rebase

Run project-appropriate checks. For Handy, prefer:

```bash
bun run lint
bun run build
bun run format:check
```

If dependencies or native build setup are unavailable, say which checks could not be run and why.

## Nix Consumption Notes

If the user's Nix config consumes the fork branch, point it at the long-lived branch, not `main`, for example:

```text
github:<user>/Handy/fork
```

Prefer lockfiles/pinned revisions. Updating Nix should be explicit, e.g. update the relevant flake input rather than relying on a moving branch implicitly.

## Communication Pattern

When performing a sync, use this concise report format:

```text
Upstream sync report

New upstream commits: <count>

New upstream features
- <feature summary> ([#123](https://github.com/cjpais/Handy/pull/123))
- <feature summary without PR link if no PR number is present>

Other notable changes
- <brief non-feature change if relevant>

Sync result
- main fast-forwarded: yes/no
- fork rebased: yes/no
- conflicts: none / resolved in <short list or count>; nature: <very brief summary if any>
- pushed: main yes/no, fork yes/no

Checks
- <check>: pass/fail/not run
```

Do not include remotes, revision ranges, or commit hashes in the user-facing report unless the user asks. Keep the focus on the number of new commits, the feature list, sync outcome, and checks.
