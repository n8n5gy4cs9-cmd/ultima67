//! Console UI state (input line, history, scrollback). Rendering lives in the game crate.
use crate::registry::Registry;
use crate::Action;

pub struct Console {
    pub registry: Registry,
    pub input: String,
    pub history: Vec<String>,
    pub scrollback: Vec<String>,
    hist_pos: Option<usize>,
}

impl Default for Console {
    fn default() -> Self {
        let mut c = Self { registry: Registry::standard(), input: String::new(), history: vec![], scrollback: vec![], hist_pos: None };
        c.print("Ultima67 console. Type 'help'. Tab completes.");
        c
    }
}

impl Console {
    pub fn print(&mut self, s: impl Into<String>) {
        for l in s.into().lines() {
            self.scrollback.push(l.to_string());
        }
        if self.scrollback.len() > 500 {
            let n = self.scrollback.len() - 500;
            self.scrollback.drain(..n);
        }
    }

    /// Submit the input line. Meta commands (help/cmds/clear/echo) are handled here and return None.
    pub fn submit(&mut self) -> Option<Action> {
        let line = std::mem::take(&mut self.input);
        self.hist_pos = None;
        if line.trim().is_empty() {
            return None;
        }
        self.print(format!("> {line}"));
        if self.history.last() != Some(&line) {
            self.history.push(line.clone());
        }
        match self.registry.parse(&line) {
            Err(e) => {
                self.print(e);
                None
            }
            Ok(None) => None,
            Ok(Some(a)) => self.meta(a),
        }
    }

    fn meta(&mut self, a: Action) -> Option<Action> {
        match a {
            Action::Clear => self.scrollback.clear(),
            Action::Echo(t) => self.print(t),
            Action::Cmds => {
                let t = self.registry.commands.iter().map(|c| c.name).collect::<Vec<_>>().join(" ");
                self.print(t);
            }
            Action::Help(None) => {
                let t = self.registry.render_commands_txt();
                self.print(t);
            }
            Action::Help(Some(n)) => match self.registry.find(&n) {
                Some(c) => {
                    let t = format!("{} {}\n  {}", c.name, c.usage, c.help);
                    self.print(t)
                }
                None => self.print(format!("no such command '{n}'")),
            },
            other => return Some(other),
        }
        None
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let i = self.hist_pos.map_or(self.history.len() - 1, |p| p.saturating_sub(1));
        self.hist_pos = Some(i);
        self.input = self.history[i].clone();
    }
    pub fn history_next(&mut self) {
        match self.hist_pos {
            Some(p) if p + 1 < self.history.len() => {
                self.hist_pos = Some(p + 1);
                self.input = self.history[p + 1].clone();
            }
            _ => {
                self.hist_pos = None;
                self.input.clear();
            }
        }
    }
    /// Complete the current word; if unique, apply it.
    pub fn tab(&mut self) {
        let c = self.registry.complete(&self.input);
        if c.len() == 1 {
            let cut = if self.input.ends_with(' ') || self.input.is_empty() { self.input.len() } else { self.input.rfind(' ').map_or(0, |i| i + 1) };
            self.input.truncate(cut);
            self.input.push_str(&c[0]);
            self.input.push(' ');
        } else if c.len() > 1 {
            self.print(c.join("  "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submit_flow() {
        let mut c = Console { input: "give rope 2".into(), ..Console::default() };
        assert!(matches!(c.submit(), Some(Action::Give { .. })));
        c.input = "nonsense".into();
        assert!(c.submit().is_none());
        assert!(c.scrollback.last().unwrap().contains("unknown command"));
        c.history_prev();
        assert_eq!(c.input, "nonsense");
        c.input = "give vainamoinen_g".into();
        c.tab();
        assert_eq!(c.input, "give vainamoinen_gun ");
    }
}
