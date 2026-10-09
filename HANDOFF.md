# HANDOFF - read this first when resuming

Work may stop at any time (token limit ~5h windows). Resume procedure:

1. `git pull`, open `TASKS.md`, find **Next action** at the top (also `PRD.json -> next_action`).
2. Take the first `todo` task of the lowest-numbered unfinished phase unless the next action says otherwise.
3. When finishing a task: set its `status` to `done` in `PRD.json`, update `next_action`, run `python3 tools/sync_tasks.py`, update the "Last session log" below, commit and push.
4. Never mark a task done unless it builds (`cargo build`) and its tests pass.
5. Keep ASSETS.md in sync whenever an asset is added (T3.11 will automate it).

## Rules
- Branch: `claude/ecstatic-meitner-cfmv5a`. Commit small and often.
- Do **not** commit original Ultima 7 / Nox game data. Do not copy GPL code verbatim without keeping the GPL licence (see `docs/REFERENCES.md`).
- Keep everything cross-platform; macOS Apple Silicon is the primary target.
- Content is data-driven (RON/JSON/script files) so it can be edited without recompiling.

## Last session log
- 2026-10-09: Planning phase (P0) completed. No code yet. Next: P1 reference study, then P2 workspace.
