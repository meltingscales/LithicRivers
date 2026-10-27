//! A search box shared by the tabs that filter a list (the Item Dictionary and Crafting): the
//! search key starts typing, the end key keeps the filter, Escape clears it, and matching is
//! a case-insensitive substring test.

use crate::app_state::Keybinds;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchBox {
    pub text: String,
    /// Whether typed keys go into the search text
    pub typing: bool,
}

impl SearchBox {
    /// Handle a key. While typing every printable key is the box's; otherwise only the search
    /// key and, with a filter set, Escape. Returns `None` for keys it leaves to the tab, or
    /// `Some(reset)` where `reset` says the filter changed and the tab's selection should go
    /// back to the top.
    pub fn handle(&mut self, keys: &Keybinds, key: &KeyCode) -> Option<bool> {
        if self.typing {
            if keys.matches("search", "END_SEARCH", key) {
                self.typing = false;
            } else if keys.matches("ui", "CLOSE_HELP_MENU", key) {
                self.typing = false;
                self.text.clear();
            } else if keys.matches("search", "DELETE_CHAR", key) {
                self.text.pop();
            } else if let KeyCode::Char(c) = key {
                self.text.push(*c);
            } else {
                return None;
            }
            return Some(true);
        }
        if keys.matches("search", "SEARCH", key) {
            self.typing = true;
            Some(false)
        } else if keys.matches("ui", "CLOSE_HELP_MENU", key) && !self.text.is_empty() {
            self.text.clear();
            Some(true)
        } else {
            None
        }
    }

    /// Whether any of `haystacks` contains the search text, ignoring case (always true with no
    /// filter).
    pub fn matches<'a>(&self, haystacks: impl IntoIterator<Item = &'a str>) -> bool {
        let needle = self.text.to_lowercase();
        needle.is_empty()
            || haystacks
                .into_iter()
                .any(|h| h.to_lowercase().contains(&needle))
    }

    /// The line under the list: how to start a search, or the search and how to change it.
    pub fn hint<K: std::fmt::Display>(&self, key: impl Fn(&str, &str) -> K) -> String {
        if self.typing {
            format!(
                "Search: {}_   ({} done, {} clear)",
                self.text,
                key("search", "END_SEARCH"),
                key("ui", "CLOSE_HELP_MENU")
            )
        } else if self.text.is_empty() {
            format!("{}: search", key("search", "SEARCH"))
        } else {
            format!(
                "Search: {}   ({} edit, {} clear)",
                self.text,
                key("search", "SEARCH"),
                key("ui", "CLOSE_HELP_MENU")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SearchBox;

    #[test]
    fn matching_ignores_case_and_an_empty_search_matches_everything() {
        let mut search = SearchBox::default();
        assert!(search.matches(["anything"]));
        search.text = "PLank".into();
        assert!(search.matches(["Stick", "Wooden Plank"]));
        assert!(!search.matches(["Stick", "Log"]));
    }
}
