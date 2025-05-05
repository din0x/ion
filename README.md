# `ion`

A modal text editor, written in Rust. 

## Keymap

| Key | Description | Action |
| --- | ----------- | ------ |
| `i` | Enter insert mode | `enter_insert` |
| `v` | Enter select mode | `enter_select` |
| `V` | Enter line select mode | `enter_select_line` |
| `d` | Remove selected text | `remove_selected` |
| `h` | Move left | `move_left` |
| `j` | Move up | `move_up` |
| `k` | Move down | `move_down` |
| `l` | Move right | `move_right` |
| `w` | Move next word start | `move_next_word_start` |
| `e` | Move next word end | `move_next_word_end` |
