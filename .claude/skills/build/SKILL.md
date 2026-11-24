---
name: build
description: Build a branch for Windows on myvam-box (diag profile) and stage it as ~/victory-data/victory-<tag>.exe.
argument-hint: "<tag> [branch=current]"
disable-model-invocation: true
---

Arguments: `$ARGUMENTS` (tag, optional branch; default the current branch).
1. Make sure the branch is pushed: `git push -u fork HEAD` if it has unpushed commits (never to `origin`).
2. Delegate to `box-runner`: build that branch with the given tag. Report the exe path, build time and any warnings/errors.
