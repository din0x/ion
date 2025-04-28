use ratatui::style::{Color, Style};

pub struct Theme {
    pub editor: Style,
    pub line_numbers: Style,
    pub active_line: Style,
    pub status_bar: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            editor: Style::new()
                .fg(Color::Rgb(230, 225, 207))
                .bg(Color::Rgb(25, 31, 38)),
            line_numbers: Style::new().fg(Color::Rgb(92, 103, 115)),
            active_line: Style::new()
                .fg(Color::Rgb(210, 205, 187))
                .bg(Color::Rgb(39, 43, 46)),
            status_bar: Style::new()
                .fg(Color::Rgb(197, 197, 197))
                .bg(Color::Rgb(15, 20, 25)),
        }
    }
}
