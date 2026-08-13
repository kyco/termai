//! Bottom-anchored input zone renderer for interactive chat.
//!
//! The anchor zone is drawn at the bottom of the terminal and consists of:
//!
//! ```text
//!   ┌ optional slash-command palette (bordered) ┐
//!   ────────────────────────────────────────────  <- separator
//!     › user input line
//!     model · session · tokens · tools off        <- dim status line
//! ```
//!
//! Line construction is pure ([`build_lines`]) so it can be unit-tested
//! without a TTY; [`AnchorRenderer`] owns the crossterm draw/erase logic and
//! knows how many lines it painted last time so it can erase itself and let
//! conversation content flow into the terminal's native scrollback above.

use crate::chat::commands::ChatCommand;
use crossterm::{cursor, queue, terminal};
use std::io::Write;

/// Spinner frames used in the status line while a response is in flight.
pub const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

const PROMPT: &str = "  › ";
const PROMPT_WIDTH: usize = 4;
/// Maximum palette rows shown at once.
const PALETTE_MAX_ROWS: usize = 8;

/// Static status segments shown when idle.
#[derive(Debug, Clone)]
pub struct StatusInfo {
    pub model: String,
    pub session: String,
    pub token_estimate: usize,
    pub tools_enabled: bool,
    /// Non-default reasoning effort, e.g. "ultra" (appended as `· ultra`).
    pub effort: Option<String>,
}

/// Live spinner segment shown while a response streams.
#[derive(Debug, Clone)]
pub struct SpinnerInfo {
    /// Index into [`SPINNER_FRAMES`].
    pub frame: usize,
    pub elapsed_secs: f32,
    /// e.g. "thinking" or "🌐 searching the web…"
    pub label: String,
}

/// One entry in the slash-command palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteItem {
    /// Base command used for tab completion, e.g. "/save".
    pub completion: String,
    /// Full display form, e.g. "/save [name]".
    pub display: &'static str,
    pub description: &'static str,
}

/// Everything needed to render the anchor zone.
#[derive(Debug, Clone)]
pub struct AnchorState {
    pub input: String,
    /// Byte offset of the cursor within `input`.
    pub cursor: usize,
    pub status: StatusInfo,
    pub spinner: Option<SpinnerInfo>,
    /// Number of messages queued while a response streams.
    pub queued: usize,
    pub palette: Vec<PaletteItem>,
    pub palette_selected: usize,
}

/// The rendered anchor: lines top-to-bottom plus where the terminal cursor
/// should sit (row is an index into `lines`, col is a display column).
#[derive(Debug)]
pub struct AnchorLayout {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: u16,
}

/// Display width of a string (unicode-aware, ANSI-aware via `console`).
fn display_width(s: &str) -> usize {
    console::measure_text_width(s)
}

/// Truncate plain (non-ANSI) text to at most `width` display columns.
fn truncate_to_width(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let w = display_width(&c.to_string());
        if used + w > width {
            break;
        }
        out.push(c);
        used += w;
    }
    out
}

fn dim(s: &str) -> String {
    format!("\x1b[2m{}\x1b[0m", s)
}

