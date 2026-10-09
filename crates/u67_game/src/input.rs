//! Keyboard/gamepad input -> per-frame `Intent`. Keys come from `Settings` (rebindable).
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

/// What player 0 wants to do this frame. (Splitscreen adds one per seat.)
#[derive(Resource, Default, Clone, Copy)]
pub struct Intent {
    pub movement: Vec2,
    pub run: bool,
    pub interact: bool,
    pub attack: bool,
}

pub fn read_intent(kb: Res<ButtonInput<KeyCode>>, keys: Res<KeyMap>, settings: Res<crate::app::SettingsRes>, pads: Query<&Gamepad>, mut intent: ResMut<Intent>) {
    let mut v = Vec2::ZERO;
    if keys.pressed(Action::Up, &kb) {
        v.y -= 1.0;
    }
    if keys.pressed(Action::Down, &kb) {
        v.y += 1.0;
    }
    if keys.pressed(Action::Left, &kb) {
        v.x -= 1.0;
    }
    if keys.pressed(Action::Right, &kb) {
        v.x += 1.0;
    }
    let mut run = keys.pressed(Action::Run, &kb);
    let mut interact = keys.just_pressed(Action::Interact, &kb);
    let mut attack = keys.just_pressed(Action::Attack, &kb);
    for pad in &pads {
        let s = pad.left_stick();
        if s.length() > settings.0.gamepad_deadzone {
            v += Vec2::new(s.x, -s.y); // stick up is +y; screen/map y grows downwards
        }
        run |= pad.pressed(GamepadButton::LeftTrigger);
        interact |= pad.just_pressed(GamepadButton::South);
        attack |= pad.just_pressed(GamepadButton::West) || pad.just_pressed(GamepadButton::RightTrigger);
    }
    *intent = Intent { movement: v.clamp_length_max(1.0), run, interact, attack };
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
}
