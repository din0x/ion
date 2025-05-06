use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::File,
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

use crate::{
    command::Command,
    document::{Document, Mode},
    keymap::Keymap,
    theme::Theme,
};

pub struct App {
    document: Option<(PathBuf, Document)>,
    documents: BTreeMap<PathBuf, Document>,
    command: Option<String>,
    err: Option<Box<dyn Error>>,

    commands: HashMap<String, Command<()>>,
    pub keymap: Keymap,
    pub theme: Theme,

    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            document: None,
            documents: BTreeMap::new(),
            command: None,
            err: None,
            commands: HashMap::new(),
            keymap: Keymap::default(),
            theme: Theme::default(),
            exit: false,
        }
    }

    pub fn document_mut(&mut self) -> Option<(&Path, &mut Document)> {
        self.document
            .as_mut()
            .map(|(name, doc)| (name.as_path(), doc))
    }

    pub fn view(&mut self, frame: &mut Frame) {
        let [editor, command] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        if let Some((_path, doc)) = &mut self.document {
            if let Some(pos) = doc.render(&self.theme, editor, frame.buffer_mut()) {
                frame.set_cursor_position(pos);
            }
        }

        match self.command.as_ref() {
            Some(v) => {
                Paragraph::new(format!(":{v}"))
                    .style(self.theme.editor)
                    .render(command, frame.buffer_mut());

                frame.set_cursor_position((
                    command.as_position().x + 1 + v.len() as u16,
                    command.as_position().y,
                ));
            }
            None if let Some(err) = &self.err => {
                Paragraph::new(err.to_string())
                    .style(self.theme.editor)
                    .red()
                    .render(command, frame.buffer_mut());
            }
            None => frame.buffer_mut().set_style(command, self.theme.editor),
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
                    if self.command.is_none() {
                        if let Some((_, doc)) = self.document_mut() {
                            doc.enter_normal();
                        }
                    }

                    self.command = None;
                    self.err = None;
                }

                KeyCode::Char(ch) if let Some(command) = self.command.as_mut() => command.push(ch),
                KeyCode::Backspace if let Some(command) = self.command.as_mut() => {
                    _ = command.pop()
                }
                KeyCode::Enter if let Some(command) = self.command.take() => {
                    if let Err(err) = self.run_command(&command) {
                        self.err = Some(err.into());
                    }
                }
                key if let Some((_, doc)) = &mut self.document
                    && doc.mode() == Mode::Insert =>
                {
                    match key {
                        KeyCode::Enter => doc.insert('\n'),
                        KeyCode::Char(ch) => doc.insert(ch),
                        KeyCode::Backspace => doc.remove_before(),
                        _ => {}
                    }
                }
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
                if let Some((_, doc)) = &mut self.document {
                    doc.scroll_up();
                    doc.move_to_view();
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            }) => {
                if let Some((_, doc)) = &mut self.document {
                    doc.scroll_down();
                    doc.move_to_view();
                }
            }
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn open_document(&mut self, path: PathBuf) -> io::Result<()> {
        let mut buf = String::new();
        let mut reader = BufReader::new(File::open(&path)?);
        reader.read_to_string(&mut buf)?;

        let doc = Document::new(buf);
        let Some((path, doc)) = self.document.replace((path, doc)) else {
            return Ok(());
        };

        self.documents.insert(path, doc);
        Ok(())
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
        self.command = Some(String::new())
    }
}
