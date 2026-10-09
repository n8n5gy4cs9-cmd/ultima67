# Cheats & Console

Open the console with **`** (backtick) or **F1**; the cheats menu is **F2**. Syntax: `command arg1 arg2`. Arguments use readable ids (`give vainamoinen_gun`). Tab autocompletes commands and ids. `help` lists everything, `help <cmd>` shows usage. The full list is `commands.txt` (auto-generated from the registry at T7.5).

## Design rules
- Verb-first, lowercase, snake_case ids, English words (`give`, `tp`, `god`).
- Toggles accept `on|off` or none to flip.
- Every command is registered once in `u67_console::registry`; the menu, help and `commands.txt` derive from it.
- Cheats disable achievements/nothing else; they are for fun and dev.

## Planned cheats menu (F2)
God mode, Infinite ammo, No clip, Super speed, All spells, Reveal map, Unlock all fast-travel, Max stats, Give item picker, Teleport picker (towns/planets), Time/weather, Spawn picker.

See `commands.txt` for the exact command list.
