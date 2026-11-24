---
name: sync
description: Bring fork main up to date - upstream main plus the single "dev: Claude Code setup" commit.
disable-model-invocation: true
---

1. `git fetch origin fork -q`. Find the setup commit: `git log --format=%H -1 --grep '^dev: Claude Code setup' fork/main`.
2. In a temporary worktree: `git worktree add -f /tmp/victory-sync origin/main`, `git -C /tmp/victory-sync cherry-pick <setup>`; resolve trivial conflicts in CLAUDE.md/.claude only.
3. `cargo xtask policy` there, then `git -C /tmp/victory-sync push --force-with-lease=main:<old fork/main sha> fork HEAD:main`; remove the worktree.
4. Update the local `dev-setup` branch to match (`git switch dev-setup && git reset --keep fork/main`) if it has no extra commits.
5. On myvam-box: `cd ~/Work/victory && git fetch fork -q`.
Report how many upstream commits came in.
