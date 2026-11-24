---
name: autopilot
description: Autonomous contribution loop for VIctory - pick a non-duplicate task, implement it, test on the Mac and myvam-box, review, push to the fork with Windows CI, and queue it for upstream. Asks one question at the end.
argument-hint: "[area=shader] [count=2]"
disable-model-invocation: true
---

Run the loop below for `$ARGUMENTS` (defaults: area `shader`, count 2). Work autonomously: do not ask the user anything until the final summary. If a step fails, fix it or drop the task and move to the next candidate; record why.

Setup once: `source ~/.cargo/env`; `git fetch origin fork -q`; if fork `main` is more than ~20 commits behind `origin/main`, run the `/sync` steps first (it is allowed without asking here).

For each task:
1. **Pick.** If myvam-box is reachable and a recent run log exists, have `log-analyst` summarize the newest one. Then `scout` with the area and that histogram. Take its top candidate not already in `~/.victory/queue.md`.
2. **Branch.** `git switch -c <area>/<short-slug> fork/main` (fork main = upstream + the setup commit).
3. **Implement.** Delegate to `shader-dev` (or do it yourself for non-shader areas), then bring its commits onto your branch.
4. **Test on the Mac.** Crate tests, `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings`, `cargo fmt --all --check`, `cargo xtask policy`.
5. **Push to the fork** (`git push -u fork HEAD`) and have `box-runner` build it (tag = slug). If the change can affect rendering, have `box-runner` run the game for the new build and compare against the last baseline run (`log-analyst` before/after). Skip the game run for pure refactors.
6. **Review.** `pr-reviewer`. Fix every real finding, re-test, repeat until "ready" (max 3 rounds; otherwise drop the task).
7. **Make the upstream branch.** `git rebase --onto origin/main fork/main` so the setup commit is gone; confirm `git diff --stat origin/main...HEAD` has no `.claude/`/`CLAUDE.md`. Push to `fork` as `up/<slug>` (`--force-with-lease`).
8. **Windows CI.** Open a draft PR inside the fork (`gh pr create --repo ruvenesh/VIctory --base main --head up/<slug> --draft ...`) so the upstream `ci.yml` runs on windows-latest; follow it with `gh run list/view --repo ruvenesh/VIctory`, fix failures, push again. (If Actions are disabled on the fork, note it and rely on the box build.)
9. **Queue.** Append to `~/.victory/queue.md`: `- up/<slug> | <title> | tests | box numbers | CI status | fork PR link`, plus the drafted body in `~/.victory/pr-<slug>.md`.

Finish with one message: a table of queued items (title, what changed, test/CI/box evidence), dropped candidates with reasons, then ask one question: which queued items to open upstream (they then go through `/pr-upstream`).

Never: push to `origin`, open/comment on upstream PRs or issues without the user's yes, commit anything game-derived, or use sudo.
