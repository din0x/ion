use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Alignment, Constraint, Layout, Position, Rect},
    widgets::{Paragraph, Widget},
};
use ropey::Rope;
use std::ops::{Add, RangeInclusive};
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, QueryMatches, TextProvider, Tree};

use crate::{language::Language, theme::Theme};

#[derive(Default)]
pub struct Document {
    content: Rope,
    tree: Option<Tree>,
    mode: Mode,
    position_byte: usize,
    position_x: usize,
    scroll_y: usize,
    // TODO: Might be better to put this behind a `Cell`?
    // TODO: Invalidate this after resize?
    last_view_area: Option<Rect>,
}

impl Document {
    pub fn new(content: Rope) -> Self {
        Self {
            content,
            tree: None,
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

    pub fn rope(&self) -> &Rope {
        &self.content
    }

    pub fn parse(&mut self, parser: &mut Parser) -> &Tree {
        let tree = parser
            .parse_with(
                &mut |byte, _| -> &[u8] {
                    let Some((mut chunks, chunk_start, ..)) = self.rope().get_chunks_at_byte(byte)
                    else {
                        return &[];
                    };

                    let offset = byte - chunk_start;

                    let Some(chunk) = chunks.next() else {
                        return &[];
                    };

                    let chunk = &chunk[offset..];

                    if chunk.is_empty() {
                        for chunk in chunks {
                            if !chunk.is_empty() {
                                return chunk.as_bytes();
                            }
                        }
                        &[]
                    } else {
                        chunk.as_bytes()
                    }
                },
                self.tree.as_ref(),
            )
            .expect("Parser::set_language was called");

        self.tree.insert(tree)
    }

    #[must_use]
    pub fn render(
        &mut self,
        language: &mut Language,
        theme: &Theme,
        area: Rect,
        buf: &mut Buffer,
    ) -> Option<Position> {
        self.parse(language.parser());
        let tree = self.tree.as_ref().unwrap();
        let text_provider = RopeTextProvider::new(&self.content);
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(language.highlights(), tree.root_node(), text_provider);
        let styles_vec = highlights_from_matches(language.highlights(), matches);
        let mut styles = styles_vec.iter().peekable();

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

            let mut byte_x = byte;

            let mut positions = text.positions();
            for (ch, pos) in line.chars().zip(&mut positions) {
                let mut cell = Cell::EMPTY;
                cell.set_char(ch);
                cell.set_style(line_style);

                while styles.peek().is_some_and(|p| p.before(byte_x)) {
                    styles.next();
                }

                if let Some(peek) = styles.peek()
                    && peek.contains(byte_x)
                {
                    cell.set_style(theme.get_token_style(peek.style));
                }

                if selection.contains(&byte_x) {
                    cell.set_style(theme.editor.patch(theme.selection));
                }

                if ch == '\n' {
                    cell.set_char(' ');
                }

                buf[pos] = cell;

                if self.position_byte == byte_x {
                    cursor = Some(pos);
                }

                byte_x += ch.len_utf8();
            }

            if self.position_byte == byte_x {
                cursor = positions.next()
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
        Paragraph::new(format!("{}:{} ", line_idx + 1, x_offset + 1))
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
        let last_line_idx = self.content.len_lines().saturating_sub(1);
        let line_idx = line_idx.min(last_line_idx);

        let line_start = self.content.line_to_byte(line_idx);

        let line = self.content.line(line_idx);
        let max_x_offset = line
            .len_bytes()
            .saturating_sub(if line_idx == last_line_idx { 0 } else { 1 });
        let x_offset = x_byte.min(max_x_offset);

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

    fn find_next_word(&self) -> Option<usize> {
        let mut offset = self.position_byte;
        let chars = self
            .content
            .chars_at(self.content.byte_to_char(self.position_byte));
        chars
            .map(|ch| {
                let result = (offset, ch);
                offset += ch.len_utf8();
                result
            })
            .map_windows(|[(_, a), (byte, b)]| (*byte, CharKind::new(*a) != CharKind::new(*b)))
            .filter_map(|(byte, is_boundary)| is_boundary.then_some(byte))
            .next()
    }

    fn find_next_word_end(&mut self) -> Option<usize> {
        let mut offset = self.position_byte;
        let chars = self
            .content
            .chars_at(self.content.byte_to_char(self.position_byte));
        chars
            .map(|ch| {
                let result = (offset, ch);
                offset += ch.len_utf8();
                result
            })
            .skip(1)
            .map_windows(|[(byte, a), (_, b)]| (*byte, CharKind::new(*a) != CharKind::new(*b)))
            .filter_map(|(byte, is_boundary)| (is_boundary).then_some(byte))
            .next()
    }

    fn find_prev_word_start(&mut self) -> Option<usize> {
        let mut offset = self.position_byte
            + self
                .content
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or_default();

        self.content
            .chars_at(
                self.content
                    .byte_to_char(self.position_byte)
                    .add(1)
                    .min(self.content.len_chars()),
            )
            .reversed()
            .map(|ch| {
                let byte = offset;
                offset -= ch.len_utf8();
                (byte, ch)
            })
            .skip(1)
            .map_windows(|[(_, a), (byte, b)]| (*byte, CharKind::new(*a) != CharKind::new(*b)))
            .filter_map(|(byte, is_boundary)| (is_boundary).then_some(byte))
            .next()
    }

    pub fn move_next_word(&mut self) {
        self.position_byte = self.find_next_word().unwrap_or(self.content.len_bytes());
        self.update_position_x();
    }

    pub fn move_next_word_end(&mut self) {
        self.position_byte = self
            .find_next_word_end()
            .unwrap_or(self.content.len_bytes());
        self.update_position_x();
    }

    pub fn move_prev_word_start(&mut self) {
        self.position_byte = self.find_prev_word_start().unwrap_or(0);
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

        let (x, y) = self.position();
        if let Some(tree) = &mut self.tree {
            let new_end_byte = self.position_byte + ch.len_utf8();

            let new_end_line = self.content.byte_to_line(new_end_byte);
            let new_end_col = new_end_byte - self.content.line_to_byte(new_end_line);

            tree.edit(&InputEdit {
                start_byte: self.position_byte,
                start_position: Point::new(y, x),
                old_end_byte: self.position_byte,
                old_end_position: Point::new(y, x),
                new_end_byte,
                new_end_position: Point::new(new_end_line, new_end_col),
            });
        }

        self.position_byte += ch.len_utf8();

        self.update_position_x();
    }

    fn byte_to_point(&self, byte: usize) -> Point {
        let row = self.content.byte_to_line(byte);
        let column = byte - self.content.line_to_byte(row);
        Point::new(row, column)
    }

    pub fn remove_before(&mut self) {
        let Some(idx) = self.content.byte_to_char(self.position_byte).checked_sub(1) else {
            return;
        };

        let byte_idx = self.content.char_to_byte(idx);
        let point = self.byte_to_point(byte_idx);
        let end_point = self.byte_to_point(self.position_byte);

        if let Some(tree) = &mut self.tree {
            tree.edit(&InputEdit {
                start_byte: byte_idx,
                start_position: point,
                old_end_byte: self.position_byte,
                old_end_position: end_point,
                new_end_byte: byte_idx,
                new_end_position: point,
            });
        }

        self.position_byte -= self.content.char(idx).len_utf8();
        self.content.remove(idx..=idx);
        self.update_position_x();
    }

    pub fn remove(&mut self) {
        if self.content.len_bytes() == 0 {
            return;
        }

        let range = self.selection();

        let start_point = self.byte_to_point(*range.start());
        let end_point = self.byte_to_point(*range.end());

        if let Some(tree) = &mut self.tree {
            tree.edit(&InputEdit {
                start_byte: *range.start(),
                start_position: start_point,
                old_end_byte: *range.end(),
                old_end_position: end_point,
                new_end_byte: *range.start(),
                new_end_position: start_point,
            });
        }

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

        self.update_position_x();
        self.scroll_to_cursor();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharKind {
    Lf,
    Whitespace,
    Other,
}

impl CharKind {
    fn new(ch: char) -> Self {
        match ch {
            '\n' | '\r' => Self::Lf,
            ch if ch.is_whitespace() => Self::Whitespace,
            _ => Self::Other,
        }
    }
}

struct RopeByteChunksIterator<'a> {
    chunks: ropey::iter::Chunks<'a>,
    skip: usize,
    chunk_byte_start: usize,
    end: usize,
}

impl<'a> Iterator for RopeByteChunksIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.chunks.next()?;
        let mut part = &chunk[self.skip..];
        let to_take = self.end - self.chunk_byte_start - self.skip;

        if part.len() > to_take {
            part = &part[..(part.len() - to_take)];
        }

        self.skip = 0;

        Some(part.as_bytes())
    }
}

struct RopeTextProvider<'a>(&'a Rope);

impl<'a> RopeTextProvider<'a> {
    fn new(rope: &'a Rope) -> Self {
        Self(rope)
    }
}

impl<'a> TextProvider<&'a [u8]> for RopeTextProvider<'a> {
    type I = RopeByteChunksIterator<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        let start = node.start_byte();
        let end = node.end_byte();
        let (chunks, chunk_byte_start, ..) = self.0.chunks_at_byte(start);

        RopeByteChunksIterator {
            chunks,
            skip: start - chunk_byte_start,
            chunk_byte_start,
            end,
        }
    }
}

fn highlights_from_matches<'a>(
    query: &'a Query,
    matches: QueryMatches<'a, 'a, RopeTextProvider<'a>, &'a [u8]>,
) -> Vec<TokenStyle<'a>> {
    let mut colors = Vec::new();

    for mat in matches {
        for cap in mat.captures {
            let node = cap.node;

            let start_byte = node.start_byte();
            let end_byte = node.end_byte();

            let scope = query.capture_names()[cap.index as usize];

            colors.push(TokenStyle {
                end_byte,
                start_byte,
                style: scope,
            });
        }
    }

    colors
}

#[derive(Debug, Clone)]
pub struct TokenStyle<'style> {
    start_byte: usize,
    end_byte: usize,
    pub style: &'style str,
}

impl<'style> TokenStyle<'style> {
    pub fn contains(&self, idx_byte: usize) -> bool {
        (self.start_byte..self.end_byte).contains(&idx_byte)
    }

    pub fn before(&self, idx_byte: usize) -> bool {
        self.end_byte < idx_byte
    }
}
