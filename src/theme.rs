use ratatui::style::{Color, Style, Stylize};
use std::collections::HashMap;

pub struct Theme {
    pub editor: Style,
    pub selection: Style,
    pub line_numbers: Style,
    pub active_line: Style,
    pub status_bar: Style,
    pub placeholder: Style,
    pub tokens: HashMap<String, Style>,
}

impl Theme {
    pub fn get_token_style(&self, token: &str) -> Style {
        self.tokens
            .get(token)
            .copied()
            .unwrap_or(self.editor.patch(self.editor))
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            editor: Style::new()
                .fg(Color::Rgb(230, 225, 207))
                .bg(Color::Rgb(25, 31, 38)),
            selection: Style::new().on_light_blue(),
            line_numbers: Style::new().fg(Color::Rgb(92, 103, 115)),
            active_line: Style::new()
                .fg(Color::Rgb(210, 205, 187))
                .bg(Color::Rgb(39, 43, 46)),
            status_bar: Style::new()
                .fg(Color::Rgb(197, 197, 197))
                .bg(Color::Rgb(15, 20, 25)),
            placeholder: Style::new().fg(Color::Rgb(170, 165, 147)),
            tokens: HashMap::from_iter([
                // Basic tokens
                (
                    "attribute".into(),
                    Style::new().fg(Color::from_u32(0x00ffb454)), // Ayu orange
                ),
                (
                    "keyword".into(),
                    Style::new().fg(Color::from_u32(0x00ff8f40)), // Ayu orange/red
                ),
                (
                    "constructor".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "function".into(),
                    Style::new().fg(Color::from_u32(0x00ffb454)), // Ayu orange
                ),
                (
                    "function.method".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "punctuation.bracket".into(),
                    Style::new().fg(Color::from_u32(0x00bfbdb6)), // Ayu light gray
                ),
                (
                    "punctuation.delimiter".into(),
                    Style::new().fg(Color::from_u32(0x00bfbdb6)), // Ayu light gray
                ),
                (
                    "type".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "property".into(),
                    Style::new().fg(Color::from_u32(0x00aad94c)), // Ayu green
                ),
                // Tree-sitter specific tokens
                (
                    "variable".into(),
                    Style::new().fg(Color::from_u32(0x00e6e1cf)), // Ayu default text
                ),
                (
                    "variable.parameter".into(),
                    Style::new().fg(Color::from_u32(0x00e6e1cf)), // Ayu default text
                ),
                (
                    "variable.builtin".into(),
                    Style::new().fg(Color::from_u32(0x00ff8f40)), // Ayu orange/red
                ),
                (
                    "constant".into(),
                    Style::new().fg(Color::from_u32(0x00d4bfff)), // Ayu purple
                ),
                (
                    "constant.builtin".into(),
                    Style::new().fg(Color::from_u32(0x00d4bfff)), // Ayu purple
                ),
                (
                    "constant.numeric".into(),
                    Style::new().fg(Color::from_u32(0x00d4bfff)), // Ayu purple
                ),
                (
                    "constant.character".into(),
                    Style::new().fg(Color::from_u32(0x00c2d94c)), // Ayu lime green
                ),
                (
                    "string".into(),
                    Style::new().fg(Color::from_u32(0x00c2d94c)), // Ayu lime green
                ),
                (
                    "string.escape".into(),
                    Style::new().fg(Color::from_u32(0x0095e6cb)), // Ayu cyan
                ),
                (
                    "string.regex".into(),
                    Style::new().fg(Color::from_u32(0x0095e6cb)), // Ayu cyan
                ),
                (
                    "comment".into(),
                    Style::new().fg(Color::from_u32(0x005c6773)), // Ayu dark gray
                ),
                (
                    "comment.documentation".into(),
                    Style::new().fg(Color::from_u32(0x005c6773)), // Ayu dark gray
                ),
                (
                    "tag".into(),
                    Style::new().fg(Color::from_u32(0x00ff8f40)), // Ayu orange/red
                ),
                (
                    "tag.attribute".into(),
                    Style::new().fg(Color::from_u32(0x00ffb454)), // Ayu orange
                ),
                (
                    "tag.delimiter".into(),
                    Style::new().fg(Color::from_u32(0x00bfbdb6)), // Ayu light gray
                ),
                (
                    "operator".into(),
                    Style::new().fg(Color::from_u32(0x00ff8f40)), // Ayu orange/red
                ),
                (
                    "label".into(),
                    Style::new().fg(Color::from_u32(0x00ffb454)), // Ayu orange
                ),
                (
                    "module".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "namespace".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "field".into(),
                    Style::new().fg(Color::from_u32(0x00aad94c)), // Ayu green
                ),
                (
                    "parameter".into(),
                    Style::new().fg(Color::from_u32(0x00e6e1cf)), // Ayu default text
                ),
                (
                    "macro".into(),
                    Style::new().fg(Color::from_u32(0x00ff8f40)), // Ayu orange/red
                ),
                (
                    "type.builtin".into(),
                    Style::new().fg(Color::from_u32(0x0059c2ff)), // Ayu blue
                ),
                (
                    "escape".into(),
                    Style::new().fg(Color::from_u32(0x0095e6cb)), // Ayu cyan
                ),
                (
                    "embedded".into(),
                    Style::new().fg(Color::from_u32(0x00e6e1cf)), // Ayu default text
                ),
                (
                    "error".into(),
                    Style::new().fg(Color::from_u32(0x00f07178)), // Ayu red
                ),
                (
                    "warning".into(),
                    Style::new().fg(Color::from_u32(0x00ffb454)), // Ayu orange
                ),
            ]),
        }
    }
}
