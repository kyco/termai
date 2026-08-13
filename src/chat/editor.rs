//! Pure line-editor state machine for the bottom-anchored chat input.
//!
//! No terminal I/O happens here: the editor consumes `crossterm` key events
//! and mutates an in-memory buffer, returning an [`EditorEvent`] that tells
//! the caller what to do (repaint, submit, cancel, exit). This keeps the
//! whole editing surface unit-testable without a TTY.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// The result of feeding one key event into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorEvent {
    /// Nothing changed; no repaint needed.
    None,
    /// Buffer or cursor changed; caller should repaint the input line.
    Redraw,
    /// The user pressed Enter on a non-empty buffer.
    Submit(String),
    /// Esc, or Ctrl-C on an empty buffer. Caller decides what "cancel" means.
    Cancel,
    /// Ctrl-D on an empty buffer: end the session.
    Exit,
}

/// A single-line editor with history, word operations and unicode-safe
/// cursor movement. The cursor is a byte offset into `buffer`, always kept
/// on a `char` boundary.
pub struct LineEditor {
    buffer: String,
    cursor: usize,
    history: Vec<String>,
    /// `Some(i)` while navigating history; `None` when editing a fresh line.
    history_index: Option<usize>,
    /// The in-progress line stashed away while navigating history.
    stash: String,
}

impl LineEditor {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            stash: String::new(),
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Byte offset of the cursor within the buffer (on a char boundary).
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Preload history (e.g. from a history file). Oldest first.
    pub fn load_history(&mut self, entries: impl IntoIterator<Item = String>) {
        for e in entries {
            let trimmed = e.trim();
            if !trimmed.is_empty() {
                self.history.push(trimmed.to_string());
            }
        }
    }

    /// Replace the whole buffer (used by palette tab-completion).
    pub fn set_text(&mut self, text: &str) {
        self.buffer = text.to_string();
        self.cursor = self.buffer.len();
        self.history_index = None;
    }

    /// Insert a string at the cursor (used for bracketed paste).
    /// Newlines are replaced with spaces since this is a single-line editor.
    pub fn insert_str(&mut self, text: &str) {
        let sanitized: String = text
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .filter(|c| !c.is_control() || *c == '\t')
            .collect();
        self.buffer.insert_str(self.cursor, &sanitized);
        self.cursor += sanitized.len();
    }

