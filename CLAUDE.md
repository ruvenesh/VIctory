# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

VIctory runs a Grand Theft Auto VI (Xbox Series) title natively: the title's x86-64 code runs unchanged in-process, and VIctory supplies the platform beneath it (loader, console API HLE, D3D12.X → host D3D12, shader recompilation to DXIL). Status: boots and reaches the world, but world rendering is incomplete and can hang real GPUs. Target is Windows 10/11 x64 (AVX2, D3D12 SM 6.6); Linux means Wine/Proton (see git history for `wine`/`proton` fixes). See `docs/architecture.md` for the data-flow diagram and import-resolution order (our `victory-hle::modules` → allowed host DLL → title-shipped DLL → generated stub).

## Commands

Toolchain is pinned in `rust-toolchain.toml` (target `x86_64-pc-windows-msvc`).

In this setup the build/test/run commands below run on myvam-box, not on the Mac (see Contributor setup).

```
cargo xtask install-hooks                  # pre-commit asset-policy check
cargo xtask policy [--staged]              # asset-policy check (allowlist)
cargo test --workspace                     # needs no game data
cargo test -p <crate> <test_name>          # single test
cargo clippy --workspace
cargo build --release -p victory
cargo build --profile diag -p victory --features diagnostics   # debug env vars (VICTORY_*) only exist here
cp victory.example.toml victory.toml       # set paths OUTSIDE the repo; env vars override
cargo run -p victory -- config             # verify paths
target/release/victory run <game dir>/<exe> --execute
```

Other subcommands: `inspect`, `census`, `lib-exports`, `shader-info`, `translate`. Logging via `VICTORY_LOG=victory_loader=trace,info`; the full list of `VICTORY_*` debug variables is in `README.md`. `crashes.log` in the data folder is the first place to look after a failure. `VICTORY_D3D12_WARP=1` renders on the software adapter and avoids resetting the GPU driver.

Tests needing real game files read paths from env vars and skip when unset; they assert properties, never snapshots.

## Rule #1: asset policy

Nothing derived from the game may enter the repo: no assets, executable bytes/strings, disassembly, shader dumps, screenshots, or snapshots. Allowed: own code, BLAKE3 build identities, per-build hook offsets, byte signatures, synthetic test data. The only exception is `crates/victory-formats/src/bcpack-tables.bin`. `cargo xtask policy` enforces an allowlist (type, <512 KiB, no binary/long hex runs); exceptions go in `xtask/src/policy.rs`. Runtime writers must call `victory_core::ensure_outside_repo`; `.victory-root` marks the tree root for that guard. Config, cache and data dirs must be outside the repo.

## Architecture (crate roles)

`victory-core` (config, policy guard) · `victory-pe` (PE/import-lib analysis) · `victory-loader` (map/bind/start the title, TLS redirect, crash reporter) · `victory-hle` (console API implementations, host pass-through policy, virtual drive path rewriting in `modules::files`) · `victory-gpu` (D3D12.X objects → host D3D12) · `victory-shader` (RDNA bytecode → SSA IR → EXEC removal → DXIL, translated lazily at pipeline creation and cached) · `victory-formats` (texture tiling, XMA/Opus, BCPack) · `victory-pipeline` (content-addressed cache) · `victory-game` (GTA VI-specific formats/hooks; everything else is platform-level and testable with any Xbox Series title) · `apps/victory` (CLI) · `tests/fixtures/hle-probe` (standalone stand-in title, excluded from the workspace and built on demand by tests).

Workspace lints deny `unsafe_op_in_unsafe_fn` and `unused_must_use`; clippy warns on undocumented `unsafe` blocks.

## Contributor setup (this fork only, never upstream)

This file and `.claude/` live in `dev:` commits (starting with `dev: Claude Code setup`) on top of upstream `main` in the fork `ruvenesh/VIctory`. Feature branches start from `fork/main`; before anything goes upstream it is rebased with `git rebase --onto origin/main fork/main` so the setup commit is dropped. `/sync` refreshes fork `main`.

### Machines

- **Mac** (this checkout): orchestrator only. Here: edits, git/gh, pushing to `fork`, driving agents and the box over ssh. Do NOT build, test, clippy or run anything here: every `cargo build/test/clippy` and every game run happens on myvam-box. The one exception is the commit hook's `cargo xtask policy --staged` (asset check on what is being committed). Code reaches the box through git: push the branch to `fork`, the box fetches it. Upstream is not rustfmt-clean: never run `cargo fmt --all`.
- **myvam-box** (`ssh myvam-box`, Linux Mint, RTX 3060 12 GB, Proton): builds Windows binaries with xwin + rust-lld and runs the title under GE-Proton 11. Use the `box-runner` agent. `~/victory-tools/test-branch.sh <tag> <fork-branch> [cargo test args]` runs CI's three steps (workspace clippy, clippy `--all-features`, tests) for the Windows target with tests under Proton's wine, log in `~/victory-data/test-<tag>.log`. Known box-only failures (they fail on unchanged upstream main under wine too, and pass on Windows CI): `victory-gpu` indirect::tests::argument_clamp_builds and pipeline::library::tests::a_rejected_blob_gives_an_empty_library, `victory-hle` modules::net::tests::only_loopback_is_reachable (wine's ws2_32 lacks WSAConnectByList), `victory-loader` load_probe::loads_binds_and_calls_probe and `xtask` policy::tests::repository_passes_policy (cannot spawn cargo/git from inside wine), `victory-shader` emit::tests::stub_pipeline_is_created. Any other failure is real; when unsure, run `test-branch.sh base upstream-main` and compare. Other tools: tools in `~/victory-tools/`, data and logs in `~/victory-data/`, game files outside every repo. Box pitfalls: `source ~/.cargo/env &&` first; no `pkill -f`/`pgrep -f` (matches the ssh command), kill by pid; script anything with quotes and `scp` it; stop the title before GPU tests (it holds ~12 GB VRAM); `Xid 109` in the kernel log = GPU hang. No sudo.
- **Windows CI**: a draft PR inside the fork runs upstream's `ci.yml` (windows-latest: clippy `-D warnings` twice, `cargo test --workspace --locked`).

### How to work

- Act autonomously: implement, test, push to `fork`, open draft PRs inside the fork, run the box. Do not stop to ask about routine steps.
- The only things that need the user's explicit yes, each time: opening a PR, issue or comment on `Marvisak/VIctory`, and anything destructive outside the repo. Never push to `origin`.
- Before starting work, check it is not already done or in flight upstream (`scout`). The upstream repo gets many PRs a day.
- One logical change per PR, synthetic tests included. Commit/PR titles: `Area: one sentence` (`Shaders:`, `GPU:`, `HLE:`, `Loader:`, `Formats:`).
- Main loop: `/autopilot [area] [n]`. Pieces: `/pick-task`, `/build <tag>`, `/test <tag>`, `/run-game <tag>`, `/stack <PR#...>`, `/sync`, `/pr-upstream`. Queue of branches ready for upstream: `~/.victory/queue.md`.
- Agents: `scout` (find + dedupe, haiku), `log-analyst` (box logs -> counts, haiku), `shader-dev` (one opcode/lowering in a worktree), `box-runner` (build/run on the box), `pr-reviewer` (pre-PR review). Keep to 1-2 parallel workers (Pro plan limits).
- Send screenshots and result files to the user with SendUserFile: they read on the phone via Remote Control.
