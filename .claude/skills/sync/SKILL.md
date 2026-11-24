---
name: sync
description: Bring fork main up to date - upstream main plus the single "dev: Claude Code setup" commit.
disable-model-invocation: true
---

1. `git fetch --multiple origin fork -q`. The setup commits are `git rev-list --reverse origin/main..fork/main` (all titled `dev: ...`; stop and report if any is not).
2. In a temporary worktree: `git worktree add -f /tmp/victory-sync origin/main`, `git -C /tmp/victory-sync cherry-pick <those commits>`; resolve trivial conflicts in CLAUDE.md/.claude only.
3. `cargo xtask policy` there, then push `HEAD:main` to `fork` with `--force-with-lease=main:<old fork/main sha>` (this replaces fork main; if the push is refused by the permission classifier, show the user the exact command to run); remove the worktree.
4. Update the local `dev-setup` branch to match (`git switch dev-setup && git reset --keep fork/main`) if it has no extra commits.
5. On myvam-box: `cd ~/Work/victory && git fetch fork -q`.
Report how many upstream commits came in.