    /// Feed one key event into the editor.
    pub fn handle_key(&mut self, key: KeyEvent) -> EditorEvent {
        if key.kind == KeyEventKind::Release {
            return EditorEvent::None;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            // --- control chords -------------------------------------------
            KeyCode::Char('c') if ctrl => {
                if self.buffer.is_empty() {
                    EditorEvent::Cancel
                } else {
                    self.buffer.clear();
                    self.cursor = 0;
                    self.history_index = None;
                    EditorEvent::Redraw
                }
            }
            KeyCode::Char('d') if ctrl => {
                if self.buffer.is_empty() {
                    EditorEvent::Exit
                } else {
                    self.delete_at_cursor();
                    EditorEvent::Redraw
                }
            }
            KeyCode::Char('a') if ctrl => {
                self.cursor = 0;
                EditorEvent::Redraw
            }
            KeyCode::Char('e') if ctrl => {
                self.cursor = self.buffer.len();
                EditorEvent::Redraw
            }
            KeyCode::Char('w') if ctrl => {
                let start = self.prev_word_boundary();
                self.buffer.replace_range(start..self.cursor, "");
                self.cursor = start;
                EditorEvent::Redraw
            }
            KeyCode::Char('u') if ctrl => {
                self.buffer.clear();
                self.cursor = 0;
                EditorEvent::Redraw
            }
            KeyCode::Char('k') if ctrl => {
                self.buffer.truncate(self.cursor);
                EditorEvent::Redraw
            }
            KeyCode::Char('b') if alt => {
                self.cursor = self.prev_word_boundary();
                EditorEvent::Redraw
            }
            KeyCode::Char('f') if alt => {
                self.cursor = self.next_word_boundary();
                EditorEvent::Redraw
            }

            // --- plain typing ---------------------------------------------
            KeyCode::Char(c) if !ctrl && !alt => {
                self.buffer.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                EditorEvent::Redraw
            }

            // --- movement -------------------------------------------------
            KeyCode::Left if ctrl || alt => {
                self.cursor = self.prev_word_boundary();
                EditorEvent::Redraw
            }
            KeyCode::Right if ctrl || alt => {
                self.cursor = self.next_word_boundary();
                EditorEvent::Redraw
            }
            KeyCode::Left => {
                self.cursor = self.prev_char_boundary();
                EditorEvent::Redraw
            }
            KeyCode::Right => {
                self.cursor = self.next_char_boundary();
                EditorEvent::Redraw
            }
            KeyCode::Home => {
                self.cursor = 0;
                EditorEvent::Redraw
            }
            KeyCode::End => {
                self.cursor = self.buffer.len();
                EditorEvent::Redraw
            }

            // --- deletion -------------------------------------------------
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let prev = self.prev_char_boundary();
                    self.buffer.replace_range(prev..self.cursor, "");
                    self.cursor = prev;
                    EditorEvent::Redraw
                } else {
                    EditorEvent::None
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.buffer.len() {
                    self.delete_at_cursor();
                    EditorEvent::Redraw
                } else {
                    EditorEvent::None
                }
            }

            // --- history --------------------------------------------------
            KeyCode::Up => self.history_prev(),
            KeyCode::Down => self.history_next(),

            // --- submit / cancel ------------------------------------------
            KeyCode::Enter => {
                let text = std::mem::take(&mut self.buffer);
                self.cursor = 0;
                self.history_index = None;
                self.stash.clear();
                if text.trim().is_empty() {
                    EditorEvent::Redraw
                } else {
                    if self.history.last().map(|s| s.as_str()) != Some(text.as_str()) {
                        self.history.push(text.clone());
                    }
                    EditorEvent::Submit(text)
                }
            }
            KeyCode::Esc => EditorEvent::Cancel,

            _ => EditorEvent::None,
        }
    }

    // --- helpers ---------------------------------------------------------

    fn delete_at_cursor(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.next_char_boundary();
            self.buffer.replace_range(self.cursor..next, "");
        }
    }

    fn prev_char_boundary(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }
        let mut i = self.cursor - 1;
        while i > 0 && !self.buffer.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn next_char_boundary(&self) -> usize {
        if self.cursor >= self.buffer.len() {
            return self.buffer.len();
        }
        let mut i = self.cursor + 1;
        while i < self.buffer.len() && !self.buffer.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    /// Start of the word before the cursor (skips trailing whitespace first).
    fn prev_word_boundary(&self) -> usize {
        let before = &self.buffer[..self.cursor];
        let mut chars: Vec<(usize, char)> = before.char_indices().collect();
        // Skip whitespace immediately before the cursor
        while let Some(&(_, c)) = chars.last() {
            if c.is_whitespace() {
                chars.pop();
            } else {
                break;
            }
        }
        // Skip the word itself
        let mut boundary = 0;
        while let Some(&(i, c)) = chars.last() {
            if c.is_whitespace() {
                break;
            }
            boundary = i;
            chars.pop();
        }
        if chars.is_empty() {
            0
        } else {
            boundary
        }
    }

    /// End of the word after the cursor (skips leading whitespace first).
    fn next_word_boundary(&self) -> usize {
        let after = &self.buffer[self.cursor..];
        let mut offset = 0;
        let mut iter = after.char_indices().peekable();
        // Skip whitespace
        while let Some(&(i, c)) = iter.peek() {
            if c.is_whitespace() {
                offset = i + c.len_utf8();
                iter.next();
            } else {
                break;
            }
        }
        // Skip the word
        while let Some(&(i, c)) = iter.peek() {
            if c.is_whitespace() {
                break;
            }
            offset = i + c.len_utf8();
            iter.next();
        }
        self.cursor + offset
    }

    fn history_prev(&mut self) -> EditorEvent {
        if self.history.is_empty() {
            return EditorEvent::None;
        }
        match self.history_index {
            None => {
                self.stash = std::mem::take(&mut self.buffer);
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return EditorEvent::None,
            Some(i) => self.history_index = Some(i - 1),
        }
        if let Some(i) = self.history_index {
            self.buffer = self.history[i].clone();
            self.cursor = self.buffer.len();
        }
        EditorEvent::Redraw
    }

    fn history_next(&mut self) -> EditorEvent {
        match self.history_index {
            None => EditorEvent::None,
            Some(i) if i + 1 < self.history.len() => {
                self.history_index = Some(i + 1);
                self.buffer = self.history[i + 1].clone();
                self.cursor = self.buffer.len();
                EditorEvent::Redraw
            }
            Some(_) => {
                self.history_index = None;
                self.buffer = std::mem::take(&mut self.stash);
                self.cursor = self.buffer.len();
                EditorEvent::Redraw
            }
        }
    }
}

impl Default for LineEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn alt(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }

    fn type_str(ed: &mut LineEditor, s: &str) {
        for c in s.chars() {
            ed.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn typing_inserts_chars_and_moves_cursor() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(key(KeyCode::Char('h'))), EditorEvent::Redraw);
        type_str(&mut ed, "ello");
        assert_eq!(ed.buffer(), "hello");
        assert_eq!(ed.cursor(), 5);
    }

    #[test]
    fn shift_chars_still_insert() {
        let mut ed = LineEditor::new();
        ed.handle_key(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT));
        assert_eq!(ed.buffer(), "H");
    }

    #[test]
    fn release_events_are_ignored() {
        let mut ed = LineEditor::new();
        let mut ev = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        ev.kind = KeyEventKind::Release;
        assert_eq!(ed.handle_key(ev), EditorEvent::None);
        assert_eq!(ed.buffer(), "");
    }

    #[test]
    fn unicode_insert_and_cursor_moves() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "héllo 世界");
        assert_eq!(ed.buffer(), "héllo 世界");
        // Cursor at byte end
        assert_eq!(ed.cursor(), "héllo 世界".len());
        // Move left over 界 (3 bytes)
        ed.handle_key(key(KeyCode::Left));
        assert_eq!(ed.cursor(), "héllo 世界".len() - 3);
        // Move left over 世
        ed.handle_key(key(KeyCode::Left));
        assert_eq!(ed.cursor(), "héllo ".len());
        // Insert in the middle
        ed.handle_key(key(KeyCode::Char('X')));
        assert_eq!(ed.buffer(), "héllo X世界");
    }

    #[test]
    fn unicode_backspace_removes_whole_char() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "aé漢");
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.buffer(), "aé");
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.buffer(), "a");
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.buffer(), "");
        // Backspace on empty buffer is a no-op
        assert_eq!(ed.handle_key(key(KeyCode::Backspace)), EditorEvent::None);
    }

    #[test]
    fn delete_removes_char_at_cursor() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "abc");
        ed.handle_key(key(KeyCode::Home));
        ed.handle_key(key(KeyCode::Delete));
        assert_eq!(ed.buffer(), "bc");
        assert_eq!(ed.cursor(), 0);
        // Delete at end is a no-op
        ed.handle_key(key(KeyCode::End));
        assert_eq!(ed.handle_key(key(KeyCode::Delete)), EditorEvent::None);
    }

    #[test]
    fn home_end_and_ctrl_a_e() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "hello");
        ed.handle_key(key(KeyCode::Home));
        assert_eq!(ed.cursor(), 0);
        ed.handle_key(key(KeyCode::End));
        assert_eq!(ed.cursor(), 5);
        ed.handle_key(ctrl('a'));
        assert_eq!(ed.cursor(), 0);
        ed.handle_key(ctrl('e'));
        assert_eq!(ed.cursor(), 5);
    }

    #[test]
    fn word_left_and_right() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "foo bar  baz");
        // Cursor at end. Word-left should land at start of "baz"
        ed.handle_key(alt(KeyCode::Left));
        assert_eq!(ed.cursor(), 9);
        // Again: start of "bar"
        ed.handle_key(alt(KeyCode::Left));
        assert_eq!(ed.cursor(), 4);
        // Again: start of "foo"
        ed.handle_key(alt(KeyCode::Left));
        assert_eq!(ed.cursor(), 0);
        // At start, stays put
        ed.handle_key(alt(KeyCode::Left));
        assert_eq!(ed.cursor(), 0);
        // Word-right: end of "foo"
        ed.handle_key(alt(KeyCode::Right));
        assert_eq!(ed.cursor(), 3);
        // Ctrl+Right also works: end of "bar"
        ed.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(ed.cursor(), 7);
        ed.handle_key(alt(KeyCode::Right));
        assert_eq!(ed.cursor(), 12);
        // At end, stays put
        ed.handle_key(alt(KeyCode::Right));
        assert_eq!(ed.cursor(), 12);
    }

    #[test]
    fn word_ops_with_unicode() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "héllo 世界 end");
        ed.handle_key(alt(KeyCode::Left)); // start of "end"
        assert_eq!(&ed.buffer()[ed.cursor()..], "end");
        ed.handle_key(alt(KeyCode::Left)); // start of "世界"
        assert_eq!(&ed.buffer()[ed.cursor()..], "世界 end");
        ed.handle_key(alt(KeyCode::Left)); // start of "héllo"
        assert_eq!(ed.cursor(), 0);
    }

    #[test]
    fn ctrl_w_deletes_word_before_cursor() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "one two three");
        ed.handle_key(ctrl('w'));
        assert_eq!(ed.buffer(), "one two ");
        ed.handle_key(ctrl('w'));
        assert_eq!(ed.buffer(), "one ");
        ed.handle_key(ctrl('w'));
        assert_eq!(ed.buffer(), "");
        // No-op on empty (still Redraw, but buffer stays empty)
        ed.handle_key(ctrl('w'));
        assert_eq!(ed.buffer(), "");
    }

    #[test]
    fn ctrl_w_mid_word() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "hello world");
        // Move cursor to just after "wor"
        for _ in 0..2 {
            ed.handle_key(key(KeyCode::Left));
        }
        ed.handle_key(ctrl('w'));
        assert_eq!(ed.buffer(), "hello ld");
        assert_eq!(ed.cursor(), 6);
    }

    #[test]
    fn ctrl_u_clears_line() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "some text");
        ed.handle_key(ctrl('u'));
        assert_eq!(ed.buffer(), "");
        assert_eq!(ed.cursor(), 0);
    }

    #[test]
    fn ctrl_k_kills_to_end() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "keep this");
        // Move to after "keep"
        for _ in 0..5 {
            ed.handle_key(key(KeyCode::Left));
        }
        ed.handle_key(ctrl('k'));
        assert_eq!(ed.buffer(), "keep");
        assert_eq!(ed.cursor(), 4);
    }

    #[test]
    fn enter_submits_and_pushes_history() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "hello");
        match ed.handle_key(key(KeyCode::Enter)) {
            EditorEvent::Submit(s) => assert_eq!(s, "hello"),
            other => panic!("expected Submit, got {:?}", other),
        }
        assert_eq!(ed.buffer(), "");
        assert_eq!(ed.cursor(), 0);
        assert_eq!(ed.history(), &["hello".to_string()]);
    }

    #[test]
    fn enter_on_empty_or_whitespace_does_not_submit() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(key(KeyCode::Enter)), EditorEvent::Redraw);
        type_str(&mut ed, "   ");
        assert_eq!(ed.handle_key(key(KeyCode::Enter)), EditorEvent::Redraw);
        assert!(ed.history().is_empty());
        assert_eq!(ed.buffer(), "");
    }

    #[test]
    fn duplicate_consecutive_history_entries_are_collapsed() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "same");
        ed.handle_key(key(KeyCode::Enter));
        type_str(&mut ed, "same");
        ed.handle_key(key(KeyCode::Enter));
        assert_eq!(ed.history().len(), 1);
    }

    #[test]
    fn history_navigation_round_trip_preserves_in_progress_buffer() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "first");
        ed.handle_key(key(KeyCode::Enter));
        type_str(&mut ed, "second");
        ed.handle_key(key(KeyCode::Enter));

        // Start typing a new line, then navigate history
        type_str(&mut ed, "in progress");
        assert_eq!(ed.handle_key(key(KeyCode::Up)), EditorEvent::Redraw);
        assert_eq!(ed.buffer(), "second");
        ed.handle_key(key(KeyCode::Up));
        assert_eq!(ed.buffer(), "first");
        // Up at the oldest entry: no change
        assert_eq!(ed.handle_key(key(KeyCode::Up)), EditorEvent::None);
        assert_eq!(ed.buffer(), "first");
        // Navigate back down
        ed.handle_key(key(KeyCode::Down));
        assert_eq!(ed.buffer(), "second");
        ed.handle_key(key(KeyCode::Down));
        // Past the newest: the stashed in-progress buffer comes back
        assert_eq!(ed.buffer(), "in progress");
        // Down again: no-op
        assert_eq!(ed.handle_key(key(KeyCode::Down)), EditorEvent::None);
    }

    #[test]
    fn history_up_on_empty_history_is_noop() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(key(KeyCode::Up)), EditorEvent::None);
        assert_eq!(ed.handle_key(key(KeyCode::Down)), EditorEvent::None);
    }

    #[test]
    fn editing_a_history_entry_then_submitting_pushes_edited_version() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "original");
        ed.handle_key(key(KeyCode::Enter));
        ed.handle_key(key(KeyCode::Up));
        type_str(&mut ed, " edited");
        match ed.handle_key(key(KeyCode::Enter)) {
            EditorEvent::Submit(s) => assert_eq!(s, "original edited"),
            other => panic!("expected Submit, got {:?}", other),
        }
        assert_eq!(ed.history().len(), 2);
    }

    #[test]
    fn ctrl_c_clears_nonempty_buffer_then_cancels() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "draft");
        assert_eq!(ed.handle_key(ctrl('c')), EditorEvent::Redraw);
        assert_eq!(ed.buffer(), "");
        assert_eq!(ed.handle_key(ctrl('c')), EditorEvent::Cancel);
    }

    #[test]
    fn ctrl_d_exits_on_empty_buffer_deletes_otherwise() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(ctrl('d')), EditorEvent::Exit);
        type_str(&mut ed, "ab");
        ed.handle_key(key(KeyCode::Home));
        assert_eq!(ed.handle_key(ctrl('d')), EditorEvent::Redraw);
        assert_eq!(ed.buffer(), "b");
    }

    #[test]
    fn esc_cancels() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(key(KeyCode::Esc)), EditorEvent::Cancel);
        type_str(&mut ed, "text stays");
        assert_eq!(ed.handle_key(key(KeyCode::Esc)), EditorEvent::Cancel);
        assert_eq!(ed.buffer(), "text stays");
    }

    #[test]
    fn paste_inserts_at_cursor_and_sanitizes_newlines() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "ab");
        ed.handle_key(key(KeyCode::Left));
        ed.insert_str("multi\nline\r\npaste");
        assert_eq!(ed.buffer(), "amulti line  pasteb");
        // Cursor sits after the pasted text
        assert_eq!(ed.cursor(), "amulti line  paste".len());
    }

    #[test]
    fn paste_unicode() {
        let mut ed = LineEditor::new();
        ed.insert_str("日本語テキスト");
        assert_eq!(ed.buffer(), "日本語テキスト");
        assert_eq!(ed.cursor(), ed.buffer().len());
    }

    #[test]
    fn set_text_replaces_buffer() {
        let mut ed = LineEditor::new();
        type_str(&mut ed, "/he");
        ed.set_text("/help");
        assert_eq!(ed.buffer(), "/help");
        assert_eq!(ed.cursor(), 5);
    }

    #[test]
    fn load_history_skips_empty_lines() {
        let mut ed = LineEditor::new();
        ed.load_history(vec![
            "one".to_string(),
            "".to_string(),
            "  ".to_string(),
            "two".to_string(),
        ]);
        assert_eq!(ed.history(), &["one".to_string(), "two".to_string()]);
    }

    #[test]
    fn tab_is_ignored_by_the_editor() {
        let mut ed = LineEditor::new();
        assert_eq!(ed.handle_key(key(KeyCode::Tab)), EditorEvent::None);
    }
}
