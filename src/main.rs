#![feature(
    if_let_guard,
    let_chains,
    macro_metavar_expr,
    round_char_boundary,
    iter_map_windows,
    str_as_str
)]

use clap::Parser;
use ratatui::crossterm::{self, cursor, event, execute, queue, terminal};
use std::{io::stdout, path::PathBuf};

use app::App;

mod app;
mod command;
mod default;
mod document;
mod input;
mod keymap;
mod language;
mod theme;

#[derive(Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut app = if let Some(file) = args.file {
        App::open(&file)
    } else {
        App::new()
    };

    default::init(&mut app);

    queue!(
        stdout(),
        cursor::SetCursorStyle::SteadyBlock,
        event::EnableMouseCapture
    )
    .unwrap();

    let mut terminal = ratatui::init();

    while !app.exit {
        execute!(stdout(), terminal::BeginSynchronizedUpdate).unwrap();
        terminal.draw(|frame| app.view(frame)).unwrap();
        queue!(stdout(), terminal::EndSynchronizedUpdate).unwrap();

        let ev = crossterm::event::read().unwrap();
        app.handle_ev(ev);
    }

    ratatui::restore();
}
