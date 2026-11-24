---
name: pick-task
description: Suggest the next VIctory task, ranked and checked against open upstream PRs.
argument-hint: "[area=shader]"
disable-model-invocation: true
---

1. If `ssh myvam-box true` works, have `log-analyst` summarize the newest run log in `~/victory-data/` (largest-numbered or newest `*.txt`).
2. Have `scout` find candidates for area `$ARGUMENTS` (default `shader`), using that histogram.
3. Show the top 3 with size, impact and duplicate-check links, and recommend one. Do not start implementing unless the user picks.
