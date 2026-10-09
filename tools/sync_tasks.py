#!/usr/bin/env python3
"""Regenerate TASKS.md from PRD.json. Edit statuses in PRD.json, then run this."""
import json, pathlib, datetime
root = pathlib.Path(__file__).resolve().parent.parent
prd = json.loads((root / "PRD.json").read_text())
mark = {"done": "x", "todo": " ", "doing": "~", "blocked": "!"}
tot = done = 0
body = []
for ph in prd["phases"]:
    body.append(f"\n## {ph['id']} - {ph['name']}\n")
    for t in ph["tasks"]:
        tot += 1; done += t["status"] == "done"
        opt = " _(optional)_" if t.get("optional") else ""
        body.append(f"- [{mark[t['status']]}] **{t['id']}** {t['title']}{opt}")
head = f"""# TASKS - Ultima67 ({prd['credit']})

> Generated from `PRD.json` by `python3 tools/sync_tasks.py`. **Edit PRD.json, then re-run.**
> Legend: `[x]` done, `[ ]` todo, `[~]` in progress, `[!]` blocked.

**Progress: {done}/{tot} done.**  
**Next action:** {prd['next_action']}
"""
(root / "TASKS.md").write_text(head + "\n".join(body) + "\n")
print(f"TASKS.md written: {done}/{tot}")
