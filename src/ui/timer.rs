use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub struct ThinkingTimer {
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ThinkingTimer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut spinner_idx = 0;

            while running.load(Ordering::SeqCst) {
                let elapsed = start.elapsed();
                let secs = elapsed.as_secs();
                
                if secs < 60 {
                    print!("\r\x1b[36m{} 🤔 AI is thinking ({}.{}s)\x1b[0m", 
                        spinner_chars[spinner_idx],
                        secs, 
                        elapsed.subsec_millis() / 100
                    );
                } else {
                    print!("\r\x1b[36m{} 🤔 AI is thinking ({:02}:{:02})\x1b[0m", 
                        spinner_chars[spinner_idx],
                        secs / 60,
                        secs % 60
                    );
                }
                io::stdout().flush().unwrap();

                spinner_idx = (spinner_idx + 1) % spinner_chars.len();
                thread::sleep(Duration::from_millis(150));
            }
            // Clear the thinking line completely and move cursor to start of line
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
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}

impl Drop for ThinkingTimer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
