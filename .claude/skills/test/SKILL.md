---
name: test
description: Run upstream CI's checks (workspace clippy twice, tests under wine) for a branch on myvam-box.
argument-hint: "<tag> [branch=current] [cargo test args]"
disable-model-invocation: true
---

Arguments: `$ARGUMENTS` (tag, optional branch, optional cargo test args such as `-p victory-shader ds_`; default the current branch and `--workspace`).
Nothing runs on the Mac.
1. Push the branch if it has unpushed commits: `git push -u fork HEAD` (never to `origin`).
2. `ssh myvam-box 'cd ~/victory-data && setsid nohup ~/victory-tools/test-branch.sh <tag> <branch> [args] > test-<tag>.out 2>&1 < /dev/null &'`
3. Poll with short ssh calls until `~/victory-data/test-<tag>.out` has the `clippy= clippy_all= tests=` line (a full workspace run takes several minutes).
4. Report the three exit codes, the test totals (`grep "test result" ~/victory-data/test-<tag>.log`) and every failing test with its panic message. Six tests fail on the box even on upstream main (wine limits); CLAUDE.md lists them. Any other failure is real.
