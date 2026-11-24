#!/bin/bash
# SessionStart: make fresh clones run the pre-commit policy hook, and tell
# Claude which machine it is on.
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
git config core.hooksPath .githooks 2>/dev/null
if [ -n "$CLAUDE_CODE_REMOTE" ]; then
  echo "Context: cloud VM (Linux). No game, no box. Edit, cargo test on portable crates, cargo clippy --target x86_64-pc-windows-msvc (check only)."
  exit 0
fi
if ssh -o ConnectTimeout=3 -o BatchMode=yes myvam-box true 2>/dev/null; then
  echo "Context: Mac control seat. myvam-box reachable: all builds, tests and game runs happen there (the Mac only orchestrates)."
else
  echo "Context: Mac control seat. WARNING: myvam-box is NOT reachable; nothing can be built or tested until it is back (the Mac only orchestrates)."
fi
branch=$(git branch --show-current)
echo "Branch: $branch. Behind origin/main: $(git rev-list --count HEAD..origin/main 2>/dev/null || echo ?)."
[ -f "$HOME/.victory/queue.md" ] && echo "Upstream PR queue: $(grep -c '^- ' "$HOME/.victory/queue.md") item(s) in ~/.victory/queue.md"
exit 0
