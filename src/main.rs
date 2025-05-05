#![feature(
    if_let_guard,
    let_chains,
    macro_metavar_expr,
    round_char_boundary,
    iter_map_windows
)]

use std::{io::stdout, path::PathBuf};
use clap::Parser;
use ratatui::crossterm::{self, cursor, queue};

use app::App;

mod app;
mod command;
mod default;
mod document;
mod keymap;
mod theme;

#[derive(Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();
    default::init(&mut app);

    if let Some(file) = args.file {
        _ = app.open_document(file);
    }

    queue!(stdout(), cursor::SetCursorStyle::SteadyBlock).unwrap();
    let mut terminal = ratatui::init();

    while !app.exit {
        terminal.draw(|frame| app.view(frame)).unwrap();

        let ev = crossterm::event::read().unwrap();
        app.handle_ev(ev);
    }

    ratatui::restore();
}
