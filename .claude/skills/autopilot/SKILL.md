---
name: autopilot
description: Autonomous contribution loop for VIctory - pick a non-duplicate task, implement it, test on myvam-box, review, push to the fork with Windows CI, and queue it for upstream. Asks one question at the end.
argument-hint: "[area=shader] [count=2]"
disable-model-invocation: true
---

Run the loop below for `$ARGUMENTS` (defaults: area `shader`, count 2). Work autonomously: do not ask the user anything until the final summary. If a step fails, fix it or drop the task and move to the next candidate; record why.

The Mac only orchestrates: never run cargo build/test/clippy or the game on it; all of that goes to myvam-box.

Setup once: `git fetch --multiple origin fork -q`; if fork `main` is more than ~20 commits behind `origin/main`, run the `/sync` steps first (it is allowed without asking here).

For each task:
1. **Pick.** Prefer the whole-archive refusal histogram: build an exe of upstream main (`build-exe.sh`) and run `~/victory-tools/translate-all.sh <tag>` on the box, or reuse a recent `~/victory-data/refusals-*.txt`. Most refusals already have several open upstream PRs: dedupe by checking that no PR *diff* removes the refusal string (`gh pr diff <n> | grep`), not just titles. A recent game log (via `log-analyst`) adds in-world weight. Then `scout` with the area and that histogram. Take its top candidate not already in `~/.victory/queue.md`.
2. **Branch.** `git switch -c <area>/<short-slug> fork/main` (fork main = upstream + the setup commit).
3. **Implement.** Delegate to `shader-dev` (or do it yourself for non-shader areas), then bring its commits onto your branch.
4. **Test on the box.** `git push -u fork HEAD`, then `ssh myvam-box '~/victory-tools/test-branch.sh <slug> <branch>'` in the background: CI's workspace clippy (with and without `--all-features`) and the full workspace tests for the Windows target under wine. Wait for the `clippy= clippy_all= tests=` line; read failures from `~/victory-data/test-<slug>.log`. Six tests fail on the box even on upstream main (wine limits); CLAUDE.md lists them. Any other failure is real. (No `cargo fmt --all`: upstream is not fmt-clean.)
5. **Build and run on the box.** Have `box-runner` build the game exe (tag = slug). If the change can affect rendering, have `box-runner` run the game for the new build and compare against the last baseline run (`log-analyst` before/after). Skip the game run for pure refactors. Plain upstream `main` may stop at the title screen; to measure world impact, merge the branch onto the newest `latest-stackN` on the box (`/stack`) and compare against that stack's last run.
6. **Review.** `pr-reviewer`. Fix every real finding, re-test, repeat until "ready" (max 3 rounds; otherwise drop the task).
7. **Make the upstream branch.** `git switch -c up/<slug> origin/main` and `git cherry-pick -n <the task's commits>`, then one commit with the final upstream message (squashed, so fixup commits never need a force-push); confirm `git diff --stat origin/main...HEAD` has no `.claude/`/`CLAUDE.md`. Push to `fork` as `up/<slug>`, and refresh the base branch: `git push fork origin/main:refs/heads/upstream-main`.
8. **Windows CI.** Open a draft PR inside the fork (`gh pr create --repo ruvenesh/VIctory --base upstream-main --head up/<slug> --draft --body-file ~/.victory/pr-<slug>.md ...`; base `upstream-main` keeps the diff clean) so the upstream `ci.yml` runs on windows-latest; wait with `gh run watch <id> --repo ruvenesh/VIctory --exit-status` in the background (Windows CI takes 15-40 min), fix failures, push again. (If Actions are disabled on the fork, note it and rely on the box build.)
9. **Queue.** Append to `~/.victory/queue.md`: `- up/<slug> | <title> | tests | box numbers | CI status | fork PR link`, plus the drafted body in `~/.victory/pr-<slug>.md`.

Finish with one message: a table of queued items (title, what changed, test/CI/box evidence), dropped candidates with reasons, then ask one question: which queued items to open upstream (they then go through `/pr-upstream`).

Never: push to `origin`, open/comment on upstream PRs or issues without the user's yes, commit anything game-derived, or use sudo.
