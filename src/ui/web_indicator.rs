use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::Duration,
};

/// Transient animated indicator shown while a web tool executes.
///
/// Mirrors the self-clearing spinner pattern of `ThinkingTimer`: it redraws a
/// single line with `\r` and fully erases it (`\r\x1b[2K`) when stopped, so
/// nothing permanent is left in the transcript.
pub struct WebIndicator {
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl WebIndicator {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Start the indicator with the given message, e.g. "Searching the web…"
    pub fn start(&mut self, message: String) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut spinner_idx = 0;

            while running.load(Ordering::SeqCst) {
                print!(
                    "\r\x1b[36m{} 🌐 {}\x1b[0m",
                    spinner_chars[spinner_idx], message
                );
                io::stdout().flush().unwrap();

                spinner_idx = (spinner_idx + 1) % spinner_chars.len();
                thread::sleep(Duration::from_millis(150));
            }
            // Clear the indicator line completely and move cursor to start of line
            print!("\r\x1b[2K\r");
            io::stdout().flush().unwrap();
        });

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        // Wait for the thread to finish and clean up
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Extra cleanup to ensure line is cleared
        print!("\r\x1b[2K");
        io::stdout().flush().unwrap();
    }
}

impl Default for WebIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WebIndicator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Routing for web-tool activity when the bottom-anchored chat UI is active.
///
/// The anchored chat UI owns the terminal's bottom lines, so the standalone
/// `WebIndicator` spinner (which redraws its own line with `\r`) would fight
/// it. While anchored mode is active, web tools publish their activity here
/// instead and the chat status line renders it as a `🌐 …` spinner segment.
/// Non-chat paths (e.g. `ask`) keep using the standalone `WebIndicator`.
pub mod activity {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::time::Instant;

    static ANCHORED: AtomicBool = AtomicBool::new(false);
    static CURRENT: Mutex<Option<(String, Instant)>> = Mutex::new(None);

    /// Enable/disable anchored routing (set by the interactive chat UI).
    pub fn set_anchored(on: bool) {
        ANCHORED.store(on, Ordering::SeqCst);
        if !on {
            end();
        }
    }

    pub fn is_anchored() -> bool {
        ANCHORED.load(Ordering::SeqCst)
    }

    /// Record that a web tool started executing, e.g. "Searching the web…".
    pub fn begin(label: String) {
        if let Ok(mut current) = CURRENT.lock() {
            *current = Some((label, Instant::now()));
        }
    }

    /// Record that the web tool finished.
    pub fn end() {
        if let Ok(mut current) = CURRENT.lock() {
            *current = None;
        }
    }

    /// The in-flight web activity, if any: (label, elapsed seconds).
    pub fn current() -> Option<(String, f32)> {
        CURRENT.lock().ok().and_then(|c| {
            c.as_ref()
                .map(|(l, t)| (l.clone(), t.elapsed().as_secs_f32()))
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn activity_round_trip() {
            set_anchored(true);
            assert!(is_anchored());
            begin("Searching the web…".to_string());
            let (label, secs) = current().expect("activity should be set");
            assert_eq!(label, "Searching the web…");
            assert!(secs >= 0.0);
            end();
            assert!(current().is_none());
            set_anchored(false);
            assert!(!is_anchored());
        }
    }
}
