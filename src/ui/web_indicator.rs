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
