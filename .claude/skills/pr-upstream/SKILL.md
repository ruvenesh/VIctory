---
name: pr-upstream
description: Open queued VIctory branches as pull requests on Marvisak/VIctory after the user approves the exact title and body.
argument-hint: "[up/<slug> ...]  (default: whole queue)"
disable-model-invocation: true
---

1. Read `~/.victory/queue.md` and pick `$ARGUMENTS` (or all items).
2. For each: re-check freshness - `git fetch origin -q`, rebase `up/<slug>` onto `origin/main` if behind, `cargo xtask policy`, confirm the diff has no `.claude/`/`CLAUDE.md`, re-check duplicates with `gh pr list -R Marvisak/VIctory --state open --search ...`, push to fork with `--force-with-lease`.
3. Show the user every title + body (from `~/.victory/pr-<slug>.md`) in one message and ask once to confirm. This is the only approval; an earlier approval does not carry over.
4. On yes: `gh pr create --repo Marvisak/VIctory --base main --head ruvenesh:up/<slug> --title ... --body-file ~/.victory/pr-<slug>.md`. Close the matching draft PR in the fork. Move the item to a "## Opened" section with the PR URL.