/// Filter the command palette for the current input buffer.
///
/// The palette is shown while the buffer starts with `/` and contains no
/// whitespace yet (once arguments are being typed the palette hides).
pub fn filter_palette(input: &str) -> Vec<PaletteItem> {
    if !input.starts_with('/') || input.contains(char::is_whitespace) {
        return Vec::new();
    }
    let needle = input.to_lowercase();
    ChatCommand::command_palette()
        .into_iter()
        .filter_map(|entry| {
            let base = entry.command.split_whitespace().next().unwrap_or("");
            let alias_match = entry
                .aliases
                .split(',')
                .map(|a| a.trim())
                .any(|a| a.starts_with(&needle));
            if base.starts_with(&needle) || alias_match {
                Some(PaletteItem {
                    completion: base.to_string(),
                    display: entry.command,
                    description: entry.description,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Format a token estimate compactly: `812 tok`, `2.4k tok`.
pub fn format_tokens(n: usize) -> String {
    if n < 1000 {
        format!("{} tok", n)
    } else {
        format!("{:.1}k tok", n as f64 / 1000.0)
    }
}

/// Build the anchor lines for the given state and terminal width. Pure.
pub fn build_lines(state: &AnchorState, width: usize) -> AnchorLayout {
    let width = width.max(20);
    let mut lines = Vec::new();

    // --- palette (above the separator) -----------------------------------
    if !state.palette.is_empty() {
        let visible = &state.palette[..state.palette.len().min(PALETTE_MAX_ROWS)];
        let cmd_col = visible
            .iter()
            .map(|p| display_width(p.display))
            .max()
            .unwrap_or(0);
        let inner_target: usize = visible
            .iter()
            .map(|p| 2 + cmd_col + 2 + display_width(p.description))
            .max()
            .unwrap_or(0);
        let inner = inner_target.min(width.saturating_sub(4));
        lines.push(format!("  ┌{}┐", "─".repeat(inner)));
        for (i, item) in visible.iter().enumerate() {
            let marker = if i == state.palette_selected {
                "▸"
            } else {
                " "
            };
            let row = format!(
                "{} {:<cmd_col$}  {}",
                marker,
                item.display,
                item.description,
                cmd_col = cmd_col
            );
            let mut row = truncate_to_width(&row, inner);
            let pad = inner.saturating_sub(display_width(&row));
            row.push_str(&" ".repeat(pad));
            if i == state.palette_selected {
                lines.push(format!("  │\x1b[1;36m{}\x1b[0m│", row));
            } else {
                lines.push(format!("  │{}│", row));
            }
        }
        lines.push(format!("  └{}┘", "─".repeat(inner)));
    }

    // --- separator --------------------------------------------------------
    lines.push(dim(&"─".repeat(width)));

    // --- input line -------------------------------------------------------
    let before = &state.input[..state.cursor.min(state.input.len())];
    let after = &state.input[state.cursor.min(state.input.len())..];
    let avail = width.saturating_sub(PROMPT_WIDTH + 1);

    // Horizontally scroll so the cursor stays visible.
    let mut visible_before: &str = before;
    while display_width(visible_before) > avail {
        let mut iter = visible_before.char_indices();
        iter.next();
        match iter.next() {
            Some((idx, _)) => visible_before = &visible_before[idx..],
            None => {
                visible_before = "";
                break;
            }
        }
    }
    let before_w = display_width(visible_before);
    let after_visible = truncate_to_width(after, avail.saturating_sub(before_w));
    let cursor_row = lines.len();
    lines.push(format!("{}{}{}", PROMPT, visible_before, after_visible));
    let cursor_col = (PROMPT_WIDTH + before_w) as u16;

    // --- status line ------------------------------------------------------
    let status_text = match &state.spinner {
        Some(sp) => {
            let frame = SPINNER_FRAMES[sp.frame % SPINNER_FRAMES.len()];
            let mut s = format!(
                "  {} {} · {:.1}s · esc to cancel",
                frame, sp.label, sp.elapsed_secs
            );
            if state.queued > 0 {
                s.push_str(&format!(" · {} queued", state.queued));
            }
            s
        }
        None => {
            let tools = if state.status.tools_enabled {
                "tools on"
            } else {
                "tools off"
            };
            let mut s = format!(
                "  {} · {} · {} · {}",
                state.status.model,
                state.status.session,
                format_tokens(state.status.token_estimate),
                tools
            );
            if let Some(effort) = &state.status.effort {
                s.push_str(&format!(" · {}", effort));
            }
            s
        }
    };
    lines.push(dim(&truncate_to_width(&status_text, width)));

    AnchorLayout {
        lines,
        cursor_row,
        cursor_col,
    }
}

/// Draws and erases the anchor zone over a `Write` using crossterm commands.
///
/// Tracks how many lines were last drawn (the palette changes the count) and
/// where it left the terminal cursor, so it can fully erase itself before
/// repainting or before content is printed into scrollback above it.
pub struct AnchorRenderer {
    lines_drawn: u16,
    cursor_row: u16,
}

impl AnchorRenderer {
    pub fn new() -> Self {
        Self {
            lines_drawn: 0,
            cursor_row: 0,
        }
    }

    /// Current terminal width with a sane fallback.
    pub fn terminal_width() -> usize {
        terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
    }

    /// Erase the previously drawn anchor (if any), leaving the cursor at
    /// column 0 of what was the anchor's top line.
    pub fn erase<W: Write>(&mut self, w: &mut W) -> std::io::Result<()> {
        if self.lines_drawn == 0 {
            return Ok(());
        }
        let down = self.lines_drawn - 1 - self.cursor_row;
        queue!(w, cursor::MoveToColumn(0))?;
        if down > 0 {
            queue!(w, cursor::MoveDown(down))?;
        }
        queue!(w, terminal::Clear(terminal::ClearType::CurrentLine))?;
        for _ in 1..self.lines_drawn {
            queue!(
                w,
                cursor::MoveUp(1),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;
        }
        w.flush()?;
        self.lines_drawn = 0;
        self.cursor_row = 0;
        Ok(())
    }

    /// Erase the old anchor and paint the new one, leaving the terminal
    /// cursor on the input line at the editor's cursor position.
    pub fn draw<W: Write>(&mut self, w: &mut W, state: &AnchorState) -> std::io::Result<()> {
        self.erase(w)?;
        let layout = build_lines(state, Self::terminal_width());
        for (i, line) in layout.lines.iter().enumerate() {
            if i > 0 {
                w.write_all(b"\r\n")?;
            }
            w.write_all(line.as_bytes())?;
        }
        let up = (layout.lines.len() - 1 - layout.cursor_row) as u16;
        queue!(w, cursor::MoveToColumn(0))?;
        if up > 0 {
            queue!(w, cursor::MoveUp(up))?;
        }
        queue!(w, cursor::MoveToColumn(layout.cursor_col))?;
        w.flush()?;
        self.lines_drawn = layout.lines.len() as u16;
        self.cursor_row = layout.cursor_row as u16;
        Ok(())
    }

    /// Print content into the scrollback above the anchor: erase the anchor,
    /// write the content (LF is converted to CRLF for raw mode, and a
    /// trailing newline is ensured), then repaint the anchor.
    pub fn print_above<W: Write>(
        &mut self,
        w: &mut W,
        content: &str,
        state: &AnchorState,
    ) -> std::io::Result<()> {
        self.erase(w)?;
        let mut converted = content.replace('\n', "\r\n");
        if !converted.ends_with("\r\n") {
            converted.push_str("\r\n");
        }
        w.write_all(converted.as_bytes())?;
        w.flush()?;
        self.draw(w, state)
    }
}

impl Default for AnchorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_ansi(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for next in chars.by_ref() {
                        if next.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                continue;
            }
            output.push(ch);
        }
        output
    }

    fn idle_state(input: &str) -> AnchorState {
        AnchorState {
            input: input.to_string(),
            cursor: input.len(),
            status: StatusInfo {
                model: "claude-sonnet-4".to_string(),
                session: "git-help".to_string(),
                token_estimate: 2400,
                tools_enabled: false,
                effort: None,
            },
            spinner: None,
            queued: 0,
            palette: Vec::new(),
            palette_selected: 0,
        }
    }

    #[test]
    fn idle_anchor_is_three_lines() {
        let layout = build_lines(&idle_state(""), 72);
        assert_eq!(layout.lines.len(), 3);
        assert_eq!(strip_ansi(&layout.lines[0]), "─".repeat(72));
        assert_eq!(layout.lines[1], "  › ");
        assert_eq!(
            strip_ansi(&layout.lines[2]),
            "  claude-sonnet-4 · git-help · 2.4k tok · tools off"
        );
        // Cursor on the input line, right after the prompt
        assert_eq!(layout.cursor_row, 1);
        assert_eq!(layout.cursor_col, 4);
    }

    #[test]
    fn input_text_and_cursor_column() {
        let mut state = idle_state("hello");
        state.cursor = 2; // between 'e' and 'l'
        let layout = build_lines(&state, 80);
        assert_eq!(layout.lines[1], "  › hello");
        assert_eq!(layout.cursor_col, 6);
    }

    #[test]
    fn streaming_status_replaces_static_segments() {
        let mut state = idle_state("next question");
        state.spinner = Some(SpinnerInfo {
            frame: 3,
            elapsed_secs: 3.2,
            label: "thinking".to_string(),
        });
        let layout = build_lines(&state, 80);
        let status = strip_ansi(&layout.lines[2]);
        assert_eq!(status, "  ⠸ thinking · 3.2s · esc to cancel");
        // Input line still shows the editable buffer while streaming
        assert_eq!(layout.lines[1], "  › next question");
    }

    #[test]
    fn queued_count_shows_in_streaming_status() {
        let mut state = idle_state("");
        state.spinner = Some(SpinnerInfo {
            frame: 0,
            elapsed_secs: 1.0,
            label: "thinking".to_string(),
        });
        state.queued = 1;
        let layout = build_lines(&state, 80);
        assert!(strip_ansi(&layout.lines[2]).ends_with("· 1 queued"));
    }

    #[test]
    fn web_label_in_spinner_segment() {
        let mut state = idle_state("");
        state.spinner = Some(SpinnerInfo {
            frame: 0,
            elapsed_secs: 1.8,
            label: "🌐 searching the web…".to_string(),
        });
        let layout = build_lines(&state, 80);
        let status = strip_ansi(&layout.lines[2]);
        assert!(status.contains("🌐 searching the web…"));
        assert!(status.contains("1.8s"));
    }

    #[test]
    fn narrow_width_truncates_status() {
        let layout = build_lines(&idle_state(""), 24);
        let status = strip_ansi(&layout.lines[2]);
        assert!(display_width(&status) <= 24, "status too wide: {}", status);
        assert_eq!(strip_ansi(&layout.lines[0]), "─".repeat(24));
    }

    #[test]
    fn long_input_scrolls_horizontally_keeping_cursor_visible() {
        let text = "x".repeat(100);
        let state = idle_state(&text);
        let layout = build_lines(&state, 40);
        let input_line = &layout.lines[1];
        assert!(display_width(input_line) <= 40);
        // Cursor column must be inside the terminal
        assert!((layout.cursor_col as usize) < 40);
    }

    #[test]
    fn wide_unicode_input_cursor_accounts_for_double_width() {
        let state = idle_state("世界"); // two double-width chars
        let layout = build_lines(&state, 80);
        assert_eq!(layout.cursor_col, 4 + 4); // prompt + 2*2 columns
    }

    #[test]
    fn palette_renders_above_separator() {
        let mut state = idle_state("/sa");
        state.palette = filter_palette("/sa");
        let layout = build_lines(&state, 80);
        // box top, one row (/save), box bottom, separator, input, status
        assert_eq!(layout.lines.len(), 6);
        let plain: Vec<String> = layout.lines.iter().map(|l| strip_ansi(l)).collect();
        assert!(plain[0].starts_with("  ┌"));
        assert!(plain[1].contains("/save [name]"));
        assert!(plain[1].contains("Save session"));
        assert!(plain[2].starts_with("  └"));
        assert!(plain[3].starts_with("─"));
        assert_eq!(plain[4], "  › /sa");
        // Cursor row shifts down because of the palette
        assert_eq!(layout.cursor_row, 4);
    }

    #[test]
    fn palette_marks_selected_entry() {
        let mut state = idle_state("/s");
        state.palette = filter_palette("/s");
        assert!(state.palette.len() >= 2); // /save, /settings, /status, /streaming
        state.palette_selected = 1;
        let layout = build_lines(&state, 80);
        let plain: Vec<String> = layout.lines.iter().map(|l| strip_ansi(l)).collect();
        assert!(!plain[1].contains('▸')); // unselected row has no marker
        assert!(plain[1].contains('/'));
        assert!(plain[2].contains('▸'));
    }

    #[test]
    fn filter_palette_matches_prefixes_and_aliases() {
        let items = filter_palette("/he");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].completion, "/help");

        // Alias match: /q -> /exit entry
        let items = filter_palette("/q");
        assert!(items.iter().any(|i| i.completion == "/exit"));

        // Bare slash lists everything
        let all = filter_palette("/");
        assert_eq!(all.len(), ChatCommand::command_palette().len());
    }

    #[test]
    fn filter_palette_hides_for_non_slash_or_args() {
        assert!(filter_palette("hello").is_empty());
        assert!(filter_palette("").is_empty());
        assert!(filter_palette("/save mysession").is_empty());
    }

    #[test]
    fn filter_palette_no_match() {
        assert!(filter_palette("/zzz").is_empty());
    }

    #[test]
    fn token_formatting() {
        assert_eq!(format_tokens(0), "0 tok");
        assert_eq!(format_tokens(812), "812 tok");
        assert_eq!(format_tokens(2400), "2.4k tok");
        assert_eq!(format_tokens(13370), "13.4k tok");
    }

    #[test]
    fn effort_shows_in_status_when_set() {
        let mut state = idle_state("");
        state.status.model = "gpt-5.6-sol".to_string();
        state.status.effort = Some("ultra".to_string());
        let layout = build_lines(&state, 80);
        assert_eq!(
            strip_ansi(&layout.lines[2]),
            "  gpt-5.6-sol · git-help · 2.4k tok · tools off · ultra"
        );
    }

    #[test]
    fn effort_absent_from_status_by_default() {
        let layout = build_lines(&idle_state(""), 80);
        assert!(!strip_ansi(&layout.lines[2]).contains("ultra"));
        assert!(strip_ansi(&layout.lines[2]).ends_with("tools off"));
    }

    #[test]
    fn tools_on_shows_in_status() {
        let mut state = idle_state("");
        state.status.tools_enabled = true;
        let layout = build_lines(&state, 80);
        assert!(strip_ansi(&layout.lines[2]).ends_with("tools on"));
    }

    #[test]
    fn print_above_converts_lf_to_crlf_and_repaints() {
        let mut renderer = AnchorRenderer::new();
        let state = idle_state("");
        let mut buf: Vec<u8> = Vec::new();
        renderer.draw(&mut buf, &state).unwrap();
        assert_eq!(renderer.lines_drawn, 3);
        buf.clear();
        renderer
            .print_above(&mut buf, "  you › hi\nsecond", &state)
            .unwrap();
        let out = String::from_utf8_lossy(&buf);
        assert!(out.contains("  you › hi\r\nsecond\r\n"));
        // Anchor repainted after the content
        assert_eq!(renderer.lines_drawn, 3);
    }

    #[test]
    fn erase_resets_line_count() {
        let mut renderer = AnchorRenderer::new();
        let state = idle_state("");
        let mut buf: Vec<u8> = Vec::new();
        renderer.draw(&mut buf, &state).unwrap();
        renderer.erase(&mut buf).unwrap();
        assert_eq!(renderer.lines_drawn, 0);
        // Erasing an already-erased anchor writes nothing
        let mut buf2: Vec<u8> = Vec::new();
        renderer.erase(&mut buf2).unwrap();
        assert!(buf2.is_empty());
    }
}
