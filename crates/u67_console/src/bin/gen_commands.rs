//! `cargo run -p u67_console --bin gen_commands > commands.txt`
fn main() {
    print!("{}", u67_console::Registry::standard().render_commands_txt());
}
