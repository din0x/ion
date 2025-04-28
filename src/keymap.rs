use std::collections::BTreeMap;

use crate::command::Command;

#[derive(Default)]
pub struct Keymap {
    keys: BTreeMap<char, Command<()>>,
}

impl Keymap {
    pub fn insert(&mut self, key: char, value: Command<()>) {
        self.keys.insert(key, value);
    }

    pub fn get(&self, key: char) -> Option<&Command<()>> {
        self.keys.get(&key)
    }
}
