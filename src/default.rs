use crate::{app::App, command::Command};

pub fn init(app: &mut App) {
    _ = app.open_document("Cargo.lock".into());
    app.keymap.insert(
        'i',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.enter_insert())),
    );
    app.keymap.insert(
        'v',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.enter_select())),
    );
    app.keymap.insert(
        'V',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.enter_select_line())),
    );
    app.keymap.insert(
        'd',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.remove())),
    );
    app.keymap.insert(
        'h',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.move_left())),
    );
    app.keymap.insert(
        'j',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.move_up();
                doc.scroll_to_cursor();
            })
        }),
    );
    app.keymap.insert(
        'k',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.move_down();
                doc.scroll_to_cursor();
            })
        }),
    );
    app.keymap.insert(
        'l',
        Command::new(|app, ()| _ = app.document_mut().map(|(_, doc)| doc.move_right())),
    );
    app.keymap.insert(
        'w',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.move_next_word();
                doc.scroll_to_cursor();
            })
        }),
    );
    app.keymap.insert(
        'e',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.move_next_word_end();
                doc.scroll_to_cursor();
            })
        }),
    );
    app.keymap.insert(
        'b',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.move_prev_word_start();
                doc.scroll_to_cursor();
            })
        }),
    );
    app.keymap.insert(
        'a',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.scroll_up();
                doc.move_to_view();
            })
        }),
    );
    app.keymap.insert(
        's',
        Command::new(|app, ()| {
            _ = app.document_mut().map(|(_, doc)| {
                doc.scroll_down();
                doc.move_to_view();
            })
        }),
    );
}
