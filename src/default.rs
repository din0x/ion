use std::{
    fs::File,
    io::{self, BufWriter},
    path::{Path, PathBuf},
};

use crate::{app::App, command::Command, document::Document, input::Input};

pub fn init(app: &mut App) {
    app.keymap
        .insert('i', Command::new(|app, ()| _ = app.doc.enter_insert()));
    app.keymap
        .insert('v', Command::new(|app, ()| _ = app.doc.enter_select()));
    app.keymap
        .insert('V', Command::new(|app, ()| _ = app.doc.enter_select_line()));
    app.keymap
        .insert('d', Command::new(|app, ()| _ = app.doc.remove()));
    app.keymap
        .insert('h', Command::new(|app, ()| _ = app.doc.move_left()));
    app.keymap.insert(
        'j',
        Command::new(|app, ()| {
            app.doc.move_up();
            app.doc.scroll_to_cursor();
        }),
    );
    app.keymap.insert(
        'k',
        Command::new(|app, ()| {
            app.doc.move_down();
            app.doc.scroll_to_cursor();
        }),
    );
    app.keymap
        .insert('l', Command::new(|app, ()| _ = app.doc.move_right()));
    app.keymap.insert(
        'w',
        Command::new(|app, ()| {
            app.doc.move_next_word();
            app.doc.scroll_to_cursor();
        }),
    );
    app.keymap.insert(
        'e',
        Command::new(|app, ()| {
            app.doc.move_next_word_end();
            app.doc.scroll_to_cursor();
        }),
    );
    app.keymap.insert(
        'b',
        Command::new(|app, ()| {
            app.doc.move_prev_word_start();
            app.doc.scroll_to_cursor();
        }),
    );
    app.keymap.insert(
        'a',
        Command::new(|app, ()| {
            app.doc.scroll_up();
            app.doc.move_to_view();
        }),
    );
    app.keymap.insert(
        's',
        Command::new(|app, ()| {
            app.doc.scroll_down();
            app.doc.move_to_view();
        }),
    );
    app.commands.insert(
        "w".into(),
        Command::new(|app, ()| {
            if let Some(name) = &app.doc_name {
                if let Err(err) = save_doc(name, &app.doc) {
                    app.report_error(err);
                }
            } else {
                app.input = Some(
                    Input::new(|s, app| {
                        if let Err(err) = save_doc(&PathBuf::from(&s), &app.doc) {
                            app.report_error(err);
                        } else {
                            app.doc_name = Some(s.into())
                        }
                    })
                    .with_placeholder("Enter file name".into()),
                );
            }
        }),
    );
}

fn save_doc(name: &Path, doc: &Document) -> io::Result<()> {
    let file = File::create(name)?;
    let writer = BufWriter::new(file);
    doc.rope().write_to(writer)
}
