---
name: stack
description: Build a test branch on myvam-box that merges a list of open upstream PRs on top of the previous stack.
argument-hint: "<PR#> [PR#...]"
disable-model-invocation: true
---

On myvam-box in `~/Work/victory` (via `box-runner` or direct ssh with `source ~/.cargo/env &&`):
1. Find the newest `latest-stackN` branch; create `latest-stack<N+1>` from it. If the main repo has uncommitted changes, stop and report.
2. For each PR number in `$ARGUMENTS`: `git fetch origin pull/<n>/head:pr<n> -q && git merge --no-edit pr<n>`. On conflict: `git merge --abort`, record it, continue.
   Known conflicts: #1088 (valu_tests.rs), #282 vs #334, #413 vs #206.
3. Report merged / skipped PRs, then offer `/build stack<N+1>`.
Stacks are local test branches only: never push them.
