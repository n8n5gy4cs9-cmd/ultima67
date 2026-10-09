//! Developer console + cheats: a pure command parser. It turns a text line into an
//! [`Action`]; the game applies the action. Everything is registered once in
//! [`Registry::standard`], which also generates `commands.txt` and the cheats menu.
mod actions;
mod console;
mod registry;

pub use actions::*;
pub use console::Console;
pub use registry::{Command, Registry};
