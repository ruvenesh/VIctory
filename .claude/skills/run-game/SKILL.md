---
name: run-game
description: Run a staged build of the title on myvam-box under Proton and report fps, refused pipelines, errors and screenshots.
argument-hint: "<tag> [baseline-tag]"
disable-model-invocation: true
---

Arguments: `$ARGUMENTS`.
1. Delegate to `box-runner`: run `victory-<tag>.exe` with drive.sh, take screenshots, return the summary.
2. Have `log-analyst` summarize `<tag>`, and compare with the baseline tag if given.
3. Show the user the numbers and the screenshots (send image files with SendUserFile so they appear on the phone).
