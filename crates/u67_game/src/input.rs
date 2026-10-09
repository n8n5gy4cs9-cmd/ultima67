//! Keyboard/gamepad input -> per-seat `Intent`. Seat 0 keys come from `Settings` (rebindable);
//! seat 1 can use the second keyboard block (arrows etc.); other seats use gamepads.
use crate::seats::{assign_devices, Device, Intent, Intents, Seats};
use crate::settings::{Action, Settings};
use bevy::prelude::*;
use std::collections::BTreeMap;

pub fn key_from_name(n: &str) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match n {
        "A" => KeyA, "B" => KeyB, "C" => KeyC, "D" => KeyD, "E" => KeyE, "F" => KeyF, "G" => KeyG, "H" => KeyH,
        "I" => KeyI, "J" => KeyJ, "K" => KeyK, "L" => KeyL, "M" => KeyM, "N" => KeyN, "O" => KeyO, "P" => KeyP,
        "Q" => KeyQ, "R" => KeyR, "S" => KeyS, "T" => KeyT, "U" => KeyU, "V" => KeyV, "W" => KeyW, "X" => KeyX,
        "Y" => KeyY, "Z" => KeyZ,
        "0" => Digit0, "1" => Digit1, "2" => Digit2, "3" => Digit3, "4" => Digit4,
        "5" => Digit5, "6" => Digit6, "7" => Digit7, "8" => Digit8, "9" => Digit9,
        "ArrowUp" => ArrowUp, "ArrowDown" => ArrowDown, "ArrowLeft" => ArrowLeft, "ArrowRight" => ArrowRight,
        "Space" => Space, "Enter" => Enter, "Escape" => Escape, "Tab" => Tab, "Backquote" => Backquote,
        "ShiftLeft" => ShiftLeft, "ShiftRight" => ShiftRight, "ControlLeft" => ControlLeft, "AltLeft" => AltLeft,
        "Minus" => Minus, "Equal" => Equal, "NumpadAdd" => NumpadAdd, "NumpadSubtract" => NumpadSubtract,
        "F1" => F1, "F2" => F2, "F3" => F3, "F4" => F4, "F5" => F5, "F6" => F6, "F7" => F7, "F8" => F8,
        "F9" => F9, "F10" => F10, "F11" => F11, "F12" => F12,
        _ => return None,
    })
}

#[derive(Resource, Default)]
pub struct KeyMap(pub BTreeMap<Action, Vec<KeyCode>>);

impl KeyMap {
    pub fn from_settings(s: &Settings) -> Self {
        let mut m = BTreeMap::new();
        for (a, names) in &s.keys {
            let codes: Vec<KeyCode> = names
                .iter()
                .filter_map(|n| {
                    let k = key_from_name(n);
                    if k.is_none() {
                        warn!("unknown key name '{n}' for {a:?}");
                    }
                    k
                })
                .collect();
            m.insert(*a, codes);
        }
        Self(m)
    }
    pub fn pressed(&self, a: Action, kb: &ButtonInput<KeyCode>) -> bool {
        self.0.get(&a).is_some_and(|v| v.iter().any(|k| kb.pressed(*k)))
    }
    pub fn just_pressed(&self, a: Action, kb: &ButtonInput<KeyCode>) -> bool {
        self.0.get(&a).is_some_and(|v| v.iter().any(|k| kb.just_pressed(*k)))
    }
}

fn keyboard1(kb: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>, keys: &KeyMap) -> Intent {
    let mut v = Vec2::ZERO;
    if keys.pressed(Action::Up, kb) {
        v.y -= 1.0;
    }
    if keys.pressed(Action::Down, kb) {
        v.y += 1.0;
    }
    if keys.pressed(Action::Left, kb) {
        v.x -= 1.0;
    }
    if keys.pressed(Action::Right, kb) {
        v.x += 1.0;
    }
    Intent {
        movement: v.clamp_length_max(1.0),
        run: keys.pressed(Action::Run, kb),
        interact: keys.just_pressed(Action::Interact, kb) || mouse.just_pressed(MouseButton::Right),
        attack: keys.just_pressed(Action::Attack, kb) || mouse.just_pressed(MouseButton::Left),
        attack_held: keys.pressed(Action::Attack, kb) || mouse.pressed(MouseButton::Left),
        roll: keys.just_pressed(Action::Roll, kb),
        reload: keys.just_pressed(Action::Reload, kb),
        inventory: keys.just_pressed(Action::Inventory, kb),
        pause: keys.just_pressed(Action::Pause, kb),
        aim: Vec2::ZERO,
        spells: [kb.just_pressed(KeyCode::Digit1), kb.just_pressed(KeyCode::Digit2), kb.just_pressed(KeyCode::Digit3)],
    }
}

