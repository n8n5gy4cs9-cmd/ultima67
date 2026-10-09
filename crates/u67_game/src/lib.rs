//! Ultima67 game library: pure logic (data, console actions, saves, settings, rules) + Bevy app.
// Bevy systems take many resources/queries by design; keep clippy focused on real problems.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
pub mod app;
pub mod apply;
pub mod audio;
#[cfg(test)]
mod balance;
pub mod combat;
pub mod creatures;
pub mod data;
pub mod db_res;
pub mod debug;
pub mod gui;
pub mod hazards;
pub mod input;
pub mod interact;
pub mod invops;
pub mod magic;
pub mod menu;
pub mod menus;
pub mod npc;
pub mod persist;
pub mod player;
pub mod render;
pub mod save;
pub mod script;
pub mod seats;
pub mod settings;
pub mod ui;
pub mod weather;
