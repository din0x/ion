use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{app::App, theme::Theme};

pub struct Input {
    buf: String,
    position: usize,
    symbol: Option<char>,
    placeholder: Option<String>,
    submit: Box<dyn FnOnce(String, &mut App)>,
}

impl Input {
    pub fn new(submit: impl FnOnce(String, &mut App) + 'static) -> Self {
        Self {
            buf: String::new(),
            position: 0,
            symbol: None,
            placeholder: None,
            submit: Box::new(submit),
        }
    }

    pub fn with_symbol(mut self, symbol: char) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn insert(&mut self, ch: char) {
        self.buf.push(ch);
        self.position += ch.len_utf8();
    }

    pub fn remove(&mut self) {
        if let Some(ch) = self.buf.pop() {
            self.position -= ch.len_utf8();
        }
    }

    pub fn submit(self, app: &mut App) {
        (self.submit)(self.buf, app)
    }

    pub fn render(&self, theme: &Theme, buf: &mut Buffer, mut area: Rect) -> Option<Position> {
        if let Some(symbol) = self.symbol {
            let [symbol_area, other] =
                Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(area);

            Paragraph::new(symbol.encode_utf8(&mut [0u8; 4]).as_str())
                .style(theme.editor)
                .render(symbol_area, buf);
            area = other;
        }

        if self.buf.is_empty() {
            let style = theme.editor.patch(theme.placeholder);

            let Some(placeholder) = self.placeholder.as_ref() else {
                buf.set_style(area, style);
                return Some(area.as_position());
            };

            Paragraph::new(placeholder.as_str())
                .style(style)
                .render(area, buf);

            Some(area.as_position())
        } else {
            const PADDING: u16 = 5;
            let max_width = area.width.saturating_sub(PADDING).into();

            let s = self.buf.lines().last().unwrap_or_default();
            let from_back = s
                .chars()
                .rev()
                .take(max_width)
                .map(char::len_utf8)
                .sum::<usize>();
            let s = &s[(s.len() - from_back)..];

            Paragraph::new(s).style(theme.editor).render(area, buf);
            Some(Position::new(
                area.x + s.chars().count().min(max_width) as u16,
                area.y,
            ))
        }
    }
}