fn keyboard2(kb: &ButtonInput<KeyCode>) -> Intent {
    let mut v = Vec2::ZERO;
    if kb.pressed(KeyCode::ArrowUp) {
        v.y -= 1.0;
    }
    if kb.pressed(KeyCode::ArrowDown) {
        v.y += 1.0;
    }
    if kb.pressed(KeyCode::ArrowLeft) {
        v.x -= 1.0;
    }
    if kb.pressed(KeyCode::ArrowRight) {
        v.x += 1.0;
    }
    Intent {
        movement: v.clamp_length_max(1.0),
        run: kb.pressed(KeyCode::ShiftRight),
        interact: kb.just_pressed(KeyCode::Slash),
        attack: kb.just_pressed(KeyCode::Period),
        attack_held: kb.pressed(KeyCode::Period),
        roll: kb.just_pressed(KeyCode::Comma),
        reload: kb.just_pressed(KeyCode::Semicolon),
        inventory: kb.just_pressed(KeyCode::Quote),
        pause: false,
        aim: Vec2::ZERO,
        spells: [kb.just_pressed(KeyCode::Digit7), kb.just_pressed(KeyCode::Digit8), kb.just_pressed(KeyCode::Digit9)],
    }
}

fn gamepad(pad: &Gamepad, dead: f32) -> Intent {
    let s = pad.left_stick();
    let m = if s.length() > dead { Vec2::new(s.x, -s.y) } else { Vec2::ZERO };
    let r = pad.right_stick();
    let aim = if r.length() > 0.35 { Vec2::new(r.x, -r.y).normalize_or_zero() } else { Vec2::ZERO };
    Intent {
        movement: m.clamp_length_max(1.0),
        run: pad.pressed(GamepadButton::LeftTrigger),
        interact: pad.just_pressed(GamepadButton::South),
        attack: pad.just_pressed(GamepadButton::RightTrigger) || pad.just_pressed(GamepadButton::RightTrigger2),
        attack_held: pad.pressed(GamepadButton::RightTrigger) || pad.pressed(GamepadButton::RightTrigger2),
        roll: pad.just_pressed(GamepadButton::East),
        reload: pad.just_pressed(GamepadButton::West),
        inventory: pad.just_pressed(GamepadButton::North),
        pause: pad.just_pressed(GamepadButton::Start),
        aim,
        spells: [pad.just_pressed(GamepadButton::DPadLeft), pad.just_pressed(GamepadButton::DPadUp), pad.just_pressed(GamepadButton::DPadRight)],
    }
}

fn merge(a: Intent, b: Intent) -> Intent {
    Intent {
        movement: (a.movement + b.movement).clamp_length_max(1.0),
        run: a.run || b.run,
        interact: a.interact || b.interact,
        attack: a.attack || b.attack,
        attack_held: a.attack_held || b.attack_held,
        roll: a.roll || b.roll,
        reload: a.reload || b.reload,
        inventory: a.inventory || b.inventory,
        pause: a.pause || b.pause,
        aim: if b.aim != Vec2::ZERO { b.aim } else { a.aim },
        spells: [a.spells[0] || b.spells[0], a.spells[1] || b.spells[1], a.spells[2] || b.spells[2]],
    }
}

#[allow(clippy::too_many_arguments)]
pub fn read_intent(
    kb: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<KeyMap>,
    settings: Res<crate::app::SettingsRes>,
    pads: Query<(Entity, &Gamepad)>,
    game: Option<Res<crate::app::Game>>,
    mut seats: ResMut<Seats>,
    mut intents: ResMut<Intents>,
) {
    let n = game.map_or(1, |g| g.0.players.len()).clamp(1, crate::seats::MAX_SEATS);
    let mut pad_ids: Vec<Entity> = pads.iter().map(|(e, _)| e).collect();
    pad_ids.sort();
    if seats.devices.len() != n || seats.devices.iter().skip(1).any(|d| matches!(d, Device::Gamepad(e) if !pad_ids.contains(e))) {
        seats.devices = assign_devices(n, &pad_ids);
    }
    let dead = settings.0.gamepad_deadzone;
    intents.list.clear();
    for (i, d) in seats.devices.iter().enumerate() {
        let mut it = match d {
            Device::Keyboard1 => keyboard1(&kb, &mouse, &keys),
            Device::Keyboard2 => keyboard2(&kb),
            Device::Gamepad(e) => pads.get(*e).map(|(_, p)| gamepad(p, dead)).unwrap_or_default(),
        };
        // single player: a connected gamepad also controls seat 0
        if i == 0 && n == 1 {
            for (_, p) in &pads {
                it = merge(it, gamepad(p, dead));
            }
        }
        intents.list.push(it);
    }
    intents.active = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_default_key_name_resolves() {
        let s = Settings::default();
        for (a, names) in &s.keys {
            for n in names {
                assert!(key_from_name(n).is_some(), "{a:?}: {n}");
            }
        }
    }
    #[test]
    fn merge_combines_inputs() {
        let a = Intent { movement: Vec2::X, attack: true, ..Default::default() };
        let b = Intent { movement: Vec2::Y, run: true, aim: Vec2::X, ..Default::default() };
        let m = merge(a, b);
        assert!(m.attack && m.run && m.movement.length() <= 1.0 + 1e-5 && m.aim == Vec2::X);
    }
}
