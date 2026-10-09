# HANDOFF - read this first when resuming

Work may stop at any time (token limit ~5h windows). Resume procedure:

1. `git pull`, open `TASKS.md`, find **Next action** at the top (also `PRD.json -> next_action`).
2. Take the first `todo` task of the lowest-numbered unfinished phase unless the next action says otherwise.
3. When finishing a task: set its `status` to `done` in `PRD.json`, update `next_action`, run `python3 tools/sync_tasks.py`, update the "Last session log" below, commit and push.
4. Never mark a task done unless it builds (`cargo build`) and its tests pass.
5. Keep ASSETS.md in sync whenever an asset is added (T3.11 will automate it).

## Rules
- **USER RULE: do not build/test after every task. Write a whole phase of code, then build + test ONCE at the end of the phase.**
- Run the game headless here: `xvfb-run -a ./target/debug/ultima67 --screenshot out.png --frames 120 [--tp place] [--time HH:MM] [--zoom N]` with `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json` (apt: mesa-vulkan-drivers libxkbcommon-x11-0 libasound2-dev libudev-dev). Bevy is pinned =0.16.1.
- Branch: `claude/ecstatic-meitner-cfmv5a`. Commit small and often.
- Do **not** commit original Ultima 7 / Nox game data. Exult/Nuvie/OpenNox code may be ported (project is GPL-3.0-or-later; add attribution header + list in REFERENCES.md). Never commit original game data (see `docs/REFERENCES.md`).
- Keep everything cross-platform; macOS Apple Silicon is the primary target.
- Content is data-driven (RON/JSON/script files) so it can be edited without recompiling.

## Regenerate assets
`cargo run -p u67_assetgen` rewrites `assets/` and `ASSETS.md` (committed). Console docs: `cargo run -q -p u67_console --bin gen_commands > commands.txt`. Tasks: `python3 tools/sync_tasks.py`.

## Last session log
- 2026-10-09: Planning phase (P0) completed. No code yet. Next: P1 reference study, then P2 workspace.
- 2026-10-09 (later): P1 notes done; u67_core/world/console/assetgen implemented + tested; assets generated. Arena mode dropped. Next: u67_mapgen, then Bevy game crate.
