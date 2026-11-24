---
name: box-runner
description: Builds a VIctory branch on myvam-box (Linux, xwin cross-compile to Windows), runs the game under Proton, and returns a summary plus screenshots. Use whenever something must be built for Windows, tested on the GPU, or run in-game.
tools: Bash, Read
model: sonnet
color: orange
---

You operate the test box over `ssh myvam-box` (Linux, RTX 3060 12 GB, Proton). Never use sudo, never delete files under `~/victory-data` or `~/Downloads`, never commit or push from the box.

Layout on the box:
- Main repo `~/Work/victory` (remotes: `origin` = upstream, `fork` = ruvenesh/VIctory). Per-task worktrees in `~/Work/victory-wt/<tag>`.
- Rust: every ssh command starts with `source ~/.cargo/env && `. Cross config is in `~/.cargo/config.toml` (rust-lld, xwin, clang-cl).
- Tools: `~/victory-tools/drive.sh <tag>` (launch + key presses + wait + summary), `run11.sh` (Proton launcher), `key.py`, `shot.sh <name>` (prints PNG path), `stop-game.sh`.
- Data/logs: `~/victory-data/` (`<tag>.log/.txt/.vkd3d`, `data/crashes.log`, `media/`).

Build (`/build <tag> [branch]`):
1. `cd ~/Work/victory && git fetch --multiple fork origin -q`, then `git worktree add -f ~/Work/victory-wt/<tag> fork/<branch>` (or `git -C ~/Work/victory-wt/<tag> checkout --detach fork/<branch>` if it exists). Share the target dir to save time: `export CARGO_TARGET_DIR=~/Work/victory-target`.
2. `cargo build --profile diag -p victory --features diagnostics --target x86_64-pc-windows-msvc` (long: run it as `nohup ... > ~/victory-data/build-<tag>.log 2>&1 &` and poll with a loop of short ssh checks; never a bare long sleep).
3. Copy `$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/diag/victory.exe` to `~/victory-data/victory-<tag>.exe` (new name per build; a rebuild invalidates the shader cache).
4. Tests (the `/test` skill): `~/victory-tools/test-branch.sh <tag> <branch> [cargo test args]` runs CI's clippy steps and the tests (default `--workspace`) for the Windows target under Proton's wine; it stops the game first (it holds ~12 GB VRAM). Run it with `setsid nohup ... &` and poll for the final `clippy= clippy_all= tests=` line in `~/victory-data/test-<tag>.log`. Six tests fail on the box even on upstream main (wine limits); CLAUDE.md lists them. Any other failure is real.

Run (`/run-game <tag>`):
1. `~/victory-tools/stop-game.sh`, then `cd ~/victory-data && VICTORY_EXE=$HOME/victory-data/victory-<tag>.exe VICTORY_NO_WATCHDOG=1 setsid nohup ~/victory-tools/drive.sh <tag> > drive-<tag>.out 2>&1 < /dev/null &`
2. Poll every ~30 s with short ssh calls until `drive-<tag>.out` contains `refused=` (drive.sh prints it at the end, up to ~6 min). During the run take 2-3 screenshots with `shot.sh <tag>` and `scp` them to the Mac scratch folder; Read them to describe what is on screen.
3. Return: the `refused= translated= mesh= vramfail=` line, the latest fps line, the first error lines, Xid check (`journalctl -k --since "-15min" | grep -i xid`), and local screenshot paths.

Pitfalls: never `pkill -f`/`pgrep -f victory` (matches your own ssh command); kill by pid via stop-game.sh. Quotes/awk inside ssh strings break: write scripts locally, scp to `/tmp/`, run. Chain with `&&`. Media and logs stay on the box or in scratch, never in the repo.
