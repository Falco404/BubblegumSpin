# BubblegumSpin

Native port of the PICO-8 game Bubblegum Spin (v1.1, by Bee_Randon). It
should play like the original on Windows and Linux; Android maybe later.
The original cart and notes are in reference/ (read-only, never edit).

## Rules
- Never push to main. Work on branches `hermes/<topic>` and open a PR.
- Run `git pull --ff-only` on main before any new task.
- Small PRs: one milestone or feature each, with tests.
- Every PR description must contain:
  1. Plain-language summary of what changed and why.
  2. How I can try it myself, with exact commands for Windows.
  3. The test summary line and any failures, pasted (not paraphrased).
  4. Anything you're unsure about or decided on your own.
- Never claim something works without showing evidence.
- If a tool, credential or permission is missing, ask me. Never work
  around a restriction (no reading other processes' environments, no
  hunting for secrets outside this repo).
- No secrets in the repo. Config goes in env vars or a gitignored file.

## Docs (keep them small)
- Rewrite and condense, never just append.
- Limits: AGENTS.md <= 80 lines, docs/STATUS.md <= 40, docs/decisions.md <= 30.
  Over a limit? Condense, and move old material to docs/archive/ (don't delete).
- Docs say what the code can't: purpose, constraints, why, how to run, gotchas.
- Record only significant decisions, one or two lines each, with the reason.
- Every PR updates docs/STATUS.md.

## Docs map (read only what the task needs)
- docs/STATUS.md: where we are. Always read.
- docs/decisions.md: read before changing architecture, libraries or tools.
- reference/NOTES.md: how the original plays. Read before changing behaviour.
- docs/architecture.md, docs/RUNNING.md: create once there is something to
  describe or run.
- docs/archive/: history. Never read unless I ask.

## Session start
1. Read AGENTS.md and docs/STATUS.md only.
2. `git fetch --all`, `git pull --ff-only` on main (or pull the working branch).
3. Run `git status`, `git log --oneline -5`, `gh pr list`,
   `gh issue list --label milestone --state open`.
4. Run the tests; report only the summary line and failures.
5. Summarize in under 10 lines (state, next step, questions), then wait for my OK.

## Handoff (when I ask, or when the session gets long)
1. Stop new work. Commit and push everything (WIP is fine); `git status` clean.
2. Open or update the PR (draft if unfinished).
3. Rewrite docs/STATUS.md; update decisions.md only for significant decisions.
4. Update the relevant issues, then post a short summary and the prompt for
   the new session.

## Project-specific rules
- Goal: plays like the original. Behaviour matters more than copying its code.
- The original uses PICO-8 fixed-point math. Flag anywhere float math changes
  behaviour.
- Keep game logic separate from drawing and sound, so it can be tested with
  no screen.
- Tests use a fixed random seed. No tests that depend on luck.
- For visual changes, render frames to PNG headless where possible and
  describe or attach them in the PR.
- The framework must target Windows and Linux now and Android later.
- CI must produce Windows and Linux builds I can download. I add the
  workflow file myself; propose its content in the PR description.
- Test command: <fill in once the approach is chosen>
- Credit Bee_Randon in the README.
