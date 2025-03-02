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
                print!(
                    "\r{} Thinking... {:02}:{:02}",
                    spinner_chars[spinner_idx],
                    elapsed.as_secs() / 60,
                    elapsed.as_secs() % 60
                );
                io::stdout().flush().unwrap();

                spinner_idx = (spinner_idx + 1) % spinner_chars.len();
                thread::sleep(Duration::from_millis(100));
            }
            print!("\r                                       \r");
            io::stdout().flush().unwrap();
        });

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for ThinkingTimer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
