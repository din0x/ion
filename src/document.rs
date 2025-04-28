use std::ops::{Add, RangeInclusive};

use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Style, Stylize},
    widgets::{Paragraph, Widget},
};
use ropey::Rope;

use crate::theme::Theme;

#[derive(Default)]
pub struct Document {
    content: Rope,
    mode: Mode,
    position_byte: usize,
    position_x: usize,
    scroll_y: usize,
    // TODO: Might be better to put this behind a `Cell`?
    // TODO: Invalidate this after resize?
    last_view_area: Option<Rect>,
}

impl Document {
    pub fn new(s: String) -> Self {
        Self {
            content: s.into(),
            mode: Mode::Normal,
            position_byte: 0,
            position_x: 0,
            scroll_y: 0,
            last_view_area: None,
        }
    }

    pub fn enter_normal(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_select(&mut self) {
        self.mode = Mode::Select {
            start_byte: self.position_byte,
            line_mode: false,
        };
    }

    pub fn enter_select_line(&mut self) {
        self.mode = Mode::Select {
            start_byte: self.position_byte,
            line_mode: true,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    #[must_use]
    pub fn render(&mut self, theme: &Theme, area: Rect, buf: &mut Buffer) -> Option<Position> {
        let [editor, status] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        #[expect(unused_variables)]
        let area = ();

        let (_, y) = self.position();
        let selection = self.selection();

        let line_number_width = (self.scroll_y + editor.height as usize)
            .to_string()
            .chars()
            .count() as u16
            + 4;

        let layout =
            Layout::horizontal([Constraint::Length(line_number_width), Constraint::Min(0)]);

        self.last_view_area = Some(layout.areas::<2>(editor)[1]);

        buf.set_style(editor, theme.editor);

        let scrolled_bytes = self.content.line_to_byte(self.scroll_y);

        let mut rows = editor.rows();
        let mut byte = scrolled_bytes;
        let mut cursor = None;

        for (idx, line) in self
            .content
            .byte_slice(scrolled_bytes..)
            .lines()
            .enumerate()
        {
            let Some(row) = rows.next() else {
                break;
            };

            let [nums, text] = layout.areas(row);

            let line_style = theme.editor.patch(
                (self.scroll_y + idx == y)
                    .then_some(theme.active_line)
                    .unwrap_or_default(),
            );

            buf.set_style(row, line_style);

            Paragraph::new(format!("  {}  ", self.scroll_y + idx + 1))
                .style(line_style.patch(theme.line_numbers))
                .alignment(Alignment::Right)
                .render(nums, buf);

            if self.position_byte == byte {
                cursor = Some(text.as_position())
            }

            let mut byte_x = byte;

            for (ch, pos) in line.chars().zip(text.positions()) {
                let mut cell = Cell::EMPTY;
                cell.set_char(ch);
                cell.set_style(line_style);

                if selection.contains(&byte_x) {
                    cell.set_style(Style::new().on_light_blue());
                }

                if ch == '\n' {
                    cell.set_char(' ');
                }

                buf[pos] = cell;

                if self.position_byte == byte_x {
                    cursor = Some(pos)
                }

                byte_x += ch.len_utf8();
            }

            byte += line.len_bytes();
        }

        let mode = match self.mode {
            Mode::Normal => "NOR",
            Mode::Insert => "INS",
            Mode::Select {
                line_mode: false, ..
            } => "SEL",
            Mode::Select {
                line_mode: true, ..
            } => "LIN",
        };

        Paragraph::new(format!(" {mode}")).render(status, buf);

        let (line_idx, x_offset) = self.position();
        Paragraph::new(format!("{line_idx}:{x_offset} "))
            .style(theme.status_bar)
            .alignment(Alignment::Right)
            .render(status, buf);

        cursor
    }

    fn selection(&self) -> RangeInclusive<usize> {
        match self.mode {
            Mode::Select {
                start_byte,
                line_mode: false,
            } => self.position_byte.min(start_byte)..=self.position_byte.max(start_byte),
            Mode::Select {
                start_byte,
                line_mode: true,
            } => {
                let start_line_idx = self.content.byte_to_line(start_byte);
                let end_line_idx = self.content.byte_to_line(self.position_byte);

                let first_line_idx = start_line_idx.min(end_line_idx);
                let last_line_idx = start_line_idx.max(end_line_idx);

                let end_line = self.content.line(last_line_idx);

                let start_byte = self.content.line_to_byte(first_line_idx);
                let end_byte = self
                    .content
                    .line_to_byte(last_line_idx)
                    .add(end_line.len_chars())
                    .saturating_sub(1);

                start_byte.min(end_byte)..=start_byte.max(end_byte)
            }
            _ => self.position_byte..=self.position_byte,
        }
    }

    #[inline(always)]
    pub fn position(&self) -> (usize, usize) {
        // TODO: That ain't efficient.
        let line_idx = self.content.byte_to_line(self.position_byte);
        let line_start = self.content.line_to_byte(line_idx);

        (self.position_byte - line_start, line_idx)
    }

    #[inline(always)]
    fn move_to(&mut self, line_idx: usize, x_byte: usize) {
        let line_idx = line_idx.min(self.content.len_lines().saturating_sub(1));

        let line_start = self.content.line_to_byte(line_idx);

        let line = self.content.line(line_idx);
        let x_offset = x_byte.min(line.len_bytes().saturating_sub(1));

        self.position_byte = line_start + x_offset;
    }

    #[inline(always)]
    fn update_position_x(&mut self) {
        let (x, _) = self.position();
        self.position_x = x;
    }

    pub fn move_up(&mut self) {
        let (x_offset, line_idx) = self.position();
        self.move_to(line_idx.saturating_sub(1), x_offset.max(self.position_x));
    }

    pub fn move_down(&mut self) {
        let (x_offset, line_idx) = self.position();
        self.move_to(line_idx + 1, x_offset.max(self.position_x));
    }

    pub fn move_left(&mut self) {
        let (x_offset, line_idx) = self.position();
        self.move_to(line_idx, x_offset.saturating_sub(1));
        self.update_position_x();
    }

    pub fn move_right(&mut self) {
        let (x_offset, line_idx) = self.position();
        self.move_to(line_idx, x_offset + 1);
        self.update_position_x();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_y = self
            .scroll_y
            .add(1)
            .min(self.content.len_lines().saturating_sub(1));
    }

    pub fn scroll_to_cursor(&mut self) {
        let (_, y) = self.position();
        let Rect { height, .. } = self
            .last_view_area
            .expect("scrolled to cursor before first render");

        self.scroll_y = self
            .scroll_y
            .clamp(y.saturating_sub(usize::from(height.saturating_sub(1))), y);
    }

    pub fn move_to_view(&mut self) {
        let (_, y) = self.position();
        let Rect { height, .. } = self
            .last_view_area
            .expect("scrolled to cursor before first render");

        let y = y.clamp(
            self.scroll_y,
            self.scroll_y.add(usize::from(height)).saturating_sub(1),
        );

        self.move_to(y, self.position_x);
    }

    pub fn insert(&mut self, ch: char) {
        let idx = self.content.byte_to_char(self.position_byte);
        self.content.insert_char(idx, ch);
        self.position_byte += ch.len_utf8();
    }

    pub fn remove_before(&mut self) {
        let Some(idx) = self.content.byte_to_char(self.position_byte).checked_sub(1) else {
            return;
        };
        self.position_byte -= self.content.char(idx).len_utf8();
        self.content.remove(idx..=idx);
    }

    pub fn remove(&mut self) {
        let range = self.selection();
        let start_char = self.content.byte_to_char(*range.start());
        let end_char = self
            .content
            .byte_to_char(*range.end())
            .min(self.content.len_chars().saturating_sub(1));
        self.content.remove(start_char..=end_char);

        self.position_byte = *range.start();

        if let Mode::Select { start_byte, .. } = &mut self.mode {
            *start_byte = *range.start();
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Select {
        start_byte: usize,
        line_mode: bool,
    },
}
