use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use ropey::Rope;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    command::Command,
    document::{Document, Mode},
    input::Input,
    keymap::Keymap,
    language::{self, Language},
    theme::Theme,
};

pub struct App {
    pub doc: Document,
    pub doc_name: Option<PathBuf>,
    pub input: Option<Input>,
    err: Option<Box<dyn Error>>,

    language: Language,
    pub commands: HashMap<String, Command<()>>,
    pub keymap: Keymap,
    pub theme: Theme,

    pub exit: bool,
}

impl App {
    pub fn open(path: &Path) -> Self {
        let content = File::open(path)
            .and_then(|file| Rope::from_reader(BufReader::new(file)))
            .unwrap_or_default();

        let doc = Document::new(content);

        Self {
            doc,
            doc_name: Some(path.into()),
            input: None,
            err: None,
            language: language::rust(),
            commands: HashMap::new(),
            keymap: Keymap::default(),
            theme: Theme::default(),
            exit: false,
        }
    }

    pub fn new() -> Self {
        Self {
            doc: Document::default(),
            doc_name: None,
            input: None,
            err: None,
            language: language::rust(),
            commands: HashMap::new(),
            keymap: Keymap::default(),
            theme: Theme::default(),
            exit: false,
        }
    }

    pub fn view(&mut self, frame: &mut Frame) {
        let [editor, input_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        if let Some(pos) =
            self.doc
                .render(&mut self.language, &self.theme, editor, frame.buffer_mut())
        {
            frame.set_cursor_position(pos);
        }

        match self.input.as_ref() {
            Some(input) => {
                let pos = input.render(&self.theme, frame.buffer_mut(), input_area);

                if let Some(pos) = pos {
                    frame.set_cursor_position(pos);
                }
            }
            None if let Some(err) = &self.err => {
                Paragraph::new(err.to_string())
                    .style(self.theme.editor)
                    .red()
                    .render(input_area, frame.buffer_mut());
            }
            None => frame.buffer_mut().set_style(input_area, self.theme.editor),
        }
    }

    pub fn handle_ev(&mut self, ev: Event) {
        match ev {
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => match code {
                KeyCode::Esc => {
                    if self.input.is_none() {
                        self.doc.enter_normal();
                    }

                    self.input = None;
                    self.err = None;
                }

                KeyCode::Char(ch) if let Some(input) = self.input.as_mut() => input.insert(ch),
                KeyCode::Backspace if let Some(input) = self.input.as_mut() => input.remove(),
                KeyCode::Enter if let Some(input) = self.input.take() => input.submit(self),
                key if self.doc.mode() == Mode::Insert => match key {
                    KeyCode::Enter => self.doc.insert('\n'),
                    KeyCode::Char(ch) => self.doc.insert(ch),
                    KeyCode::Backspace => self.doc.remove_before(),
                    KeyCode::Tab => {
                        let (x, _) = self.doc.position();
                        let spaces = if x % 4 == 0 { 4 } else { 4 - x % 4 };
                        for _ in 0..spaces {
                            self.doc.insert(' ');
                        }
                    }
                    _ => {}
                },
                KeyCode::Char(':') => self.open_command(),
                KeyCode::Char(key) => {
                    if let Some(command) = self.keymap.get(key).cloned() {
                        command.run(self, ());
                    }
                }
                _ => {}
            },
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            }) => {
                self.doc.scroll_up();
                self.doc.move_to_view();
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            }) => {
                self.doc.scroll_up();
                self.doc.move_to_view();
            }
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn report_error(&mut self, err: impl Error + 'static) {
        self.err = Some(Box::new(err));
    }

    fn run_command(&mut self, command: &str) -> Result<(), String> {
        let (name, _) = command.split_once(' ').unwrap_or((command, ""));

        if name == "q" {
            self.exit();
            return Ok(());
        }

        let Some(command) = self.commands.remove(name) else {
            return Err(format!("command not found: {name}"));
        };

        self.err = None;
        command.run(self, ());

        self.commands.insert(name.to_owned(), command);
        Ok(())
    }

    fn open_command(&mut self) {
        self.input = Some(
            Input::new(|cmd, app| {
                if let Err(err) = app.run_command(&cmd) {
                    app.err = Some(err.into());
                }
            })
            .with_symbol(':'),
        );
    }
}
