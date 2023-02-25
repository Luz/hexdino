use crossterm::event::KeyCode;

// We do not care what platform we are on, we convert the platform independent events
// (E.g. the Enter, Esc and Backspace) to simple characters.
// We later need these simple fix values when using the pest parser for further processing.
pub fn extract(input: KeyCode) -> Option<char> {
    let value = match input {
        // This is the most important one:
        KeyCode::Char(z) => Some(z), // Actually extract the character!

        // These characters need to be same as in application!
        KeyCode::Backspace => Some('\u{7f}'),
        KeyCode::Enter => Some('\n'),
        KeyCode::Esc => Some('\u{1b}'),
        KeyCode::Left => Some('h'),
        KeyCode::Right => Some('l'),
        KeyCode::Up => Some('k'),
        KeyCode::Down => Some('j'),
        KeyCode::Home => Some('0'),
        KeyCode::End => Some('$'),
        KeyCode::Tab => Some('J'),     // TODO: is this good?
        KeyCode::BackTab => Some('J'), // TODO: is this good?
        KeyCode::Insert => Some('i'),

        // Keys that might get added in the future
        /*
        KeyCode::PageUp => None,
        KeyCode::PageDown => None,
        KeyCode::Delete => None,
        KeyCode::F(_nr) => None,
        KeyCode::Null => None,
        KeyCode::CapsLock => None,
        KeyCode::ScrollLock => None,
        KeyCode::NumLock => None,
        KeyCode::PrintScreen => None,
        KeyCode::Pause => None,
        KeyCode::Menu => None,
        KeyCode::KeypadBegin => None,
        KeyCode::Media(_m) => None,
        KeyCode::Modifier(_m) => None,
        */
        // All unhandled KeyCodes:
        _ => None,
    };
    value
}
